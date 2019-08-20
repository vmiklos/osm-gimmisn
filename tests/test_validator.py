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

    def assert_failure_msg(self, path: str, expected: str) -> None:
        """Asserts that a given input fails with a given error message."""
        # Set up arguments.
        argv = ["", path]
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
                    self.assertEqual(buf.read(), expected)

    def test_relation_source_bad_type(self) -> None:
        """Tests the relation path: bad source type."""
        expected = "failed to validate tests/data/relation-gazdagret-source-int.yaml"
        expected += ": expected value type for 'source' is <class 'str'>\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-source-int.yaml", expected)

    def test_relation_filters_bad_type(self) -> None:
        """Tests the relation path: bad filters type."""
        expected = "failed to validate tests/data/relation-gazdagret-filters-bad.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.ranges' is list\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filters-bad.yaml", expected)

    def test_relation_bad_key_name(self) -> None:
        """Tests the relation path: bad toplevel key name."""
        expected = "failed to validate tests/data/relation-gazdagret-bad-key.yaml"
        expected += ": unexpected key 'invalid'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-bad-key.yaml", expected)

    def test_relation_strfilters_bad_type(self) -> None:
        """Tests the relation path: bad strfilters value type."""
        expected = "failed to validate tests/data/relation-gazdagret-street-filters-bad.yaml"
        expected += ": expected value type for 'street-filters[0]' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-street-filters-bad.yaml", expected)

    def test_relation_refstreets_bad_value_type(self) -> None:
        """Tests the relation path: bad refstreets value type."""
        expected = "failed to validate tests/data/relation-gazdagret-refstreets-bad-value.yaml"
        expected += ": expected value type for 'refstreets.OSM Name 1' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-refstreets-bad-value.yaml", expected)


if __name__ == '__main__':
    unittest.main()
