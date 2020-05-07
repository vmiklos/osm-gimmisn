#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cherry module covers the cherry module."""

import os
import unittest
import unittest.mock

import cherrypy  # type: ignore

import cherry


def get_abspath(path: str) -> str:
    """Mock get_abspath() that uses the test directory."""
    if os.path.isabs(path):
        return path
    return os.path.join(os.path.dirname(__file__), path)


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        cherrypy.config.update({'log.screen': False})

        mock_block_called = False

        def mock_block() -> None:
            nonlocal mock_block_called
            mock_block_called = True

        with unittest.mock.patch('config.get_abspath', get_abspath):
            with unittest.mock.patch('cherrypy.engine.block', mock_block):
                cherry.main()
        cherrypy.engine.exit()
        self.assertTrue(mock_block_called)


if __name__ == '__main__':
    unittest.main()
