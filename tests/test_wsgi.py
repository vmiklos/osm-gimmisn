#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi module covers the wsgi module."""

from typing import Any
from typing import Container
from typing import Dict
from typing import List
from typing import cast
import calendar
import datetime
import io
import json
import unittest
import urllib.error
import urllib.parse
import xml.etree.ElementTree as ET
import xmlrpc.client

import test_context

import areas
import webframe
import wsgi


class TestWsgi(unittest.TestCase):
    """Base class for wsgi tests."""
    def __init__(self, method_name: str) -> None:
        unittest.TestCase.__init__(self, method_name)
        self.gzip_compress = False
        self.ctx = test_context.make_test_context()
        self.environ: Dict[str, Any] = {}
        self.bytes = bytes()

    def get_dom_for_path(self, path: str, absolute: bool = False, expected_status: str = "") -> ET.Element:
        """Generates an XML DOM for a given wsgi path."""
        if not expected_status:
            expected_status = "200 OK"

        prefix = self.ctx.get_ini().get_uri_prefix()
        if not absolute:
            path = prefix + path
        self.environ["PATH_INFO"] = path
        if self.gzip_compress:
            self.environ["HTTP_ACCEPT_ENCODING"] = "gzip, deflate"
        status, response_headers, output_list = wsgi.application(self.environ, self.bytes, self.ctx)
        # Make sure the built-in exception catcher is not kicking in.
        self.assertEqual(status, expected_status)
        header_dict = dict(response_headers)
        self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")
        self.assertTrue(output_list)
        if self.gzip_compress:
            output_bytes = xmlrpc.client.gzip_decode(output_list[0])
        else:
            output_bytes = output_list[0]
        output = output_bytes.decode('utf-8')
        stream = io.StringIO(output)
        tree = ET.parse(stream)
        return tree.getroot()

    def get_txt_for_path(self, path: str) -> str:
        """Generates a string for a given wsgi path."""
        prefix = self.ctx.get_ini().get_uri_prefix()
        environ = {
            "PATH_INFO": prefix + path
        }
        status, response_headers, output_list = wsgi.application(environ, bytes(), self.ctx)
        # Make sure the built-in exception catcher is not kicking in.
        self.assertEqual(status, "200 OK")
        header_dict = dict(response_headers)
        if path.endswith(".chkl"):
            self.assertEqual(header_dict["Content-type"], "application/octet-stream")
        else:
            self.assertEqual(header_dict["Content-type"], "text/plain; charset=utf-8")
        self.assertTrue(output_list)
        output = output_list[0].decode('utf-8')
        return output

    def get_css_for_path(self, path: str) -> str:
        """Generates a string for a given wsgi path."""
        prefix = self.ctx.get_ini().get_uri_prefix()
        environ = {
            "PATH_INFO": prefix + path
        }
        status, response_headers, output_list = wsgi.application(environ, bytes(), self.ctx)
        # Make sure the built-in exception catcher is not kicking in.
        self.assertEqual(status, "200 OK")
        header_dict = dict(response_headers)
        self.assertEqual(header_dict["Content-type"], "text/css; charset=utf-8")
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
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-streets-gazdagret.csv"),
        ]
        network = test_context.TestNetwork(routes)
        self.ctx.set_network(network)
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/streets-gazdagret.csv"): streets_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/streets/gazdagret/update-result")
        self.assertTrue(streets_value.tell())
        results = root.findall("body")
        self.assertEqual(len(results), 1)

    def test_update_result_error_well_formed(self) -> None:
        """Tests if the update-result output on error is well-formed."""
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path=""),  # no result -> error
        ]
        network = test_context.TestNetwork(routes)
        self.ctx.set_network(network)
        root = self.get_dom_for_path("/streets/gazdagret/update-result")
        results = root.findall("body/div[@id='overpass-error']")
        self.assertTrue(results)

    def test_update_result_missing_streets_well_formed(self) -> None:
        """
        Tests if the update-result output is well-formed for should_check_missing_streets() ==
        "only".
        """
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-streets-ujbuda.csv"),
        ]
        network = test_context.TestNetwork(routes)
        self.ctx.set_network(network)
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/streets-ujbuda.csv"): streets_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/streets/ujbuda/update-result")
        self.assertTrue(streets_value.tell())
        results = root.findall("body")
        self.assertEqual(len(results), 1)


class TestMissingHousenumbers(TestWsgi):
    """Tests the missing house numbers page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)
        # refstreets: >0 invalid osm name
        results = root.findall("body/div[@id='osm-invalids-container']")
        self.assertEqual(len(results), 1)
        # refstreets: >0 invalid ref name
        results = root.findall("body/div[@id='ref-invalids-container']")
        self.assertEqual(len(results), 1)

    def test_no_such_relation(self) -> None:
        """Tests the output for a non-existing relation."""
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret42/view-result")
        results = root.findall("body/div[@id='no-such-relation-error']")
        self.assertEqual(len(results), 1)

    def test_well_formed_compat(self) -> None:
        """Tests if the output is well-formed (URL rewrite)."""
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        htmlcache_value = io.BytesIO()
        htmlcache_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/gazdagret.percent"): streets_value,
            self.ctx.get_abspath("workdir/gazdagret.htmlcache.en"): htmlcache_value,
        }
        file_system.set_files(files)
        # Make sure the cache is outdated.
        mtimes = {
            self.ctx.get_abspath("workdir/gazdagret.htmlcache.en"): 0.0,
        }
        file_system.set_mtimes(mtimes)
        self.ctx.set_file_system(file_system)

        root = self.get_dom_for_path("/suspicious-streets/gazdagret/view-result")

        self.assertTrue(streets_value.tell())
        self.assertTrue(htmlcache_value.tell())
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_well_formed_compat_relation(self) -> None:
        """Tests if the output is well-formed (URL rewrite for relation name)."""
        root = self.get_dom_for_path("/suspicious-streets/budapest_22/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-streets']")
        self.assertEqual(len(results), 1)

    def test_no_osm_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm housenumbers case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-housenumbers']")
        self.assertEqual(len(results), 1)

    def test_no_ref_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref housenumbers case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_housenumbers_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
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
        hoursnumbers_ref = """Hamzsabégi út	1
Ref Name 1	1
Ref Name 1	2
Törökugrató utca	1	comment
Törökugrató utca	10
Törökugrató utca	11
Törökugrató utca	12
Törökugrató utca	2
Törökugrató utca	7
Tűzkő utca	1
Tűzkő utca	2
Tűzkő utca	9
Tűzkő utca	10
Tűzkő utca	12
Tűzkő utca	13
Tűzkő utca	14
Tűzkő utca	15
Tűzkő utca	16
Tűzkő utca	17
Tűzkő utca	18
Tűzkő utca	19
Tűzkő utca	20
Tűzkő utca	21
Tűzkő utca	22
Tűzkő utca	22
Tűzkő utca	24
Tűzkő utca	25
Tűzkő utca	26
Tűzkő utca	27
Tűzkő utca	28
Tűzkő utca	29
Tűzkő utca	30
Tűzkő utca	31
"""
        hoursnumbers_ref_value = io.BytesIO()
        hoursnumbers_ref_value.write(hoursnumbers_ref.encode("utf-8"))
        hoursnumbers_ref_value.seek(0)
        file_system = test_context.TestFileSystem()
        hoursnumbers_ref_path = self.ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst")
        file_system.set_files({hoursnumbers_ref_path: hoursnumbers_ref_value})
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
        expected = """[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [7], [10]
[ ] Tűzkő utca [1, 13, 15, 17, 19, 21, 25, 27, 29, 31]
[ ] Tűzkő utca [2, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]"""
        self.assertEqual(result, expected)

    def test_view_result_chkl_no_osm_streets_hn(self) -> None:
        """Tests the chkl output, no osm streets/hn case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
        self.assertEqual(result, "No existing streets")

        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        file_system.set_hide_paths([hide_path])
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
        self.assertEqual(result, "No existing house numbers")

    def test_view_result_chkl_no_ref_housenumbers(self) -> None:
        """Tests the chkl output, no ref housenumbers case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_housenumbers_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl")
        self.assertEqual(result, "No reference house numbers")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt")
        self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_osm_housenumbers(self) -> None:
        """Tests the txt output, no osm housenumbers case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt")
        self.assertEqual(result, "No existing house numbers")

    def test_view_result_txt_no_ref_housenumbers(self) -> None:
        """Tests the txt output, no ref housenumbers case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_housenumbers_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
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
        file_system = test_context.TestFileSystem()
        housenumbers_value = io.BytesIO()
        housenumbers_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"): housenumbers_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-housenumbers/gazdagret/update-result")
        self.assertTrue(housenumbers_value.tell())
        ctx = test_context.make_test_context()
        prefix = ctx.get_ini().get_uri_prefix()
        results = root.findall("body/a[@href='" + prefix + "/missing-housenumbers/gazdagret/view-result']")
        self.assertEqual(len(results), 1)


class TestStreetHousenumbers(TestWsgi):
    """Tests handle_street_housenumbers()."""
    def test_view_result_update_result_link(self) -> None:
        """Tests view result: the update-result link."""
        ctx = test_context.make_test_context()
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/view-result")
        uri = ctx.get_ini().get_uri_prefix() + "/missing-housenumbers/gazdagret/view-result"
        results = root.findall("body/div[@id='toolbar']/a[@href='" + uri + "']")
        self.assertTrue(results)

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result_well_formed(self) -> None:
        """Tests if the update-result output is well-formed."""
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-housenumbers-gazdagret.csv"),
        ]
        network = test_context.TestNetwork(routes)
        self.ctx.set_network(network)
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv"): streets_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/update-result")
        self.assertTrue(streets_value.tell())
        results = root.findall("body")
        self.assertEqual(len(results), 1)

    def test_update_result_error_well_formed(self) -> None:
        """Tests if the update-result output on error is well-formed."""
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path=""),
        ]
        network = test_context.TestNetwork(routes)
        self.ctx.set_network(network)
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/update-result")
        results = root.findall("body/div[@id='overpass-error']")
        self.assertTrue(results)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/street-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-housenumbers']")
        self.assertEqual(len(results), 1)


class TestMissingStreets(TestWsgi):
    """Tests the missing streets page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/gazdagret-streets.percent"): streets_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-result")
        self.assertTrue(streets_value.tell())
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)
        # refstreets: >0 invalid osm name
        results = root.findall("body/div[@id='osm-invalids-container']")
        self.assertEqual(len(results), 1)
        # refstreets: >0 invalid ref name
        results = root.findall("body/div[@id='ref-invalids-container']")
        self.assertEqual(len(results), 1)

    def test_well_formed_compat(self) -> None:
        """Tests if the output is well-formed (URL rewrite)."""
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/gazdagret-streets.percent"): streets_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/suspicious-relations/gazdagret/view-result")
        self.assertTrue(streets_value.tell())
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-streets']")
        self.assertEqual(len(results), 1)

    def test_no_ref_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-result")
        results = root.findall("body/div[@id='no-ref-streets']")
        self.assertEqual(len(results), 1)

    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "Only In Ref utca\n")

    def test_view_result_chkl(self) -> None:
        """Tests the chkl output."""
        result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.chkl")
        self.assertEqual(result, "[ ] Only In Ref utca\n")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_ref_streets(self) -> None:
        """Tests the txt output, no ref streets case."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        result = self.get_txt_for_path("/missing-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "No reference streets")

    def test_view_query_well_formed(self) -> None:
        """Tests if the view-query output is well-formed."""
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-query")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)

    def test_update_result(self) -> None:
        """Tests the update-result output."""
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            self.ctx.get_abspath("workdir/streets-reference-gazdagret.lst"): streets_value,
        }
        file_system.set_files(files)
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/missing-streets/gazdagret/update-result")
        self.assertTrue(streets_value.tell())
        results = root.findall("body/div[@id='update-success']")
        self.assertEqual(len(results), 1)

    def test_view_turbo(self) -> None:
        """Tests the view-turbo output."""
        root = self.get_dom_for_path("/missing-streets/gazdagret/view-turbo")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)
        self.assertIn("OSM Name 1", cast(Container[Any], results[0].text))
        # This is silenced with `show-refstreet: false`.
        self.assertNotIn("OSM Name 2", cast(Container[Any], results[0].text))


class TestMain(TestWsgi):
    """Tests handle_main()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_path(self) -> None:
        """Tests the case when PATH_INFO is empty (should give the main page)."""
        environ = {
            "PATH_INFO": ""
        }
        ctx = test_context.make_test_context()
        relations = areas.make_relations(test_context.make_test_context())
        ret = webframe.get_request_uri(environ, ctx, relations)
        self.assertEqual(ret, "")

    def test_filter_for_incomplete_well_formed(self) -> None:
        """Tests if the /osm/filter-for/incomplete output is well-formed."""
        root = self.get_dom_for_path("/filter-for/incomplete")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_everything_well_formed(self) -> None:
        """Tests if the /osm/filter-for/everything output is well-formed."""
        root = self.get_dom_for_path("/filter-for/everything")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refcounty_well_formed(self) -> None:
        """Tests if the /osm/filter-for/refcounty output is well-formed."""
        root = self.get_dom_for_path("/filter-for/refcounty/01/whole-county")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refcounty_no_refsettlement(self) -> None:
        """Tests if the /osm/filter-for/refcounty output is well-formed."""
        root = self.get_dom_for_path("/filter-for/refcounty/67/whole-county")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_refcounty_refsettlement_well_formed(self) -> None:
        """Tests if the /osm/filter-for/refcounty/<value>/refsettlement/<value> output is well-formed."""
        root = self.get_dom_for_path("/filter-for/refcounty/01/refsettlement/011")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_filter_for_relations(self) -> None:
        """Tests if the /osm/filter-for/relations/... output is well-formed."""
        root = self.get_dom_for_path("/filter-for/relations/44,45")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)
        table = results[0]
        # header + the two requested relations
        self.assertEqual(len(table.getchildren()), 3)

    def test_filter_for_relations_empty(self) -> None:
        """Tests if the /osm/filter-for/relations/ output is well-formed."""
        root = self.get_dom_for_path("/filter-for/relations/")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)
        table = results[0]
        # header + no requested relations
        self.assertEqual(len(table.getchildren()), 1)

    def test_application_exception(self) -> None:
        """Tests application(), exception catching case."""
        ctx = test_context.make_test_context()
        ctx.set_unit(test_context.TestUnit())
        environ = {
            "PATH_INFO": "/"
        }

        status, response_headers, output_list = wsgi.application(environ, bytes(), ctx)
        self.assertTrue(status.startswith("500"))
        header_dict = dict(response_headers)
        self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        self.assertTrue(output_list)
        output = output_list[0].decode('utf-8')
        self.assertIn("TestError", output)


class TestWebhooks(TestWsgi):
    """Tests /osm/webhooks/."""
    def test_github(self) -> None:
        """Tests /osm/webhooks/github."""
        root = {"ref": "refs/heads/master"}
        payload = json.dumps(root)
        body = {"payload": [payload]}
        query_string = urllib.parse.urlencode(body, doseq=True)
        self.bytes = query_string.encode('utf-8')
        ctx = test_context.make_test_context()
        expected_args = "make -C " + ctx.get_abspath("") + " deploy"
        outputs = {
            expected_args: str()
        }
        subprocess = test_context.TestSubprocess(outputs)
        self.ctx.set_subprocess(subprocess)
        self.get_dom_for_path("/webhooks/github")
        self.assertTrue(subprocess.get_runs() != [])
        self.assertEqual(subprocess.get_exits(), [1])

    def test_github_branch(self) -> None:
        """Tests /osm/webhooks/github, the case when a non-master branch is updated."""
        ctx = test_context.make_test_context()
        outputs: Dict[str, str] = {}
        subprocess = test_context.TestSubprocess(outputs)
        ctx.set_subprocess(subprocess)
        root = {"ref": "refs/heads/stable"}
        payload = json.dumps(root)
        body = {"payload": [payload]}
        query_string = urllib.parse.urlencode(body, doseq=True)
        buf = query_string.encode('utf-8')
        webframe.handle_github_webhook(buf, ctx)
        self.assertEqual(subprocess.get_runs(), [])


class TestStatic(TestWsgi):
    """Tests /osm/static/."""
    def test_css(self) -> None:
        """Tests /osm/static/, css case."""
        result = self.get_css_for_path("/static/osm.min.css")
        # Starts with a JS comment.
        self.assertTrue(result.endswith("}"))

    def test_robots(self) -> None:
        """Tests robots.txt."""
        result = self.get_txt_for_path("/robots.txt")
        self.assertEqual(result, "User-agent: *\n")


class TestStats(TestWsgi):
    """Tests handle_stats()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/housenumber-stats/hungary/")
        results = root.findall("body/h2")
        # 8 chart types + note
        self.assertEqual(len(results), 9)


class TestStatsCityProgress(TestWsgi):
    """Tests handle_stats_cityprogress()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        time = test_context.TestTime(calendar.timegm(datetime.date(2019, 7, 17).timetuple()))
        self.ctx.set_time(time)
        root = self.get_dom_for_path("/housenumber-stats/hungary/cityprogress")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)


class TestInvalidRefstreets(TestWsgi):
    """Tests handle_invalid_refstreets()."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/housenumber-stats/hungary/invalid-relations")
        results = root.findall("body/h1/a")
        self.assertNotEqual(results, [])

    def test_no_osm_sreets(self) -> None:
        """Tests error handling when osm street list is missing for a relation."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.ctx.set_file_system(file_system)
        root = self.get_dom_for_path("/housenumber-stats/hungary/invalid-relations")
        results = root.findall("body")
        self.assertNotEqual(results, [])


class TestNotFound(TestWsgi):
    """Tests the not-found page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/asdf", absolute=True, expected_status="404 Not Found")
        results = root.findall("body/h1")
        self.assertNotEqual(results, [])


class TestCompress(TestWsgi):
    """Tests gzip compress case."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        self.gzip_compress = True
        root = self.get_dom_for_path("/")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)


if __name__ == '__main__':
    unittest.main()
