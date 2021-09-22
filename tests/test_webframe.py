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
import traceback
import unittest

import test_context

import webframe
import yattag

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse  # noqa: F401


class TestHandleStatic(unittest.TestCase):
    """Tests handle_static()."""
    def test_happy(self) -> None:
        """Tests the happy path: css case."""
        ctx = test_context.make_test_context()
        prefix = ctx.get_ini().get_uri_prefix()
        content, content_type, extra_headers = webframe.handle_static(ctx, prefix + "/static/osm.min.css")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "text/css")
        self.assertEqual(len(extra_headers), 1)
        self.assertEqual(extra_headers[0][0], "Last-Modified")

    def test_generated_javascript(self) -> None:
        """Tests the generated javascript case."""
        ctx = test_context.make_test_context()
        prefix = ctx.get_ini().get_uri_prefix()
        content, content_type, extra_headers = webframe.handle_static(ctx, prefix + "/static/bundle.js")
        self.assertEqual("// bundle.js\n", content.decode("utf-8"))
        self.assertEqual(content_type, "application/x-javascript")
        self.assertEqual(len(extra_headers), 1)
        self.assertEqual(extra_headers[0][0], "Last-Modified")

    def test_json(self) -> None:
        """Tests the json case."""
        ctx = test_context.make_test_context()
        prefix = ctx.get_ini().get_uri_prefix()
        content, content_type, extra_headers = webframe.handle_static(ctx, prefix + "/static/stats-empty.json")
        self.assertTrue(content.decode("utf-8").startswith("{"))
        self.assertEqual(content_type, "application/json")
        self.assertEqual(len(extra_headers), 1)
        self.assertEqual(extra_headers[0][0], "Last-Modified")

    def test_ico(self) -> None:
        """Tests the ico case."""
        ctx = test_context.make_test_context()
        content, content_type, extra_headers = webframe.handle_static(ctx, "/favicon.ico")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "image/x-icon")
        self.assertEqual(len(extra_headers), 1)
        self.assertEqual(extra_headers[0][0], "Last-Modified")

    def test_svg(self) -> None:
        """Tests the svg case."""
        ctx = test_context.make_test_context()
        content, content_type, extra_headers = webframe.handle_static(ctx, "/favicon.svg")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "image/svg+xml")
        self.assertEqual(len(extra_headers), 1)
        self.assertEqual(extra_headers[0][0], "Last-Modified")

    def test_else(self) -> None:
        """Tests the case when the content type is not recognized."""
        ctx = test_context.make_test_context()
        prefix = ctx.get_ini().get_uri_prefix()
        content, content_type, extra_headers = webframe.handle_static(ctx, prefix + "/static/test.xyz")
        self.assertFalse(len(content))
        self.assertFalse(len(content_type))
        # No last modified non-existing file.
        self.assertEqual(len(extra_headers), 0)


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
            output_iterable = webframe.handle_exception(environ, callback, traceback.format_exc())
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            self.assertIn("ValueError", output)
            return
        self.fail()


class TestFillMissingHeaderItems(unittest.TestCase):
    """Tests fill_missing_header_items()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        streets = "no"
        relation_name = "gazdagret"
        items: List[yattag.Doc] = []
        additional_housenumbers = True
        ctx = test_context.make_test_context()
        items = webframe.fill_missing_header_items(ctx, streets, additional_housenumbers, relation_name, items)
        html = items[0].get_value()
        self.assertIn("Missing house numbers", html)
        self.assertNotIn("Missing streets", html)


if __name__ == '__main__':
    unittest.main()
