#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_get_reference_housenumbers module covers the get_reference_housenumbers module."""

import os
import unittest
import unittest.mock
from typing import Any
import helpers
import get_reference_housenumbers


class ChdirContext:
    """Context manager for os.chdir()."""
    def __init__(self, directory: str) -> None:
        """Remembers what should be the new directory."""
        self.old = os.getcwd()
        self.directory = directory

    def __enter__(self) -> 'ChdirContext':
        """Switches to the new directory."""
        os.chdir(self.directory)
        return self

    def __exit__(self, _exc_type: Any, _exc_value: Any, _exc_traceback: Any) -> bool:
        """Switches back to the old directory."""
        os.chdir(self.old)
        return True


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with ChdirContext("tests"):
            expected = helpers.get_content("workdir/street-housenumbers-reference-gazdagret.lst")
            argv = ["", "gazdagret"]
            with unittest.mock.patch('sys.argv', argv):
                get_reference_housenumbers.main()
            actual = helpers.get_content("workdir/street-housenumbers-reference-gazdagret.lst")
            self.assertEqual(actual, expected)


if __name__ == '__main__':
    unittest.main()
