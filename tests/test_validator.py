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
            ret: List[int] = []
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
                ret: List[int] = []
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
                ret: List[int] = []
                with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                    validator.main()
                    self.assertEqual(ret, [])
                    buf.seek(0)
                    self.assertEqual(buf.read(), "")


class TestValidatorMainFailureMsgBase(unittest.TestCase):
    """Tests main(), the way it fails."""
    def assert_failure_msg(self, path: str, expected: str) -> None:
        """Asserts that a given input fails with a given error message."""
        # Set up arguments.
        argv = ["", path]
        with unittest.mock.patch('sys.argv', argv):
            # Capture standard output.
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                # Capture exit code.
                ret: List[int] = []
                with unittest.mock.patch('sys.exit', mock_sys_exit(ret)):
                    validator.main()
                    self.assertEqual(ret, [1])
                    buf.seek(0)
                    self.assertEqual(buf.read(), expected)


class TestValidatorMainFailureMsg1(TestValidatorMainFailureMsgBase):
    """First suite of expected error messages."""
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

    def test_relation_refstreets_quote(self) -> None:
        """Tests the relation path: quote in refstreets key or value."""
        expected = "failed to validate tests/data/relation-gazdagret-refstreets-quote.yaml"
        expected += ": expected no quotes in 'refstreets.OSM Name 1''\n"
        expected += "failed to validate tests/data/relation-gazdagret-refstreets-quote.yaml"
        expected += ": expected no quotes in value of 'refstreets.OSM Name 1''\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-refstreets-quote.yaml", expected)

    def test_relation_filters_interpolation_bad(self) -> None:
        """Tests the relation path: bad filters -> interpolation value type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-interpolation-bad.yaml"
        expected += ": expected value type for 'filters.Hamzsabégi út.interpolation' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-interpolation-bad.yaml", expected)

    def test_relation_filters_bad_subkey(self) -> None:
        """Tests the relation path: bad filterssubkey name."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-bad.yaml"
        expected += ": unexpected key 'filters.Budaörsi út.unexpected'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-bad.yaml", expected)

    def test_relation_filters_refsettlement_bad(self) -> None:
        """Tests the relation path: bad filters -> refsettlement value type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-refsettlement-bad.yaml"
        expected += ": expected value type for 'filters.Hamzsabégi út.refsettlement' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-refsettlement-bad.yaml", expected)

    def test_relation_filters_invalid_bad(self) -> None:
        """Tests the relation path: bad filters -> ... -> invalid subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-invalid-bad.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.invalid[0]' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-invalid-bad.yaml", expected)

    def test_relation_filters_invalid_bad_type(self) -> None:
        """Tests the relation path: bad type for the filters -> ... -> invalid subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-invalid-bad-type.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.invalid' is list\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-invalid-bad-type.yaml", expected)

    def test_relation_filters_ranges_bad(self) -> None:
        """Tests the relation path: bad filters -> ... -> ranges subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad.yaml"
        expected += ": unexpected key 'filters.Budaörsi út.ranges[0].unexpected'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-bad.yaml", expected)

    def test_relation_filters_ranges_bad_type(self) -> None:
        """Tests the relation path: bad filters -> ... -> ranges subkey type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-type.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].refsettlement' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-bad-type.yaml", expected)

    def test_relation_filters_ranges_bad_end(self) -> None:
        """Tests the relation path: bad filters -> ... -> ranges -> end type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-end.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].end' is str\n"
        expected += "failed to validate tests/data/relation-gazdagret-filter-range-bad-end.yaml"
        expected += ": expected end >= start for 'filters.Budaörsi út.ranges[0]'\n"
        expected += "failed to validate tests/data/relation-gazdagret-filter-range-bad-end.yaml"
        expected += ": expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-bad-end.yaml", expected)

    def test_relation_filters_ranges_start_end_swap(self) -> None:
        """Tests the relation path: bad filters -> ... -> ranges -> if start/end is swapped type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-start-end-swap.yaml"
        expected += ": expected end >= start for 'filters.Budaörsi út.ranges[0]'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-start-end-swap.yaml", expected)

    def test_relation_filters_ranges_start_end_even_odd(self) -> None:
        """Tests the relation path: bad filters -> ... -> ranges -> if start/end is either both even/odd or not."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-start-end-even-odd.yaml"
        expected += ": expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-start-end-even-odd.yaml", expected)

    def test_relation_filters_ranges_bad_start(self) -> None:
        """Tests the relation path: bad filters -> ... -> ranges -> start type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-start.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].start' is str\n"
        expected += "failed to validate tests/data/relation-gazdagret-filter-range-bad-start.yaml"
        expected += ": expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-bad-start.yaml", expected)

    def test_relation_filters_ranges_missing_start(self) -> None:
        """Tests the relation path: missing filters -> ... -> ranges -> start key."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-missing-start.yaml"
        expected += ": unexpected missing key 'start' for 'filters.Budaörsi út.ranges[0]'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-missing-start.yaml", expected)

    def test_relation_filters_ranges_missing_end(self) -> None:
        """Tests the relation path: missing filters -> ... -> ranges -> end key."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-missing-end.yaml"
        expected += ": unexpected missing key 'end' for 'filters.Budaörsi út.ranges[0]'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-missing-end.yaml", expected)


class TestValidatorMainFailureMsg2(TestValidatorMainFailureMsgBase):
    """Second suite of expected error messages."""
    def test_relation_housenumber_letters_bad(self) -> None:
        """"Tests the housenumber-letters key: bad type."""
        expected = "failed to validate tests/data/relation-gazdagret-housenumber-letters-bad.yaml"
        expected += ": expected value type for 'housenumber-letters' is <class 'bool'>\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-housenumber-letters-bad.yaml", expected)

    def test_relation_alias_bad(self) -> None:
        """Tests the relation path: bad alias subkey."""
        expected = "failed to validate tests/data/relation-budafok-alias-bad.yaml"
        expected += ": expected value type for 'alias[0]' is str\n"
        self.assert_failure_msg("tests/data/relation-budafok-alias-bad.yaml", expected)

    def test_relation_filters_alias_bad_type(self) -> None:
        """Tests the relation path: bad type for the alias subkey."""
        expected = "failed to validate tests/data/relation-budafok-alias-bad-type.yaml"
        expected += ": expected value type for 'alias' is <class 'list'>\n"
        self.assert_failure_msg("tests/data/relation-budafok-alias-bad-type.yaml", expected)


if __name__ == '__main__':
    unittest.main()
