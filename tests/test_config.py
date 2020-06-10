#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

import os
import unittest
import unittest.mock


def get_abspath(path: str) -> str:
    """Mock get_abspath() that uses the test directory."""
    if os.path.isabs(path):
        return path
    return os.path.join(os.path.dirname(__file__), path)


class TestCase(unittest.TestCase):
    """Same as unittest.TestCase, but sets up get_abspath to use the test root."""
    def setUp(self) -> None:
        """Sets up the test config."""
        self.get_abspath_patcher = unittest.mock.patch('config.get_abspath', get_abspath)
        self.get_abspath_patcher.start()

    def tearDown(self) -> None:
        """Tears down the test config."""
        self.get_abspath_patcher.stop()


if __name__ == '__main__':
    unittest.main()
