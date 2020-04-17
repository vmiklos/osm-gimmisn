#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_webframe module covers the webframe module."""

from typing import List
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import datetime
import os
import unittest
import unittest.mock
import time

# pylint: disable=unused-import
import yattag

import util
import webframe

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse  # noqa: F401


def get_abspath(path: str) -> str:
    """Mock get_abspath() that uses the test directory."""
    if os.path.isabs(path):
        return path
    return os.path.join(os.path.dirname(__file__), path)


class TestHandleStatic(unittest.TestCase):
    """Tests handle_static()."""
    def test_happy(self) -> None:
        """Tests the happy path: css case."""
        prefix = util.Config.get_uri_prefix()
        content, content_type = webframe.handle_static(prefix + "/static/osm.css")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "text/css")

    def test_javascript(self) -> None:
        """Tests the javascript case."""
        prefix = util.Config.get_uri_prefix()
        content, content_type = webframe.handle_static(prefix + "/static/sorttable.js")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "application/x-javascript")

    def test_json(self) -> None:
        """Tests the json case."""
        prefix = util.Config.get_uri_prefix()
        with unittest.mock.patch('util.get_abspath', get_abspath):
            content, content_type = webframe.handle_static(prefix + "/static/stats.json")
            self.assertEqual(content, "{}\n")
            self.assertEqual(content_type, "application/json")

    def test_else(self) -> None:
        """Tests the case when the content type is not recognized."""
        prefix = util.Config.get_uri_prefix()
        content, content_type = webframe.handle_static(prefix + "/static/test.xyz")
        self.assertFalse(len(content))
        self.assertFalse(len(content_type))


class TestHandleException(unittest.TestCase):
    """Tests handle_exception()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        environ = {
            "PATH_INFO": "/"
        }

        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            self.assertTrue(status.startswith("500"))
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        try:
            int("a")
        # pylint: disable=broad-except
        except Exception:
            callback = cast('StartResponse', start_response)
            output_iterable = webframe.handle_exception(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            self.assertIn("ValueError", output)
            return
        self.fail()


class TestLocalToUiTz(unittest.TestCase):
    """Tests local_to_ui_tz()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('util.get_abspath', get_abspath):
            with util.ConfigContext("timezone", "Europe/Budapest"):
                local_dt = datetime.datetime.fromtimestamp(0)
                ui_dt = webframe.local_to_ui_tz(local_dt)
                if time.strftime('%Z%z') == "CET+0100":
                    self.assertEqual(ui_dt.timestamp(), 0)


class TestFillMissingHeaderItems(unittest.TestCase):
    """Tests fill_missing_header_items()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        streets = "no"
        relation_name = "gazdagret"
        items: List[yattag.doc.Doc] = []
        webframe.fill_missing_header_items(streets, relation_name, items)
        html = items[0].getvalue()
        self.assertIn("Missing house numbers", html)
        self.assertNotIn("Missing streets", html)


if __name__ == '__main__':
    unittest.main()
