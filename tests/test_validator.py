#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_validator module covers the validator module."""

import io
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
        argv = ["", "tests/data/relations-missing-osmrelation/relations.yaml"]
        with unittest.mock.patch('sys.argv', argv):
            # Capture standard output.
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                # Capture exit code.
                ret = []  # type: List[int]
                with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                    validator.main()
                    self.assertEqual(ret, [1])
                    buf.seek(0)
                    expected = "failed to validate tests/data/relations-missing-osmrelation/relations.yaml"
                    expected += ": missing key 'gazdagret.osmrelation'\n"
                    self.assertEqual(buf.read(), expected)

    def test_relation_happy(self) -> None:
        """Tests the happy relation path."""
        # Set up arguments.
        argv = ["", "tests/data/relation-gazdagret.yaml"]
        with unittest.mock.patch('sys.argv', argv):
            # Capture standard output.
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                # Capture exit code.
                ret = []  # type: List[int]
                with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                    validator.main()
                    self.assertEqual(ret, [])
                    buf.seek(0)
                    self.assertEqual(buf.read(), "")

    def test_relation_source_bad_type(self) -> None:
        """Tests the relation path: bad source type."""
        # Set up arguments.
        argv = ["", "tests/data/relation-gazdagret-source-int.yaml"]
        with unittest.mock.patch('sys.argv', argv):
            # Capture standard output.
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                # Capture exit code.
                ret = []  # type: List[int]
                with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                    validator.main()
                    self.assertEqual(ret, [1])
                    buf.seek(0)
                    expected = "failed to validate tests/data/relation-gazdagret-source-int.yaml"
                    expected += ": expected value type for 'source' is <class 'str'>\n"
                    self.assertEqual(buf.read(), expected)


if __name__ == '__main__':
    unittest.main()
