#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_missing_housenumbers module covers the missing_housenumbers module."""

import io
import unittest
import unittest.mock

import test_config

import config
import missing_housenumbers


def mock_make_config() -> config.Config2:
    """Creates a Config instance that has its root as /tests."""
    return config.Config2("tests")


class TestMain(test_config.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        argv = ["", "gh195"]
        buf = io.StringIO()
        with unittest.mock.patch('sys.argv', argv):
            with unittest.mock.patch('sys.stdout', buf):
                with unittest.mock.patch("config.make_config", mock_make_config):
                    missing_housenumbers.main()

        buf.seek(0)
        self.assertEqual(buf.read(), "Kalotaszeg utca\t3\n['25', '27-37', '31*']\n")


if __name__ == '__main__':
    unittest.main()
