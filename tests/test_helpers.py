#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import unittest
import os
# pylint: disable=unused-import
from typing import List
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


class TestSortStreetsCsv(unittest.TestCase):
    def test_single_field(self):
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(helpers.sort_streets_csv(unsorted), expected)

    def test_two_fields(self):
        unsorted = 'head\n1\tb\n2\ta'
        expected = 'head\n2\ta\n1\tb'
        self.assertEqual(helpers.sort_streets_csv(unsorted), expected)


class TestSortStreets(unittest.TestCase):
    def test_primary(self):
        unsorted = [
            '0\t\tprimary',
            '1\tPear\tprimary',
            '2\tApple\tsecondary',
            '3\tApple\tprimary',
        ]
        expected = [
            '3\tApple\tprimary',
            '2\tApple\tsecondary',
            '1\tPear\tprimary',
            '0\t\tprimary',
        ]
        self.assertEqual(helpers.sort_streets(unsorted), expected)

    def test_service(self):
        unsorted = [
            '4\tMine\tservice\tdriveway',
            '5\tMine\tservice\tallay',
        ]
        sort = [
            '5\tMine\tservice\tallay',
            '4\tMine\tservice\tdriveway',
        ]
        self.assertEqual(helpers.sort_streets(unsorted), sort)


class TestSortHouseNumbersCsv(unittest.TestCase):
    def test_happy(self):
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(helpers.sort_housenumbers_csv(unsorted), expected)


class TestSortHousenumbers(unittest.TestCase):
    def test_happy(self):
        unsorted = [
            '0\t\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '1\tApple ave\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '2\tPear ave\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '3\tApple ave\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tBase of OpenStreetMap',
            '4\tApple ave\t5\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '5\tApple ave\t\t1234\t\t\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '6\tApple ave\t\t1234\t\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '7\tApple ave\t\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '8\tApple ave\t42\t\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '9\tApple ave\t42\t\tPalace\t1000/11',
        ]
        expected = [
            '9\tApple ave\t42\t\tPalace\t1000/11',
            '8\tApple ave\t42\t\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '5\tApple ave\t\t1234\t\t\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '6\tApple ave\t\t1234\t\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '7\tApple ave\t\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '0\t\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '4\tApple ave\t5\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '3\tApple ave\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tBase of OpenStreetMap',
            '1\tApple ave\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
            '2\tPear ave\t42\t1234\tPalace\t1000/11\t8\t0\t2\tA\tMinistry of OpenStreetMap',
        ]
        self.assertEqual(helpers.sort_housenumbers(unsorted), expected)


class TestSimplify(unittest.TestCase):
    def test_no_space_decode(self):
        original = 'árvíztűrőtükörfúrógép ÁRVÍZTŰRŐTÜKÖRFÚRÓGÉP'
        expected = 'arvizturotukorfurogep_arvizturotukorfurogep'
        self.assertEqual(helpers.simplify(original), expected)

    def test_space_decode(self):
        original = 'árvíztűrőtükörfúrógép ÁRVÍZTŰRŐTÜKÖRFÚRÓGÉP'
        expected = 'arvizturotukorfurogep%20arvizturotukorfurogep'
        self.assertEqual(helpers.simplify(original, spaceDecode=True), expected)


class TestInBoth(unittest.TestCase):
    def test_happy(self):
        self.assertEqual(helpers.get_in_both([1, 2, 3], [2, 3, 4]), [2, 3])


class TestOnlyInFirst(unittest.TestCase):
    def test_happy(self):
        self.assertEqual(helpers.get_only_in_first([1, 2, 3], [3, 4]), [1, 2])


class TestGitLink(unittest.TestCase):
    def test_happy(self):
        actual = helpers.git_link("v1-151-g64ecc85", "http://www.example.com/")
        expected = "<a href=\"http://www.example.com/64ecc85\">v1-151-g64ecc85</a>"
        self.assertEqual(actual, expected)


class TestGetStreets(unittest.TestCase):
    def test_happy(self):
        workdir = os.path.join(os.path.dirname(__file__), "data")
        actual = helpers.get_streets(workdir, "test")
        expected = ['B1', 'B2', 'HB1', 'HB2']
        self.assertEqual(actual, expected)


class TestGetUrlHash(unittest.TestCase):
    def test_happy(self):
        actual = helpers.get_url_hash("http://www.example.com/")
        expected = "14b570acce51451285fa2340e01f97344efe518c8770f5bbc0a794d9bcd55f01"
        self.assertEqual(actual, expected)


if __name__ == '__main__':
    unittest.main()
