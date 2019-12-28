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
import unittest
import unittest.mock

import webframe

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


class TestHandleStatic(unittest.TestCase):
    """Tests handle_static()."""
    def test_happy(self) -> None:
        """Tests the happy path: css case."""
        content, content_type = webframe.handle_static("/osm/static/osm.css")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "text/css")

    def test_javascript(self) -> None:
        """Tests the javascript case."""
        content, content_type = webframe.handle_static("/osm/static/sorttable.js")
        self.assertTrue(len(content))
        self.assertEqual(content_type, "application/x-javascript")

    def test_else(self) -> None:
        """Tests the case when the content type is not recognized."""
        content, content_type = webframe.handle_static("/osm/static/test.xyz")
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
            header_dict = {key: value for (key, value) in response_headers}
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        try:
            int("a")
        # pylint: disable=broad-except
        except Exception:
            callback = cast('StartResponse', start_response)  # type: StartResponse
            output_iterable = webframe.handle_exception(environ, callback)
            output_list = cast(List[bytes], output_iterable)
            self.assertTrue(output_list)
            output = output_list[0].decode('utf-8')
            self.assertIn("ValueError", output)
            return
        self.fail()


if __name__ == '__main__':
    unittest.main()
