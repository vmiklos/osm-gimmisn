#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_util module covers the util module."""

import io
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
                                   'Törökugrató utca': ['1', '10', '11', '12', '2', '7'],
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
                                   'Törökugrató utca': ['1', '10', '11', '12', '2', '7'],
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


class TestGenLink(unittest.TestCase):
    """Tests gen_link()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        doc = util.gen_link("http://www.example.com", "label")
        expected = '<a href="http://www.example.com">label...</a>'
        expected += '<script type="text/javascript">window.location.href = "http://www.example.com";</script>'
        self.assertEqual(doc.getvalue(), expected)


class TestProcessTemplate(unittest.TestCase):
    """Tests process_template()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        template = "aaa @RELATION@ bbb @AREA@ ccc"
        expected = "aaa 42 bbb 3600000042 ccc"
        actual = util.process_template(template, 42)
        self.assertEqual(actual, expected)


class TestHtmlTableFromList(unittest.TestCase):
    """Tests html_table_from_list()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        fro = [[util.html_escape("A1"),
                util.html_escape("B1")],
               [util.html_escape("A2"),
                util.html_escape("B2")]]
        expected = '<table class="sortable">'
        expected += '<tr><th><a href="#">A1</a></th>'
        expected += '<th><a href="#">B1</a></th></tr>'
        expected += '<tr><td>A2</td><td>B2</td></tr></table>'
        ret = util.html_table_from_list(fro).getvalue()
        self.assertEqual(ret, expected)


class TestTsvToList(unittest.TestCase):
    """Tests tsv_to_list()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        sock = io.StringIO("h1\th2\n\nv1\tv2\n")
        ret = util.tsv_to_list(sock)
        self.assertEqual(len(ret), 2)
        row1 = [cell.getvalue() for cell in ret[0]]
        self.assertEqual(row1, ['h1', 'h2'])
        row2 = [cell.getvalue() for cell in ret[1]]
        self.assertEqual(row2, ['v1', 'v2'])

    def test_type(self) -> None:
        """Tests when a @type column is available."""
        stream = io.StringIO("@id\t@type\n42\tnode\n")
        ret = util.tsv_to_list(stream)
        self.assertEqual(len(ret), 2)
        row1 = [cell.getvalue() for cell in ret[0]]
        self.assertEqual(row1, ["@id", "@type"])
        row2 = [cell.getvalue() for cell in ret[1]]
        cell_a2 = '<a href="https://www.openstreetmap.org/node/42" target="_blank">42</a>'
        self.assertEqual(row2, [cell_a2, "node"])


class TestHouseNumber(unittest.TestCase):
    """Tests the HouseNumber class."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        house_number = util.HouseNumber("1", "1-2")
        self.assertEqual(house_number.get_number(), "1")
        self.assertEqual(house_number.get_source(), "1-2")
        self.assertTrue(util.HouseNumber("1", "1-2") != util.HouseNumber("2", "1-2"))
        self.assertEqual(len(set([util.HouseNumber("1", "1-2"),
                                  util.HouseNumber("2", "1-2"),
                                  util.HouseNumber("2", "1-2")])), 2)

    def test_is_invalid(self) -> None:
        """Tests is_invalid()."""
        self.assertTrue(util.HouseNumber.is_invalid("15 a", ["15a"]))
        self.assertTrue(util.HouseNumber.is_invalid("15/a", ["15a"]))
        self.assertTrue(util.HouseNumber.is_invalid("15A", ["15a"]))

        # Make sure we don't throw an exception on input which does not start with a number.
        self.assertFalse(util.HouseNumber.is_invalid("A", ["15a"]))

    def test_has_letter_suffix(self) -> None:
        """Tests has_letter_suffix()."""
        self.assertTrue(util.HouseNumber.has_letter_suffix("42a"))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42 a"))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42/a"))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42A"))
        self.assertFalse(util.HouseNumber.has_letter_suffix("42 AB"))

    def test_normalize_letter_suffix(self) -> None:
        """Tests normalize_letter_suffix()."""
        normalize = util.HouseNumber.normalize_letter_suffix
        self.assertEqual(normalize("42a", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42 a", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42/a", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42/A", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42 A", util.LetterSuffixStyle.UPPER), "42/A")
        with self.assertRaises(ValueError):
            util.HouseNumber.normalize_letter_suffix("x", util.LetterSuffixStyle.UPPER)
        self.assertEqual(normalize("42/A", util.LetterSuffixStyle.LOWER), "42a")


class TestGetHousenumberRanges(unittest.TestCase):
    """Tests get_housenumber_ranges()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        house_numbers = [
            util.HouseNumber("25", "25"),
            util.HouseNumber("27", "27-37"),
            util.HouseNumber("29", "27-37"),
            util.HouseNumber("31", "27-37"),
            util.HouseNumber("33", "27-37"),
            util.HouseNumber("35", "27-37"),
            util.HouseNumber("37", "27-37"),
            util.HouseNumber("31*", "31*"),
        ]
        ranges = util.get_housenumber_ranges(house_numbers)
        self.assertEqual(ranges, ["25", "27-37", "31*"])


class TestGitLink(unittest.TestCase):
    """Tests git_link()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        actual = util.git_link("v1-151-g64ecc85", "http://www.example.com/").getvalue()
        expected = "<a href=\"http://www.example.com/64ecc85\">v1-151-g64ecc85</a>"
        self.assertEqual(actual, expected)


class TestGetAbspath(unittest.TestCase):
    """Tests get_abspath()."""
    def test_happy(self) -> None:
        """Tests the happy path, when the input is relative."""
        actual = util.get_abspath("foo")
        expected = os.path.join(os.getcwd(), "foo")
        self.assertEqual(actual, expected)


class TestSortNumerically(unittest.TestCase):
    """Tests sort_numerically()."""
    def test_numbers(self) -> None:
        """Tests numbers."""
        ascending = util.sort_numerically([util.HouseNumber('1', ''),
                                           util.HouseNumber('20', ''),
                                           util.HouseNumber('3', '')])
        self.assertEqual([i.get_number() for i in ascending], ['1', '3', '20'])

    def test_alpha_suffix(self) -> None:
        """Tests numbers with suffixes."""
        ascending = util.sort_numerically([util.HouseNumber('1a', ''),
                                           util.HouseNumber('20a', ''),
                                           util.HouseNumber('3a', '')])
        self.assertEqual([i.get_number() for i in ascending], ['1a', '3a', '20a'])

    def test_alpha(self) -> None:
        """Tests just suffixes."""
        ascending = util.sort_numerically([util.HouseNumber('a', ''),
                                           util.HouseNumber('c', ''),
                                           util.HouseNumber('b', '')])
        self.assertEqual([i.get_number() for i in ascending], ['a', 'b', 'c'])


class TestSortStreets(unittest.TestCase):
    """Tests sort_streets()."""
    def test_primary(self) -> None:
        """Tests that missing 2nd col is ordered last."""
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
        self.assertEqual(util.sort_streets(unsorted), expected)

    def test_service(self) -> None:
        """Tests that matching 2nd and 3rd col means ordering by 4th col."""
        unsorted = [
            '4\tMine\tservice\tdriveway',
            '5\tMine\tservice\tallay',
        ]
        sort = [
            '5\tMine\tservice\tallay',
            '4\tMine\tservice\tdriveway',
        ]
        self.assertEqual(util.sort_streets(unsorted), sort)


class TestSortStreetsCsv(unittest.TestCase):
    """Tests sort_streets_csv()."""
    def test_single_field(self) -> None:
        """Tests a single column."""
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(util.sort_streets_csv(unsorted), expected)

    def test_two_fields(self) -> None:
        """Tests 2 columns."""
        unsorted = 'head\n1\tb\n2\ta'
        expected = 'head\n2\ta\n1\tb'
        self.assertEqual(util.sort_streets_csv(unsorted), expected)


class TestSortHousenumbers(unittest.TestCase):
    """Tests sort_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
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
        self.assertEqual(util.sort_housenumbers(unsorted), expected)


class TestSortHouseNumbersCsv(unittest.TestCase):
    """Tests sort_housenumbers_csv()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(util.sort_housenumbers_csv(unsorted), expected)


if __name__ == '__main__':
    unittest.main()
