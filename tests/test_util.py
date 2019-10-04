#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_util module covers the util module."""

import os
import unittest

import util


class TestFormatEvenOdd(unittest.TestCase):
    """Tests format_even_odd()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(util.format_even_odd(["1", "2"], html=False), ["1", "2"])

    def test_only_odd(self) -> None:
        """Tests when we have odd numbers only."""
        self.assertEqual(util.format_even_odd(["1", "3"], html=False), ["1, 3"])

    def test_only_even(self) -> None:
        """Tests when we have even numbers only."""
        self.assertEqual(util.format_even_odd(["2", "4"], html=False), ["2, 4"])

    def test_html(self) -> None:
        """Tests HTML coloring."""
        self.assertEqual(util.format_even_odd(["2*", "4"], html=True), ['<span style="color: blue;">2</span>, 4'])


class TestBuildStreetReferenceCache(unittest.TestCase):
    """Tests build_street_reference_cache()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "utcak_20190514.tsv")
        memory_cache = util.build_street_reference_cache(refpath)
        expected = {'01': {'011': ['Törökugrató utca',
                                   'Tűzkő utca',
                                   'Ref Name 1',
                                   'Only In Ref utca',
                                   'Only In Ref Nonsense utca',
                                   'Hamzsabégi út']}}
        self.assertEqual(memory_cache, expected)
        os.unlink(refpath + ".pickle")

    def test_cached(self) -> None:
        """Tests the case when the pickle cache is already available."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "utcak_20190514.tsv")
        util.build_street_reference_cache(refpath)
        memory_cache = util.build_street_reference_cache(refpath)
        expected = {'01': {'011': ['Törökugrató utca',
                                   'Tűzkő utca',
                                   'Ref Name 1',
                                   'Only In Ref utca',
                                   'Only In Ref Nonsense utca',
                                   'Hamzsabégi út']}}
        self.assertEqual(memory_cache, expected)
        os.unlink(refpath + ".pickle")


class TestBuildReferenceCache(unittest.TestCase):
    """Tests build_reference_cache()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        memory_cache = util.build_reference_cache(refpath)
        expected = {'01': {'011': {'Ref Name 1': ['1', '2'],
                                   'Törökugrató utca': ['1', '10', '2', '7'],
                                   'Tűzkő utca': ['1', '10', '2', '9'],
                                   'Hamzsabégi út': ['1']}}}
        self.assertEqual(memory_cache, expected)
        os.unlink(refpath + ".pickle")

    def test_cached(self) -> None:
        """Tests the case when the pickle cache is already available."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        util.build_reference_cache(refpath)
        memory_cache = util.build_reference_cache(refpath)
        expected = {'01': {'011': {'Hamzsabégi út': ['1'],
                                   'Ref Name 1': ['1', '2'],
                                   'Törökugrató utca': ['1', '10', '2', '7'],
                                   'Tűzkő utca': ['1', '10', '2', '9']}}}
        self.assertEqual(memory_cache, expected)
        os.unlink(refpath + ".pickle")


class TestSplitHouseNumber(unittest.TestCase):
    """Tests split_house_number()."""
    def test_only_number(self) -> None:
        """Tests just numbers."""
        self.assertEqual(util.split_house_number('42'), (42, ''))

    def test_number_alpha(self) -> None:
        """Tests numbers and suffixes."""
        self.assertEqual(util.split_house_number('42ab'), (42, 'ab'))

    def test_alpha(self) -> None:
        """Tests just suffixes."""
        self.assertEqual(util.split_house_number('a'), (0, 'a'))


if __name__ == '__main__':
    unittest.main()
