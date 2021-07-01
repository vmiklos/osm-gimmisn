#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi_json module covers the wsgi_json module."""

from typing import Any
from typing import Dict
from typing import List
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import json
import unittest

import test_config

import config
import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


class TestWsgiJson(unittest.TestCase):
    """Base class for wsgi_json tests."""

    def get_json_for_path(self, conf: config.Config, path: str) -> Dict[str, Any]:
        """Generates an json dict for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "application/json; charset=utf-8")

        prefix = conf.get_uri_prefix()
        environ = {
            "PATH_INFO": prefix + path
        }
        callback = cast('StartResponse', start_response)
        output_iterable = wsgi.application(environ, callback, conf)
        output_list = cast(List[str], output_iterable)
        self.assertTrue(output_list)
        output = output_list[0]
        return cast(Dict[str, Any], json.loads(output))


class TestJsonStreets(TestWsgiJson):
    """Tests streets_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/interpreter",
                                 data_path="",
                                 result_path="tests/network/overpass-streets-gazdagret.csv")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        root = self.get_json_for_path(conf, "/streets/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")

    def test_update_result_json_error(self) -> None:
        """Tests if the update-result json output on error is well-formed."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/interpreter",
                                 data_path="",
                                 result_path="")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        root = self.get_json_for_path(conf, "/streets/gazdagret/update-result.json")
        self.assertIn("error", root)


class TestJsonStreetHousenumbers(TestWsgiJson):
    """Tests street_housenumbers_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result output is well-formed."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/interpreter",
                                 data_path="",
                                 result_path="tests/network/overpass-housenumbers-gazdagret.csv")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        root = self.get_json_for_path(conf, "/street-housenumbers/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")

    def test_update_result_error_json(self) -> None:
        """Tests if the update-result output on error is well-formed."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/interpreter",
                                 data_path="",
                                 result_path="")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        root = self.get_json_for_path(conf, "/street-housenumbers/gazdagret/update-result.json")
        self.assertIn("error", root)


class TestJsonMissingHousenumbers(TestWsgiJson):
    """Tests missing_housenumbers_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        conf = test_config.make_test_config()
        root = self.get_json_for_path(conf, "/missing-housenumbers/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")


class TestJsonMissingStreets(TestWsgiJson):
    """Tests missing_streets_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        conf = test_config.make_test_config()
        root = self.get_json_for_path(conf, "/missing-streets/gazdagret/update-result.json")
        self.assertEqual(root["error"], "")


if __name__ == '__main__':
    unittest.main()
