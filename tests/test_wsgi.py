#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi module covers the wsgi module."""

from typing import BinaryIO
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import io
import os
import unittest
import unittest.mock
import urllib.error
import xml.etree.ElementTree as ET

import areas
import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_relations() -> areas.Relations:
    """Returns a Relations object that uses the test data and workdir."""
    datadir = os.path.join(os.path.dirname(__file__), "data")
    workdir = os.path.join(os.path.dirname(__file__), "workdir")
    return areas.Relations(datadir, workdir)


class TestWsgi(unittest.TestCase):
    """Base class for wsgi tests."""
    def get_dom_for_path(self, path: str) -> ET.Element:
        """Generates an XML DOM for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = {key: value for (key, value) in response_headers}
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        def get_abspath(path: str) -> str:
            if os.path.isabs(path):
                return path
            return os.path.join(os.path.dirname(__file__), path)
        with unittest.mock.patch('util.get_abspath', get_abspath):
            environ = {
                "PATH_INFO": path
            }
            callback = cast('StartResponse', start_response)  # type: StartResponse
            output_iterable = wsgi.application(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            stream = io.StringIO(output)
            tree = ET.parse(stream)
            root = tree.getroot()
            return root

    def get_txt_for_path(self, path: str) -> str:
        """Generates a string for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = {key: value for (key, value) in response_headers}
            self.assertEqual(header_dict["Content-type"], "text/plain; charset=utf-8")

        def get_abspath(path: str) -> str:
            if os.path.isabs(path):
                return path
            return os.path.join(os.path.dirname(__file__), path)
        with unittest.mock.patch('util.get_abspath', get_abspath):
            environ = {
                "PATH_INFO": path
            }
            callback = cast('StartResponse', start_response)  # type: StartResponse
            output_iterable = wsgi.application(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            return output


class TestStreets(TestWsgi):
    """Tests handle_streets()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm/streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/osm/streets/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result_well_formed(self) -> None:
        """Tests if the update-result output is well-formed."""
        result_from_overpass = "@id\tname\n1\tTűzkő utca\n2\tTörökugrató utca\n3\tOSM Name 1\n4\tHamzsabégi út\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/osm/streets/gazdagret/update-result")
            results = root.findall("body")
            self.assertEqual(len(results), 1)

    def test_update_result_error_well_formed(self) -> None:
        """Tests if the update-result output on error is well-formed."""

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            raise urllib.error.HTTPError(url=None, code=None, msg=None, hdrs=None, fp=None)
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/osm/streets/gazdagret/update-result")
            results = root.findall("body/div[@id='overpass-error']")
            self.assertTrue(results)

    def test_update_result_missing_streets_well_formed(self) -> None:
        """
        Tests if the update-result output is well-formed for should_check_missing_streets() ==
        "only".
        """
        result_from_overpass = "@id\tname\n3\tOSM Name 1\n2\tTörökugrató utca\n1\tTűzkő utca\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/osm/streets/ujbuda/update-result")
            results = root.findall("body")
            self.assertEqual(len(results), 1)


class TestMissingHousenumbers(TestWsgi):
    """Tests the missing house numbers page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-result")
            results = root.findall("body/div[@id='no-osm-streets']")
            self.assertEqual(len(results), 1)

    def test_no_osm_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm housenumbers case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-result")
            results = root.findall("body/div[@id='no-osm-housenumbers']")
            self.assertEqual(len(results), 1)

    def test_no_ref_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref housenumbers case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_housenumbers_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-result")
            results = root.findall("body/div[@id='no-ref-housenumbers']")
            self.assertEqual(len(results), 1)

    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        result = self.get_txt_for_path("/osm/missing-housenumbers/budafok/view-result.txt")
        # Note how 12 is ordered after 2.
        self.assertEqual(result, "Vöröskúti határsor\t[2, 12, 34, 36*]")

    def test_view_result_txt_even_odd(self) -> None:
        """Tests the txt output (even-odd streets)."""
        result = self.get_txt_for_path("/osm/missing-housenumbers/gazdagret/view-result.txt")
        expected = """Hamzsabégi út	[1]
Törökugrató utca	[7], [10]
Tűzkő utca	[1], [2]"""
        self.assertEqual(result, expected)

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/osm/missing-housenumbers/gazdagret/view-result.txt")
            self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_osm_housenumbers(self) -> None:
        """Tests the txt output, no osm housenumbers case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/osm/missing-housenumbers/gazdagret/view-result.txt")
            self.assertEqual(result, "No existing house numbers")

    def test_view_result_txt_no_ref_housenumbers(self) -> None:
        """Tests the txt output, no ref housenumbers case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_housenumbers_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/osm/missing-housenumbers/gazdagret/view-result.txt")
            self.assertEqual(result, "No reference house numbers")

    def test_view_turbo_well_formed(self) -> None:
        """Tests if the view-turbo output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-turbo")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result_link(self) -> None:
        """Tests if the update-result output links back to the correct page."""
        root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/update-result")
        results = root.findall("body/a[@href='/osm/missing-housenumbers/gazdagret/view-result']")
        self.assertEqual(len(results), 1)


class TestStreetHousenumbers(TestWsgi):
    """Tests handle_street_housenumbers()."""
    def test_view_result_update_result_link(self) -> None:
        """Tests view result: the update-result link."""
        root = self.get_dom_for_path("/osm/street-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='toolbar']/a[@href='/osm/missing-housenumbers/gazdagret/view-result']")
        self.assertTrue(results)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/osm/street-housenumbers/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result_well_formed(self) -> None:
        """Tests if the update-result output is well-formed."""
        result_from_overpass = "@id\taddr:street\taddr:housenumber\n"
        result_from_overpass += "1\tTörökugrató utca\t1\n"
        result_from_overpass += "1\tTörökugrató utca\t2\n"
        result_from_overpass += "1\tTűzkő utca\t9\n"
        result_from_overpass += "1\tTűzkő utca\t10\n"
        result_from_overpass += "1\tOSM Name 1\t1\n"
        result_from_overpass += "1\tOSM Name 1\t2\n"
        result_from_overpass += "1\tOnly In OSM utca\t1\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/osm/street-housenumbers/gazdagret/update-result")
            results = root.findall("body")
            self.assertEqual(len(results), 1)

    def test_update_result_error_well_formed(self) -> None:
        """Tests if the update-result output on error is well-formed."""

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            raise urllib.error.HTTPError(url=None, code=None, msg=None, hdrs=None, fp=None)
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/osm/street-housenumbers/gazdagret/update-result")
            results = root.findall("body/div[@id='overpass-error']")
            self.assertTrue(results)


class TestMissingStreets(TestWsgi):
    """Tests the missing streets page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            root = self.get_dom_for_path("/osm/missing-streets/gazdagret/view-result")
            results = root.findall("body/div[@id='no-osm-streets']")
            self.assertEqual(len(results), 1)

    def test_no_ref_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            root = self.get_dom_for_path("/osm/missing-streets/gazdagret/view-result")
            results = root.findall("body/div[@id='no-ref-streets']")
            self.assertEqual(len(results), 1)

    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        result = self.get_txt_for_path("/osm/missing-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "Only In Ref utca")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/osm/missing-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_ref_streets(self) -> None:
        """Tests the txt output, no ref streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/osm/missing-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "No reference streets")

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-streets/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result(self) -> None:
        """Tests the update-result output."""
        root = self.get_dom_for_path("/osm/missing-streets/gazdagret/update-result")
        results = root.findall("body/div[@id='update-success']")
        self.assertEqual(len(results), 1)


class TestMain(TestWsgi):
    """Tests handle_main()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_incomplete_well_formed(self) -> None:
        """Tests if the /osm/filter-for/incomplete output is well-formed."""
        root = self.get_dom_for_path("/osm/filter-for/incomplete")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refmegye_well_formed(self) -> None:
        """Tests if the /osm/filter-for/refmegye output is well-formed."""
        root = self.get_dom_for_path("/osm/filter-for/refmegye/01")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refmegye_reftelepules_well_formed(self) -> None:
        """Tests if the /osm/filter-for/refmegye/<value>/reftelepules/<value> output is well-formed."""
        root = self.get_dom_for_path("/osm/filter-for/refmegye/01/reftelepules/011")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)


if __name__ == '__main__':
    unittest.main()
