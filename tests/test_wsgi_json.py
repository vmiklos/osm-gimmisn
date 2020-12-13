#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi_json module covers the wsgi_json module."""

from typing import Any
from typing import BinaryIO
from typing import Dict
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import io
import json
import os
import unittest
import unittest.mock
import urllib.error

import test_config

import areas
import config
import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_relations() -> areas.Relations:
    """Returns a Relations object that uses the test data and workdir."""
    workdir = os.path.join(os.path.dirname(__file__), "workdir")
    return areas.Relations(workdir)


class TestWsgiJson(test_config.TestCase):
    """Base class for wsgi_json tests."""

    def get_json_for_path(self, path: str) -> Dict[str, Any]:
        """Generates an json dict for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "application/json; charset=utf-8")

        prefix = config.Config.get_uri_prefix()
        environ = {
            "PATH_INFO": prefix + path
        }
        callback = cast('StartResponse', start_response)
        output_iterable = wsgi.application(environ, callback)
        output_list = cast(List[bytes], output_iterable)
        self.assertTrue(output_list)
        output = output_list[0].decode('utf-8')
        return cast(Dict[str, Any], json.loads(output))


class TestJsonStreets(TestWsgiJson):
    """Tests streets_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result json output is well-formed."""
        result_from_overpass = "@id\tname\n1\tTűzkő utca\n2\tTörökugrató utca\n3\tOSM Name 1\n4\tHamzsabégi út\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_json_for_path("/streets/gazdagret/update-result.json")
            self.assertEqual(root["error"], "")

    def test_update_result_json_error(self) -> None:
        """Tests if the update-result json output on error is well-formed."""

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            raise urllib.error.HTTPError(url="", code=0, msg="", hdrs={}, fp=io.BytesIO())
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_json_for_path("/streets/gazdagret/update-result.json")
            self.assertEqual(root["error"], "HTTP Error 0: ")


class TestJsonStreetHousenumbers(TestWsgiJson):
    """Tests street_housenumbers_update_result_json()."""
    def test_update_result_json(self) -> None:
        """Tests if the update-result output is well-formed."""
        result_from_overpass = "@id\taddr:street\taddr:housenumber\n"
        result_from_overpass += "1\tTörökugrató utca\t1\n"
        result_from_overpass += "1\tTörökugrató utca\t2\n"
        result_from_overpass += "1\tTűzkő utca\t9\n"
        result_from_overpass += "1\tTűzkő utca\t10\n"
        result_from_overpass += "1\tOSM Name 1\t1\n"
        result_from_overpass += "1\tOSM Name 1\t2\n"
        result_from_overpass += "1\tOnly In OSM utca\t1\n"
        result_from_overpass += "1\tSecond Only In OSM utca\t1\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_json_for_path("/street-housenumbers/gazdagret/update-result.json")
            self.assertEqual(root["error"], "")

    def test_update_result_error_json(self) -> None:
        """Tests if the update-result output on error is well-formed."""

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            raise urllib.error.HTTPError(url="", code=0, msg="", hdrs={}, fp=io.BytesIO())
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
            root = self.get_json_for_path("/street-housenumbers/gazdagret/update-result.json")
            self.assertEqual(root["error"], "HTTP Error 0: ")


if __name__ == '__main__':
    unittest.main()
