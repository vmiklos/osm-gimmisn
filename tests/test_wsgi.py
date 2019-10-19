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
import html.parser
import os
import unittest
import unittest.mock

import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


class TestStreetHousenumbers(unittest.TestCase):
    """Tests handle_street_housenumbers()."""
    def test_view_result_update_result_link(self) -> None:
        """Tests view result: the update-result link."""
        class MyHTMLParser(html.parser.HTMLParser):
            """Parses the HTML output for /osm/street-housenumbers/gazdagret/view-result."""
            def __init__(self) -> None:
                html.parser.HTMLParser.__init__(self)
                self.in_toolbar = False
                self.current_href = ""
                self.toolbar_overpass_link = ""

            def handle_starttag(self, tag: str, attrs: List[Tuple[str, str]]) -> None:
                if tag == "div":
                    for key, value in attrs:
                        if key == "id" and value == "toolbar":
                            self.in_toolbar = True
                elif tag == "a":
                    for key, value in attrs:
                        if key == "href":
                            self.current_href = value

            def handle_endtag(self, tag: str) -> None:
                if tag == "div":
                    self.in_toolbar = False
                elif tag == "a":
                    self.current_href = ""

            def handle_data(self, data: str) -> None:
                if self.in_toolbar:
                    if "Call Overpass to update" in data:
                        self.toolbar_overpass_link = self.current_href

            def error(self, message: str) -> None:
                raise NotImplementedError()

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
        parser = MyHTMLParser()
        parser.feed(output)
        self.assertTrue(parser.toolbar_overpass_link, "/osm/street-housenumbers/gazdagret/update-result")


if __name__ == '__main__':
    unittest.main()
