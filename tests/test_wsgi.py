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
import configparser
import io
import os
import unittest
import unittest.mock
import xml.etree.ElementTree as ET

import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


class TestStreetHousenumbers(unittest.TestCase):
    """Tests handle_street_housenumbers()."""
    def test_view_result_update_result_link(self) -> None:
        """Tests view result: the update-result link."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            self.assertEqual(status, "200 OK")
            header_dict = {key: value for (key, value) in response_headers}
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        def get_config() -> configparser.ConfigParser:
            config = configparser.ConfigParser()
            config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
            config.read(config_path)
            return config

        def get_datadir() -> str:
            return os.path.join(os.path.dirname(__file__), "data")

        def get_workdir(_config: configparser.ConfigParser) -> str:
            return os.path.join(os.path.dirname(__file__), "workdir")
        output = ""
        with unittest.mock.patch('wsgi.get_config', get_config):
            with unittest.mock.patch('wsgi.get_datadir', get_datadir):
                with unittest.mock.patch('helpers.get_workdir', get_workdir):
                    environ = {
                        "PATH_INFO": "/osm/street-housenumbers/gazdagret/view-result"
                    }
                    callback = cast('StartResponse', start_response)  # type: StartResponse
                    output_iterable = wsgi.application(environ, callback)
                    output_list = cast(List[bytes], output_iterable)
                    self.assertTrue(output_list)
                    output = output_list[0].decode('utf-8')
        stream = io.StringIO(output)
        tree = ET.parse(stream)
        root = tree.getroot()
        results = root.findall("body/div[@id='toolbar']/a[@href='/osm/suspicious-streets/gazdagret/view-result']")
        self.assertTrue(results)


if __name__ == '__main__':
    unittest.main()
