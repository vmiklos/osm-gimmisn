#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import unittest
import helpers


class TestSortNumerically(unittest.TestCase):
    def test_numers(self):
        ascending = helpers.sort_numerically(['1', '20', '3'])
        self.assertEqual(ascending, ['1', '3', '20'])

    def test_alpha_suffix(self):
        ascending = helpers.sort_numerically(['1a', '20a', '3a'])
        self.assertEqual(ascending, ['1a', '3a', '20a'])

    def test_alpha(self):
        ascending = helpers.sort_numerically(['a', 'c', 'b'])
        self.assertEqual(ascending, ['a', 'b', 'c'])


class TestSplitHouseNumber(unittest.TestCase):
    def test_only_number(self):
        self.assertEqual(helpers.split_house_number('42'), (42, ''))

    def test_number_alpha(self):
        self.assertEqual(helpers.split_house_number('42ab'), (42, 'ab'))

    def test_alpha(self):
        self.assertEqual(helpers.split_house_number('a'), (0, 'a'))


if __name__ == '__main__':
    unittest.main()
