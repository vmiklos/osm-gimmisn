#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_validator module covers the validator module."""

import io
import unittest
import validator


class TestValidatorMainFailureMsgBase(unittest.TestCase):
    """Tests main(), the way it fails."""
    def assert_failure_msg(self, path: str, expected: str) -> None:
        """Asserts that a given input fails with a given error message."""
        argv = ["", path]
        buf = io.BytesIO()
        buf.__setattr__("close", lambda: None)
        ret = validator.main(argv, buf)
        self.assertEqual(ret, 1)
        buf.seek(0)
        self.assertEqual(buf.read(), expected.encode("utf-8"))


class TestValidatorMainFailureMsg1(TestValidatorMainFailureMsgBase):
    """First suite of expected error messages."""
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

    def test_relation_filters_invalid_bad2(self) -> None:
        """Tests the relation path: bad filters -> ... -> invalid subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-invalid-bad2.yaml"
        expected += ": expected format for 'filters.Budaörsi út.invalid[0]' is '42', '42a' or '42/1'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-invalid-bad2.yaml", expected)

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
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].end' is a digit str\n"
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
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].start' is a digit str\n"
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

    def test_relation_filters_show_refstreet_bad(self) -> None:
        """Tests the relation path: bad filters -> show-refstreet value type."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-show-refstreet-bad.yaml"
        expected += ": expected value type for 'filters.Hamzsabégi út.show-refstreet' is bool\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-show-refstreet-bad.yaml", expected)

    def test_relation_refstreets_bad_map_type(self) -> None:
        """Tests the relation path: bad refstreets map, not 1:1."""
        expected = "failed to validate tests/data/relation-gazdagret-refstreets-bad-map.yaml"
        expected += ": osm and ref streets are not a 1:1 mapping in 'refstreets.'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-refstreets-bad-map.yaml", expected)

    def test_relation_filters_valid_bad(self) -> None:
        """Tests the relation path: bad filters -> ... -> valid subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-valid-bad.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.valid[0]' is str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-valid-bad.yaml", expected)

    def test_relation_filters_valid_bad2(self) -> None:
        """Tests the relation path: bad filters -> ... -> valid subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-valid-bad2.yaml"
        expected += ": expected format for 'filters.Budaörsi út.valid[0]' is '42', '42a' or '42/1'\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-valid-bad2.yaml", expected)

    def test_relation_filters_valid_bad_type(self) -> None:
        """Tests the relation path: bad type for the filters -> ... -> valid subkey."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-valid-bad-type.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.valid' is list\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-valid-bad-type.yaml", expected)

    def test_start_whitespace(self) -> None:
        """Tests that we do not accept whitespace in the value of the 'start' key."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-start2.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].start' is a digit str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-bad-start2.yaml", expected)

    def test_end_whitespace(self) -> None:
        """Tests that we do not accept whitespace in the value of the 'end' key."""
        expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-end2.yaml"
        expected += ": expected value type for 'filters.Budaörsi út.ranges[0].end' is a digit str\n"
        self.assert_failure_msg("tests/data/relation-gazdagret-filter-range-bad-end2.yaml", expected)


if __name__ == '__main__':
    unittest.main()
