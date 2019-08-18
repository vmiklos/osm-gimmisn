#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_validator module covers the validator module."""

from typing import Any
from typing import List
import unittest
import unittest.mock
import validator


def mock_sys_exit(ret: List[int]) -> Any:
    """Mocks sys.exit()."""
    def mock(code: int) -> None:
        ret.append(code)
    return mock


class TestValidatorMain(unittest.TestCase):
    """Tests main()."""
    def test_relations_happy(self) -> None:
        """Tests the happy relations path."""
        argv = ["", "tests/data/relations.yaml"]
        with unittest.mock.patch('sys.argv', argv):
            ret = []  # type: List[int]
            with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                validator.main()
                self.assertEqual(ret, [])

    def test_relations_missing_osmrelation(self) -> None:
        """Tests the missing-osmrelation relations path."""
        # Set up arguments.
        argv = ["", "tests/data/relations-missing-osmrelation.yaml"]
        with unittest.mock.patch('sys.argv', argv):
            # Silence error message.
            with unittest.mock.patch('sys.stdout.write', lambda _value: None):
                # Capture exit code.
                ret = []  # type: List[int]
                with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                    validator.main()
                    self.assertEqual(ret, [1])


if __name__ == '__main__':
    unittest.main()
