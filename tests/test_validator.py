#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_validator module covers the validator module."""

import io
import unittest
import validator


class TestValidatorMainFailureMsg(unittest.TestCase):
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
