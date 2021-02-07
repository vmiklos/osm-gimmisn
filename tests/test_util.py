#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_util module covers the util module."""

from typing import List
import io
import os
import unittest
import unittest.mock
import urllib.error

import yattag

import config
import util


def hnr_list(ranges: List[str]) -> List[util.HouseNumberRange]:
    """Converts a string list into a house number range list."""
    return [util.HouseNumberRange(i, "") for i in ranges]


class TestFormatEvenOdd(unittest.TestCase):
    """Tests format_even_odd()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(util.format_even_odd(hnr_list(["1", "2"]), doc=None), ["1", "2"])

    def test_only_odd(self) -> None:
        """Tests when we have odd numbers only."""
        self.assertEqual(util.format_even_odd(hnr_list(["1", "3"]), doc=None), ["1, 3"])

    def test_only_even(self) -> None:
        """Tests when we have even numbers only."""
        self.assertEqual(util.format_even_odd(hnr_list(["2", "4"]), doc=None), ["2, 4"])

    def test_html(self) -> None:
        """Tests HTML coloring."""
        doc = yattag.doc.Doc()
        util.format_even_odd(hnr_list(["2*", "4"]), doc)
        self.assertEqual(doc.getvalue(), '<span style="color: blue;">2</span>, 4')

    def test_html_comment(self) -> None:
        """Tests HTML commenting."""
        doc = yattag.doc.Doc()
        house_numbers = [
            util.HouseNumberRange("2*", "foo"),
            util.HouseNumberRange("4", ""),
        ]
        util.format_even_odd(house_numbers, doc)
        self.assertEqual(doc.getvalue(), '<span style="color: blue;"><abbr title="foo" tabindex="0">2</abbr></span>, 4')

    def test_html_multi_odd(self) -> None:
        """Tests HTML output with multiple odd numbers."""
        doc = yattag.doc.Doc()
        util.format_even_odd(hnr_list(["1", "3"]), doc)
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
        memory_cache = util.build_reference_cache(refpath, "01")
        expected = {'01': {'011': {'Ref Name 1': hnr_list(['1', '2']),
                                   'Törökugrató utca': hnr_list(['1', '10', '11', '12', '2', '7']),
                                   'Tűzkő utca': hnr_list(['1', '10', '2', '9']),
                                   'Hamzsabégi út': hnr_list(['1'])}}}
        self.assertEqual(memory_cache, expected)
        os.unlink(util.get_reference_cache_path(refpath, "01"))

    def test_cached(self) -> None:
        """Tests the case when the pickle cache is already available."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        util.build_reference_cache(refpath, "01")
        memory_cache = util.build_reference_cache(refpath, "01")
        expected = {'01': {'011': {'Hamzsabégi út': hnr_list(['1']),
                                   'Ref Name 1': hnr_list(['1', '2']),
                                   'Törökugrató utca': hnr_list(['1', '10', '11', '12', '2', '7']),
                                   'Tűzkő utca': hnr_list(['1', '10', '2', '9'])}}}
        self.assertEqual(memory_cache, expected)
        os.unlink(util.get_reference_cache_path(refpath, "01"))


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

    def test_refcounty(self) -> None:
        """Tests the refcounty case."""
        fro = ["osm", "filter-for", "refcounty", "42"]
        self.assertEqual(util.parse_filters(fro), {"refcounty": "42"})

    def test_refsettlement(self) -> None:
        """Tests the refsettlement case."""
        fro = ["osm", "filter-for", "refcounty", "42", "refsettlement", "43"]
        filters = util.parse_filters(fro)
        self.assertEqual(filters["refcounty"], "42")
        filters = util.parse_filters(fro)
        self.assertEqual(filters["refsettlement"], "43")


class TestHandleOverpassError(unittest.TestCase):
    """Tests handle_overpass_error()."""
    def test_no_sleep(self) -> None:
        """Tests the case when no sleep is needed."""
        def need_sleep() -> int:
            return 0
        error = urllib.error.HTTPError("http://example.com", 404, "no such file", {}, io.BytesIO())
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', need_sleep):
            doc = util.handle_overpass_error(error)
            expected = """<div id="overpass-error">Overpass error: HTTP Error 404: no such file</div>"""
            self.assertEqual(doc.getvalue(), expected)

    def test_need_sleep(self) -> None:
        """Tests the case when sleep is needed."""
        def need_sleep() -> int:
            return 42
        error = urllib.error.HTTPError("http://example.com", 404, "no such file", {}, io.BytesIO())
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', need_sleep):
            doc = util.handle_overpass_error(error)
            expected = """<div id="overpass-error">Overpass error: HTTP Error 404: no such file"""
            expected += """<br />Note: wait for 42 seconds</div>"""
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
        sock = util.CsvIO(io.StringIO("h1\th2\n\nv1\tv2\n"))
        ret = util.tsv_to_list(sock)
        self.assertEqual(len(ret), 2)
        row1 = [cell.getvalue() for cell in ret[0]]
        self.assertEqual(row1, ['h1', 'h2'])
        row2 = [cell.getvalue() for cell in ret[1]]
        self.assertEqual(row2, ['v1', 'v2'])

    def test_type(self) -> None:
        """Tests when a @type column is available."""
        stream = util.CsvIO(io.StringIO("@id\t@type\n42\tnode\n"))
        ret = util.tsv_to_list(stream)
        self.assertEqual(len(ret), 2)
        row1 = [cell.getvalue() for cell in ret[0]]
        self.assertEqual(row1, ["@id", "@type"])
        row2 = [cell.getvalue() for cell in ret[1]]
        cell_a2 = '<a href="https://www.openstreetmap.org/node/42" target="_blank">42</a>'
        self.assertEqual(row2, [cell_a2, "node"])

    def test_escape(self) -> None:
        """Tests escaping."""
        sock = util.CsvIO(io.StringIO("\"h,1\"\th2\n"))
        ret = util.tsv_to_list(sock)
        self.assertEqual(len(ret), 1)
        row1 = [cell.getvalue() for cell in ret[0]]
        # Note how this is just h,1 and not "h,1".
        self.assertEqual(row1, ['h,1', 'h2'])


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
        self.assertTrue(util.HouseNumber.is_invalid("67/5*", ["67/5"]))

        # Make sure we don't throw an exception on input which does not start with a number.
        self.assertFalse(util.HouseNumber.is_invalid("A", ["15a"]))

    def test_has_letter_suffix(self) -> None:
        """Tests has_letter_suffix()."""
        self.assertTrue(util.HouseNumber.has_letter_suffix("42a", ""))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42 a", ""))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42/a", ""))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42/a*", "*"))
        self.assertTrue(util.HouseNumber.has_letter_suffix("42A", ""))
        self.assertFalse(util.HouseNumber.has_letter_suffix("42 AB", ""))

    def test_normalize_letter_suffix(self) -> None:
        """Tests normalize_letter_suffix()."""
        normalize = util.HouseNumber.normalize_letter_suffix
        self.assertEqual(normalize("42a", "", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42 a", "", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42/a", "", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42/A", "", util.LetterSuffixStyle.UPPER), "42/A")
        self.assertEqual(normalize("42/A*", "*", util.LetterSuffixStyle.UPPER), "42/A*")
        self.assertEqual(normalize("42 A", "", util.LetterSuffixStyle.UPPER), "42/A")
        with self.assertRaises(ValueError):
            util.HouseNumber.normalize_letter_suffix("x", "", util.LetterSuffixStyle.UPPER)
        self.assertEqual(normalize("42/A", "", util.LetterSuffixStyle.LOWER), "42a")


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
        range_names = [i.get_number() for i in ranges]
        self.assertEqual(range_names, ["25", "27-37", "31*"])


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
        actual = config.get_abspath("foo")
        expected = os.path.join(os.getcwd(), "foo")
        self.assertEqual(actual, expected)
        self.assertEqual(config.get_abspath(actual), expected)


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


class TestOnlyInFirst(unittest.TestCase):
    """Tests get_only_in_first()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(util.get_only_in_first(["1", "2", "3"], ["3", "4"]), ["1", "2"])


class TestInBoth(unittest.TestCase):
    """Tests get_in_both()."""
    def test_happy(self) -> None:
        """Tests that happy path."""
        self.assertEqual(util.get_in_both(["1", "2", "3"], ["2", "3", "4"]), ["2", "3"])


class TestGetWorkdir(unittest.TestCase):
    """Tests get_workdir()."""
    def test_happy(self) -> None:
        """Tests the happy path."""

        with config.ConfigContext("workdir", "/path/to/workdir"):
            actual = config.Config.get_workdir()
            expected = "/path/to/workdir"
            self.assertEqual(actual, expected)


class TestGetContent(unittest.TestCase):
    """Tests get_content()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = util.get_content(workdir, "gazdagret.percent")
        expected = "54.55"
        self.assertEqual(actual, expected)

    def test_one_arg(self) -> None:
        """Tests the case when only one argument is given."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = util.get_content(os.path.join(workdir, "gazdagret.percent"))
        expected = "54.55"
        self.assertEqual(actual, expected)


class TestStreet(unittest.TestCase):
    """Tests Street."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        street = util.Street("foo", "bar")
        self.assertEqual(street.get_ref_name(), "bar")
        self.assertEqual(street.to_html().getvalue(), "foo<br />(bar)")


class TestGetCityKey(unittest.TestCase):
    """Tests get_city_key()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        valid_settlements = set(["lábatlan"])
        self.assertEqual(util.get_city_key("1234", "Budapest", valid_settlements), "budapest_23")
        self.assertEqual(util.get_city_key("1889", "Budapest", valid_settlements), "budapest")
        self.assertEqual(util.get_city_key("9999", "", valid_settlements), "_Empty")
        self.assertEqual(util.get_city_key("9999", "Lábatlan", valid_settlements), "lábatlan")
        self.assertEqual(util.get_city_key("9999", "junk", valid_settlements), "_Invalid")
        # Even if the postcode does not start with 1.
        self.assertEqual(util.get_city_key("9999", "Budapest", valid_settlements), "budapest")


class TestGetStreetFromHousenumber(unittest.TestCase):
    """Tests get_street_from_housenumber()."""
    def test_addr_place(self) -> None:
        """Tests the case when addr:place is used."""
        with util.CsvIO(open("tests/workdir/street-housenumbers-gh964.csv", "r")) as stream:
            actual = util.get_street_from_housenumber(stream)
        # This is picked up from addr:place because addr:street was empty.
        self.assertEqual(actual, [util.Street(osm_name="Tolvajos tanya")])


if __name__ == '__main__':
    unittest.main()
