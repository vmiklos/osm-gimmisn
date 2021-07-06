#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_ranges module covers the ranges module."""

import unittest

import ranges


def make_range(start: int, end: int) -> ranges.Range:
    """Factory for Range without specifying interpolation."""
    return ranges.Range(start, end, interpolation="")


class TestRange(unittest.TestCase):
    """Tests Range."""
    def test_isodd_bad(self) -> None:
        """Tests an odd range with an even number."""
        test = make_range(1, 3)
        self.assertFalse(2 in test)

    def test_range_bad(self) -> None:
        """Tests an odd range with a large number."""
        test = make_range(1, 3)
        self.assertFalse(5 in test)

    def test_happy(self) -> None:
        """Tests the happy path."""
        test = make_range(1, 5)
        self.assertTrue(1 in test)
        self.assertTrue(3 in test)
        self.assertTrue(5 in test)
        self.assertEqual(test.get_start(), 1)
        self.assertEqual(test.get_end(), 5)

    def test_eq(self) -> None:
        """Tests equality code."""
        self.assertTrue(make_range(1, 5) != make_range(3, 5))
        self.assertTrue(make_range(1, 5) != make_range(1, 3))
        self.assertTrue(make_range(1, 3) != ranges.Range(1, 3, interpolation="all"))

    def test_interpolation_all(self) -> None:
        """Tests the interpolation modes."""
        self.assertFalse(2 in make_range(1, 3))
        self.assertTrue(2 in ranges.Range(1, 3, interpolation="all"))


class TestRanges(unittest.TestCase):
    """Tests Ranges."""
    def test_a(self) -> None:
        """Tests when the arg is in the first range."""
        test = ranges.Ranges([make_range(0, 0), make_range(1, 1)])
        self.assertTrue(0 in test)

    def test_b(self) -> None:
        """Tests when the arg is in the second range."""
        test = ranges.Ranges([make_range(0, 0), make_range(1, 1)])
        self.assertTrue(1 in test)

    def test_ab(self) -> None:
        """Tests when the arg is in both ranges."""
        test = ranges.Ranges([make_range(1, 1), make_range(1, 1)])
        self.assertTrue(1 in test)

    def test_none(self) -> None:
        """Tests when the arg is in neither ranges."""
        test = ranges.Ranges([make_range(0, 0), make_range(1, 1)])
        self.assertFalse(2 in test)


if __name__ == '__main__':
    unittest.main()
