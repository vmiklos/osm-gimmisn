#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cherry module covers the cherry module."""

import unittest
import unittest.mock

import cherrypy  # type: ignore

import test_config

import cherry


class TestMain(test_config.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        cherrypy.config.update({'log.screen': False})

        mock_block_called = False

        def mock_block() -> None:
            nonlocal mock_block_called
            mock_block_called = True

        with unittest.mock.patch('cherrypy.engine.block', mock_block):
            cherry.main()
        cherrypy.engine.exit()
        self.assertTrue(mock_block_called)


if __name__ == '__main__':
    unittest.main()
