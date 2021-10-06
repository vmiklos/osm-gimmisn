#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi_json module covers the wsgi_json module."""

from typing import Any
from typing import Dict
from typing import List
from typing import cast
import io
import json
import unittest

import test_context

import rust
import wsgi


class TestWsgiJson(unittest.TestCase):
    """Base class for wsgi_json tests."""

    def get_json_for_path(self, ctx: rust.PyContext, path: str) -> Dict[str, Any]:
        """Generates an json dict for a given wsgi path."""
        prefix = ctx.get_ini().get_uri_prefix()
        request_headers = {
            "PATH_INFO": prefix + path
        }
        status, response_headers, output_bytes_list = wsgi.application(request_headers, bytes(), ctx)
        self.assertEqual(status, "200 OK")
        header_dict = dict(response_headers)
        self.assertEqual(header_dict["Content-type"], "application/json; charset=utf-8")
        output_list = cast(List[str], output_bytes_list)
        self.assertTrue(output_list)
        output = output_list[0]
        return cast(Dict[str, Any], json.loads(output))


class TestJsonStreets(TestWsgiJson):
    """Tests streets_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-streets-gazdagret.csv")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/streets-gazdagret.csv"): streets_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        root = self.get_json_for_path(ctx, "/streets/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")
        self.assertTrue(streets_value.tell())

    def test_update_result_json_error(self) -> None:
        """Tests if the update-result json output on error is well-formed."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        root = self.get_json_for_path(ctx, "/streets/gazdagret/update-result.json")
        self.assertIn("error", root)


class TestJsonStreetHousenumbers(TestWsgiJson):
    """Tests street_housenumbers_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result output is well-formed."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-housenumbers-gazdagret.csv")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        file_system = test_context.TestFileSystem()
        housenumbers_value = io.BytesIO()
        housenumbers_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv"): housenumbers_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        root = self.get_json_for_path(ctx, "/street-housenumbers/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")
        self.assertTrue(housenumbers_value.tell())

    def test_update_result_error_json(self) -> None:
        """Tests if the update-result output on error is well-formed."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        root = self.get_json_for_path(ctx, "/street-housenumbers/gazdagret/update-result.json")
        self.assertIn("error", root)


class TestJsonMissingHousenumbers(TestWsgiJson):
    """Tests missing_housenumbers_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        housenumbers_value = io.BytesIO()
        housenumbers_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"): housenumbers_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        root = self.get_json_for_path(ctx, "/missing-housenumbers/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")
        self.assertTrue(housenumbers_value.tell())


class TestJsonMissingStreets(TestWsgiJson):
    """Tests missing_streets_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        streets_value = io.BytesIO()
        streets_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/streets-reference-gazdagret.lst"): streets_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        root = self.get_json_for_path(ctx, "/missing-streets/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")
        self.assertTrue(streets_value.tell())


if __name__ == '__main__':
    unittest.main()
