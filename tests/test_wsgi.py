#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi module covers the wsgi module."""

from typing import List
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import io
import os
import unittest
import unittest.mock
import xml.etree.ElementTree as ET

import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


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


class TestMissingHousenumbers(TestWsgi):
    """Tests the missing house numbers page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-result")
        results = root.findall("body/table")
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

    def test_view_turbo_well_formed(self) -> None:
        """Tests if the view-turbo output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-housenumbers/gazdagret/view-turbo")
        results = root.findall("body/pre")
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


class TestMissingStreets(TestWsgi):
    """Tests the missing streets page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm/missing-streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)


class TestMain(TestWsgi):
    """Tests handle_main()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/osm")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)


class TestGetStaticdir(unittest.TestCase):
    """Tests get_staticdir()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        actual = wsgi.get_staticdir()
        expected = os.path.join(os.getcwd(), "static")
        self.assertEqual(actual, expected)


if __name__ == '__main__':
    unittest.main()
