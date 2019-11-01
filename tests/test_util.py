#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_util module covers the util module."""

import os
import unittest
import unittest.mock
import urllib.error

import yattag  # type: ignore

import util


class TestFormatEvenOdd(unittest.TestCase):
    """Tests format_even_odd()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(util.format_even_odd(["1", "2"], doc=None), ["1", "2"])

    def test_only_odd(self) -> None:
        """Tests when we have odd numbers only."""
        self.assertEqual(util.format_even_odd(["1", "3"], doc=None), ["1, 3"])

    def test_only_even(self) -> None:
        """Tests when we have even numbers only."""
        self.assertEqual(util.format_even_odd(["2", "4"], doc=None), ["2, 4"])

    def test_html(self) -> None:
        """Tests HTML coloring."""
        doc = yattag.Doc()
        util.format_even_odd(["2*", "4"], doc)
        self.assertEqual(doc.getvalue(), '<span style="color: blue;">2</span>, 4')

    def test_html_multi_odd(self) -> None:
        """Tests HTML output with multiple odd numbers."""
        doc = yattag.Doc()
        util.format_even_odd(["1", "3"], doc)
        self.assertEqual(doc.getvalue(), "1, 3")


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


class TestParseFilters(unittest.TestCase):
    """Tests parse_filters()."""
    def test_incomplete(self) -> None:
        """Tests the incomplete case."""
        fro = ["osm", "filter-for", "incomplete"]
        self.assertTrue("incomplete" in util.parse_filters(fro))

    def test_refmegye(self) -> None:
        """Tests the refmegye case."""
        fro = ["osm", "filter-for", "refmegye", "42"]
        self.assertEqual(util.parse_filters(fro), {"refmegye": "42"})

    def test_reftelepules(self) -> None:
        """Tests the reftelepules case."""
        fro = ["osm", "filter-for", "refmegye", "42", "reftelepules", "43"]
        filters = util.parse_filters(fro)
        self.assertEqual(filters["refmegye"], "42")
        filters = util.parse_filters(fro)
        self.assertEqual(filters["reftelepules"], "43")


class TestHandleOverpassError(unittest.TestCase):
    """Tests handle_overpass_error()."""
    def test_no_sleep(self) -> None:
        """Tests the case when no sleep is needed."""
        def need_sleep() -> int:
            return 0
        error = urllib.error.HTTPError("http://example.com", 404, "no such file", {}, None)
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', need_sleep):
            doc = util.handle_overpass_error(error)
            self.assertEqual(doc.getvalue(), "Overpass error: HTTP Error 404: no such file")

    def test_need_sleep(self) -> None:
        """Tests the case when sleep is needed."""
        def need_sleep() -> int:
            return 42
        error = urllib.error.HTTPError("http://example.com", 404, "no such file", {}, None)
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', need_sleep):
            doc = util.handle_overpass_error(error)
            expected = "Overpass error: HTTP Error 404: no such file<br />Note: wait for 42 seconds"
            self.assertEqual(doc.getvalue(), expected)


class TestSetupLocalization(unittest.TestCase):
    """Tests setup_localization()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        def set_language(language: str) -> None:
            self.assertEqual(language, "en")
        environ = {"HTTP_ACCEPT_LANGUAGE": "en-US,el;q=0.8"}
        with unittest.mock.patch('i18n.set_language', set_language):
            util.setup_localization(environ)

    def test_parse_error(self) -> None:
        """Tests the error path."""
        def set_language(_language: str) -> None:
            self.fail("unexpected call")
        environ = {"HTTP_ACCEPT_LANGUAGE": ","}
        with unittest.mock.patch('i18n.set_language', set_language):
            util.setup_localization(environ)


if __name__ == '__main__':
    unittest.main()
