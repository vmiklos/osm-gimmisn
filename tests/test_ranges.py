#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_ranges module covers the ranges module."""

import unittest

import ranges


class TestRange(unittest.TestCase):
    """Tests Range."""
    def test_isodd_bad(self) -> None:
        """Tests an odd range with an even number."""
        test = ranges.Range(1, 3)
        self.assertFalse(2 in test)

    def test_range_bad(self) -> None:
        """Tests an odd range with a large number."""
        test = ranges.Range(1, 3)
        self.assertFalse(5 in test)

    def test_happy(self) -> None:
        """Tests the happy path."""
        test = ranges.Range(1, 5)
        self.assertTrue(1 in test)
        self.assertTrue(3 in test)
        self.assertTrue(5 in test)
        self.assertEqual(test.get_start(), 1)
        self.assertEqual(test.get_end(), 5)

    def test_eq(self) -> None:
        """Tests equality code."""
        self.assertTrue(ranges.Range(1, 5) != ranges.Range(3, 5))
        self.assertTrue(ranges.Range(1, 5) != ranges.Range(1, 3))
        self.assertTrue(ranges.Range(1, 3) != ranges.Range(1, 3, interpolation="all"))

    def test_interpolation_all(self) -> None:
        """Tests the interpolation modes."""
        self.assertFalse(2 in ranges.Range(1, 3))
        self.assertTrue(2 in ranges.Range(1, 3, interpolation="all"))


class TestRanges(unittest.TestCase):
    """Tests Ranges."""
    def test_a(self) -> None:
        """Tests when the arg is in the first range."""
        test = ranges.Ranges([ranges.Range(0, 0), ranges.Range(1, 1)])
        self.assertTrue(0 in test)

    def test_b(self) -> None:
        """Tests when the arg is in the second range."""
        test = ranges.Ranges([ranges.Range(0, 0), ranges.Range(1, 1)])
        self.assertTrue(1 in test)

    def test_ab(self) -> None:
        """Tests when the arg is in both ranges."""
        test = ranges.Ranges([ranges.Range(1, 1), ranges.Range(1, 1)])
        self.assertTrue(1 in test)

    def test_none(self) -> None:
        """Tests when the arg is in neither ranges."""
        test = ranges.Ranges([ranges.Range(0, 0), ranges.Range(1, 1)])
        self.assertFalse(2 in test)


if __name__ == '__main__':
    unittest.main()
