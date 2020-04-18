#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_missing_streets module covers the missing_streets module."""

import io
import os
import unittest
import unittest.mock

import missing_streets


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        def get_abspath(path: str) -> str:
            if os.path.isabs(path):
                return path
            return os.path.join(os.path.dirname(__file__), path)
        with unittest.mock.patch('config.get_abspath', get_abspath):
            argv = ["", "gazdagret"]
            buf = io.StringIO()
            with unittest.mock.patch('sys.argv', argv):
                with unittest.mock.patch('sys.stdout', buf):
                    missing_streets.main()

            buf.seek(0)
            self.assertEqual(buf.read(), "Only In Ref utca\n")


if __name__ == '__main__':
    unittest.main()
