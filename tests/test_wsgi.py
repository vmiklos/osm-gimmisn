#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi module covers the wsgi module."""

from typing import Any
from typing import BinaryIO
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import io
import json
import locale
import os
import unittest
import unittest.mock
import urllib.error
import xml.etree.ElementTree as ET

import yattag

import areas
import util
import webframe
import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_relations() -> areas.Relations:
    """Returns a Relations object that uses the test data and workdir."""
    workdir = os.path.join(os.path.dirname(__file__), "workdir")
    return areas.Relations(workdir)


def get_abspath(path: str) -> str:
    """Mock get_abspath() that uses the test directory."""
    if os.path.isabs(path):
        return path
    return os.path.join(os.path.dirname(__file__), path)


class TestWsgi(unittest.TestCase):
    """Base class for wsgi tests."""
    def get_dom_for_path(self, path: str) -> ET.Element:
        """Generates an XML DOM for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        with unittest.mock.patch('util.get_abspath', get_abspath):
            prefix = util.Config.get_uri_prefix()
            environ = {
                "PATH_INFO": prefix + path
            }
            callback = cast('StartResponse', start_response)
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
            header_dict = dict(response_headers)
            if path.endswith(".chkl"):
                self.assertEqual(header_dict["Content-type"], "application/octet-stream")
            else:
                self.assertEqual(header_dict["Content-type"], "text/plain; charset=utf-8")

        with unittest.mock.patch('util.get_abspath', get_abspath):
            prefix = util.Config.get_uri_prefix()
            environ = {
                "PATH_INFO": prefix + path
            }
            callback = cast('StartResponse', start_response)
            output_iterable = wsgi.application(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            return output

    def get_js_for_path(self, path: str) -> str:
        """Generates a string for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "application/x-javascript; charset=utf-8")

        with unittest.mock.patch('util.get_abspath', get_abspath):
            prefix = util.Config.get_uri_prefix()
            environ = {
                "PATH_INFO": prefix + path
            }
            callback = cast('StartResponse', start_response)
            output_iterable = wsgi.application(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            return output


class TestStreets(TestWsgi):
    """Tests handle_streets()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/streets/gazdagret/view-query")
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
            root = self.get_dom_for_path("/streets/gazdagret/update-result")
            results = root.findall("body")
            self.assertEqual(len(results), 1)

    def test_update_result_error_well_formed(self) -> None:
        """Tests if the update-result output on error is well-formed."""

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            raise urllib.error.HTTPError(url=None, code=None, msg=None, hdrs=None, fp=None)
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/streets/gazdagret/update-result")
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
            root = self.get_dom_for_path("/streets/ujbuda/update-result")
            results = root.findall("body")
            self.assertEqual(len(results), 1)


class TestMissingHousenumbers(TestWsgi):
    """Tests the missing house numbers page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_such_relation(self) -> None:
        """Tests the output for a non-existing relation."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret42/view-result")
        results = root.findall("body/div[@id='no-such-relation-error']")
        self.assertEqual(len(results), 1)

    def test_well_formed_compat(self) -> None:
        """Tests if the output is well-formed (URL rewrite)."""
        root = self.get_dom_for_path("/suspicious-streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_well_formed_compat_relation(self) -> None:
        """Tests if the output is well-formed (URL rewrite for relation name)."""
        root = self.get_dom_for_path("/suspicious-streets/budapest_22/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-osm-streets']")
                self.assertEqual(len(results), 1)

    def test_no_osm_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm housenumbers case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-osm-housenumbers']")
                self.assertEqual(len(results), 1)

    def test_no_ref_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref housenumbers case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_ref_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-ref-housenumbers']")
                self.assertEqual(len(results), 1)

    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        result = self.get_txt_for_path("/missing-housenumbers/budafok/view-result.txt")
        # Note how 12 is ordered after 2.
        self.assertEqual(result, "Vöröskúti határsor\t[2, 12, 34, 36*]")

    def test_view_result_txt_even_odd(self) -> None:
        """Tests the txt output (even-odd streets)."""
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt")
        expected = """Hamzsabégi út	[1]
Törökugrató utca	[7], [10]
Tűzkő utca	[1], [2]"""
        self.assertEqual(result, expected)

    def test_view_result_chkl(self) -> None:
        """Tests the chkl output."""
        result = self.get_txt_for_path("/missing-housenumbers/budafok/view-result.chkl")
        # Note how 12 is ordered after 2.
        self.assertEqual(result, "[ ] Vöröskúti határsor [2, 12, 34, 36*]")

    def test_view_result_chkl_even_odd(self) -> None:
        """Tests the chkl output (even-odd streets)."""
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
        expected = """[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [7], [10]
[ ] Tűzkő utca [1], [2]"""
        self.assertEqual(result, expected)

    def test_view_result_chkl_even_odd_split(self) -> None:
        """Tests the chkl output (even-odd streets)."""
        def mock_format_even_odd(_only_in_ref: List[str], doc: Optional[yattag.doc.Doc]) -> List[str]:
            assert doc is None
            return ["1, 3", "2, 4"]

        def mock_get_chkl_split_limit() -> int:
            return 1

        with unittest.mock.patch("util.format_even_odd", mock_format_even_odd):
            with unittest.mock.patch("wsgi.get_chkl_split_limit", mock_get_chkl_split_limit):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
                expected = """[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [1, 3]
[ ] Törökugrató utca [2, 4]
[ ] Tűzkő utca [1, 3]
[ ] Tűzkő utca [2, 4]"""
                self.assertEqual(result, expected)

    def test_view_result_chkl_no_osm_streets_hn(self) -> None:
        """Tests the chkl output, no osm streets/hn case."""
        hide_path = ""
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)

        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_streets_path()
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
                self.assertEqual(result, "No existing streets")

        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_housenumbers_path()
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
                self.assertEqual(result, "No existing house numbers")

    def test_view_result_chkl_no_ref_housenumbers(self) -> None:
        """Tests the chkl output, no ref housenumbers case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_ref_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
                self.assertEqual(result, "No reference house numbers")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt")
                self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_osm_housenumbers(self) -> None:
        """Tests the txt output, no osm housenumbers case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt")
                self.assertEqual(result, "No existing house numbers")

    def test_view_result_txt_no_ref_housenumbers(self) -> None:
        """Tests the txt output, no ref housenumbers case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_ref_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt")
                self.assertEqual(result, "No reference house numbers")

    def test_view_turbo_well_formed(self) -> None:
        """Tests if the view-turbo output is well-formed."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-turbo")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result_link(self) -> None:
        """Tests if the update-result output links back to the correct page."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/update-result")
        prefix = util.Config.get_uri_prefix()
        results = root.findall("body/a[@href='" + prefix + "/missing-housenumbers/gazdagret/view-result']")
        self.assertEqual(len(results), 1)


class TestStreetHousenumbers(TestWsgi):
    """Tests handle_street_housenumbers()."""
    def test_view_result_update_result_link(self) -> None:
        """Tests view result: the update-result link."""
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/view-result")
        uri = util.Config.get_uri_prefix() + "/missing-housenumbers/gazdagret/view-result"
        results = root.findall("body/div[@id='toolbar']/a[@href='" + uri + "']")
        self.assertTrue(results)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/view-query")
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
            root = self.get_dom_for_path("/street-housenumbers/gazdagret/update-result")
            results = root.findall("body")
            self.assertEqual(len(results), 1)

    def test_update_result_error_well_formed(self) -> None:
        """Tests if the update-result output on error is well-formed."""

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            raise urllib.error.HTTPError(url=None, code=None, msg=None, hdrs=None, fp=None)
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_dom_for_path("/street-housenumbers/gazdagret/update-result")
            results = root.findall("body/div[@id='overpass-error']")
            self.assertTrue(results)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/street-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-osm-housenumbers']")
                self.assertEqual(len(results), 1)


class TestMissingStreets(TestWsgi):
    """Tests the missing streets page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_well_formed_compat(self) -> None:
        """Tests if the output is well-formed (URL rewrite)."""
        root = self.get_dom_for_path("/suspicious-relations/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/missing-streets/gazdagret/view-result")
                results = root.findall("body/div[@id='no-osm-streets']")
                self.assertEqual(len(results), 1)

    def test_no_ref_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_ref_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/missing-streets/gazdagret/view-result")
                results = root.findall("body/div[@id='no-ref-streets']")
                self.assertEqual(len(results), 1)

    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "Only In Ref utca")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.txt")
                self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_ref_streets(self) -> None:
        """Tests the txt output, no ref streets case."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            relations = get_relations()
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_ref_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.txt")
                self.assertEqual(result, "No reference streets")

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result(self) -> None:
        """Tests the update-result output."""
        root = self.get_dom_for_path("/missing-streets/gazdagret/update-result")
        results = root.findall("body/div[@id='update-success']")
        self.assertEqual(len(results), 1)


class TestMain(TestWsgi):
    """Tests handle_main()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_path(self) -> None:
        """Tests the case when PATH_INFO is empty (should give the main page)."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            environ = {
                "PATH_INFO": ""
            }
            ret = webframe.get_request_uri(environ, get_relations())
            self.assertEqual(ret, "")

    def test_filter_for_incomplete_well_formed(self) -> None:
        """Tests if the /osm/filter-for/incomplete output is well-formed."""
        root = self.get_dom_for_path("/filter-for/incomplete")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refcounty_well_formed(self) -> None:
        """Tests if the /osm/filter-for/refcounty output is well-formed."""
        root = self.get_dom_for_path("/filter-for/refcounty/01")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refcounty_no_refsettlement(self) -> None:
        """Tests if the /osm/filter-for/refcounty output is well-formed."""
        root = self.get_dom_for_path("/filter-for/refcounty/67")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refcounty_refsettlement_well_formed(self) -> None:
        """Tests if the /osm/filter-for/refcounty/<value>/refsettlement/<value> output is well-formed."""
        root = self.get_dom_for_path("/filter-for/refcounty/01/refsettlement/011")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_custom_locale(self) -> None:
        """Tests the main page with a custom locale."""
        with util.ConfigContext("locale", "en_US.UTF-8"):
            root = self.get_dom_for_path("")
            results = root.findall("body/table")
            self.assertEqual(len(results), 1)

    def test_failing_locale(self) -> None:
        """Tests the main page with a failing locale."""

        def mock_setlocale(category: int, locale_name: str) -> str:
            raise locale.Error()
        with unittest.mock.patch('locale.setlocale', mock_setlocale):
            root = self.get_dom_for_path("")
            results = root.findall("body/table")
            self.assertEqual(len(results), 1)

    def test_application_exception(self) -> None:
        """Tests application(), exception catching case."""
        environ = {
            "PATH_INFO": "/"
        }

        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            self.assertTrue(status.startswith("500"))
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        def mock_application(environ: Dict[str, Any], start_response: 'StartResponse') -> Iterable[bytes]:
            int("a")
            # Never reached.
            return wsgi.our_application(environ, start_response)

        with unittest.mock.patch('wsgi.our_application', mock_application):
            callback = cast('StartResponse', start_response)
            output_iterable = wsgi.application(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            self.assertIn("ValueError", output)

    def test_main(self) -> None:
        """Tests main()."""
        serving = False

        class MockServer:
            """Mock WSGI server."""
            # pylint: disable=no-self-use
            def serve_forever(self) -> None:
                """Handles one request at a time until shutdown."""
                nonlocal serving
                serving = True

        def mock_make_server(_host: str, _port: int, _app: Any) -> MockServer:
            """Creates a new mock WSGI server."""
            return MockServer()

        with unittest.mock.patch('wsgiref.simple_server.make_server', mock_make_server):
            # Capture standard output.
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                wsgi.main()
        self.assertTrue(serving)


class TestWebhooks(TestWsgi):
    """Tests /osm/webhooks/."""
    def test_github(self) -> None:
        """Tests /osm/webhooks/github."""
        environ: Dict[str, BinaryIO] = {}
        root = {"ref": "refs/heads/master"}
        payload = json.dumps(root)
        body = {"payload": [payload]}
        query_string = urllib.parse.urlencode(body, doseq=True)
        buf = io.BytesIO()
        buf.write(query_string.encode('utf-8'))
        buf.seek(0)
        environ["wsgi.input"] = buf
        actual_args: List[str] = []
        actual_check = False
        actual_path = ""

        def mock_subprocess_run(args: List[str], check: bool, env: Any) -> None:
            nonlocal actual_args
            nonlocal actual_check
            nonlocal actual_path
            actual_args = args
            actual_check = check
            actual_path = env["PATH"]

        with unittest.mock.patch('subprocess.run', mock_subprocess_run):
            wsgi.handle_github_webhook(environ)
        self.assertEqual(actual_args[0], "make")
        self.assertEqual(actual_args[-1], "deploy-pythonanywhere")
        self.assertTrue(actual_check)
        self.assertIn("osm-gimmisn-env/bin", actual_path)

    def test_github_branch(self) -> None:
        """Tests /osm/webhooks/github, the case when a non-master branch is updated."""
        environ: Dict[str, BinaryIO] = {}
        root = {"ref": "refs/heads/stable"}
        payload = json.dumps(root)
        body = {"payload": [payload]}
        query_string = urllib.parse.urlencode(body, doseq=True)
        buf = io.BytesIO()
        buf.write(query_string.encode('utf-8'))
        buf.seek(0)
        environ["wsgi.input"] = buf
        invoked = False

        def mock_subprocess_run(_args: List[str], _check: bool) -> None:
            nonlocal invoked

        with unittest.mock.patch('subprocess.run', mock_subprocess_run):
            wsgi.handle_github_webhook(environ)
        self.assertFalse(invoked)

    def test_route(self) -> None:
        """Tests the /osm/webhooks/github -> handle_github_webhook() routing."""

        mock_called = False

        def mock_handler(_environ: Dict[str, BinaryIO]) -> yattag.doc.Doc:
            nonlocal mock_called
            mock_called = True
            return util.html_escape("")

        with unittest.mock.patch("wsgi.handle_github_webhook", mock_handler):
            self.get_dom_for_path("/webhooks/github")
        self.assertTrue(mock_called)


class TestStatic(TestWsgi):
    """Tests /osm/static/."""
    def test_js(self) -> None:
        """Tests /osm/static/, javascript case."""
        result = self.get_js_for_path("/static/sorttable.js")
        # Starts with a JS comment.
        self.assertTrue(result.startswith("/*"))


class TestStats(TestWsgi):
    """Tests handle_stats()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/housenumber-stats/hungary/")
        results = root.findall("body/h2")
        # 6 chart types + note
        self.assertEqual(len(results), 7)


if __name__ == '__main__':
    unittest.main()
