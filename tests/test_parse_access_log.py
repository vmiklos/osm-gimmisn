#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_parse_access_log module covers the parse_access_log module."""

import io
import unittest
import unittest.mock

import parse_access_log
import test_config


class TestMain(test_config.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        argv = ["", "tests/mock/access_log"]
        buf = io.StringIO()
        with unittest.mock.patch('sys.argv', argv):
            with unittest.mock.patch('sys.stdout', buf):
                parse_access_log.main()

        buf.seek(0)
        actual = buf.read()
        self.assertIn("data/relation-inactiverelation.yaml: set inactive: false\n", actual)
        self.assertIn("data/relation-gazdagret.yaml: set inactive: true\n", actual)
        self.assertNotIn("data/relation-nosuchrelation.yaml: set inactive: ", actual)


if __name__ == '__main__':
    unittest.main()
