#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_webframe module covers the webframe module."""

import unittest
import unittest.mock

import webframe


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


if __name__ == '__main__':
    unittest.main()
