#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_helpers module covers the helpers module."""

import configparser
import io
import os
import unittest
# pylint: disable=unused-import
from typing import List

import helpers


class TestSortNumerically(unittest.TestCase):
    """Tests sort_numerically()."""
    def test_numbers(self):
        """Tests numbers."""
        ascending = helpers.sort_numerically(['1', '20', '3'])
        self.assertEqual(ascending, ['1', '3', '20'])

    def test_alpha_suffix(self):
        """Tests numbers with suffixes."""
        ascending = helpers.sort_numerically(['1a', '20a', '3a'])
        self.assertEqual(ascending, ['1a', '3a', '20a'])

    def test_alpha(self):
        """Tests just suffixes."""
        ascending = helpers.sort_numerically(['a', 'c', 'b'])
        self.assertEqual(ascending, ['a', 'b', 'c'])


class TestSplitHouseNumber(unittest.TestCase):
    """Tests split_house_number()."""
    def test_only_number(self):
        """Tests just numbers."""
        self.assertEqual(helpers.split_house_number('42'), (42, ''))

    def test_number_alpha(self):
        """Tests numbers and suffixes."""
        self.assertEqual(helpers.split_house_number('42ab'), (42, 'ab'))

    def test_alpha(self):
        """Tests just suffixes."""
        self.assertEqual(helpers.split_house_number('a'), (0, 'a'))


class TestSortStreetsCsv(unittest.TestCase):
    """Tests sort_streets_csv()."""
    def test_single_field(self):
        """Tests a single column."""
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(helpers.sort_streets_csv(unsorted), expected)

    def test_two_fields(self):
        """Tests 2 columns."""
        unsorted = 'head\n1\tb\n2\ta'
        expected = 'head\n2\ta\n1\tb'
        self.assertEqual(helpers.sort_streets_csv(unsorted), expected)


class TestSortStreets(unittest.TestCase):
    """Tests sort_streets()."""
    def test_primary(self):
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
        self.assertEqual(helpers.sort_streets(unsorted), expected)

    def test_service(self):
        """Tests that matching 2nd and 3rd col means ordering by 4th col."""
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
    """Tests sort_housenumbers_csv()."""
    def test_happy(self):
        """Tests the happy path."""
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(helpers.sort_housenumbers_csv(unsorted), expected)


class TestSortHousenumbers(unittest.TestCase):
    """Tests sort_housenumbers()."""
    def test_happy(self):
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
        self.assertEqual(helpers.sort_housenumbers(unsorted), expected)


class TestInBoth(unittest.TestCase):
    """Tests get_in_both()."""
    def test_happy(self):
        """Tests that happy path."""
        self.assertEqual(helpers.get_in_both([1, 2, 3], [2, 3, 4]), [2, 3])


class TestOnlyInFirst(unittest.TestCase):
    """Tests get_only_in_first()."""
    def test_happy(self):
        """Tests the happy path."""
        self.assertEqual(helpers.get_only_in_first([1, 2, 3], [3, 4]), [1, 2])


class TestGitLink(unittest.TestCase):
    """Tests git_link()."""
    def test_happy(self):
        """Tests the happy path."""
        actual = helpers.git_link("v1-151-g64ecc85", "http://www.example.com/")
        expected = "<a href=\"http://www.example.com/64ecc85\">v1-151-g64ecc85</a>"
        self.assertEqual(actual, expected)


class TestGetStreets(unittest.TestCase):
    """Tests get_streets()."""
    def test_happy(self):
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "data")
        actual = helpers.get_streets(workdir, "test")
        expected = ['B1', 'B2', 'HB1', 'HB2']
        self.assertEqual(actual, expected)


class TestRange(unittest.TestCase):
    """Tests Range."""
    def test_isodd_bad(self):
        """Tests an odd range with an even number."""
        test = helpers.Range(1, 3)
        self.assertFalse(2 in test)

    def test_range_bad(self):
        """Tests an odd range with a large number."""
        test = helpers.Range(1, 3)
        self.assertFalse(5 in test)

    def test_happy(self):
        """Tests the happy path."""
        test = helpers.Range(1, 5)
        self.assertTrue(1 in test)
        self.assertTrue(3 in test)
        self.assertTrue(5 in test)

    def test_eq(self):
        """Tests equality code."""
        self.assertTrue(helpers.Range(1, 5) != helpers.Range(3, 5))
        self.assertTrue(helpers.Range(1, 5) != helpers.Range(1, 3))


class TestRanges(unittest.TestCase):
    """Tests Ranges."""
    def test_a(self):
        """Tests when the arg is in the first range."""
        test = helpers.Ranges([[0], [1]])
        self.assertTrue(0 in test)

    def test_b(self):
        """Tests when the arg is in the second range."""
        test = helpers.Ranges([[0], [1]])
        self.assertTrue(1 in test)

    def test_ab(self):
        """Tests when the arg is in both ranges."""
        test = helpers.Ranges([[1], [1]])
        self.assertTrue(1 in test)

    def test_none(self):
        """Tests when the arg is in neither ranges."""
        test = helpers.Ranges([[0], [1]])
        self.assertFalse(2 in test)


class TestGetWorkdir(unittest.TestCase):
    """Tests get_workdir()."""
    def test_happy(self):
        """Tests the happy path."""
        config = configparser.ConfigParser()
        config.read_dict({"wsgi": {"workdir": "/path/to/workdir"}})
        actual = helpers.get_workdir(config)
        expected = "/path/to/workdir"
        self.assertEqual(actual, expected)


class TestProcessTemplate(unittest.TestCase):
    """Tests process_template()."""
    def test_happy(self):
        """Tests the happy path."""
        template = "aaa @RELATION@ bbb @AREA@ ccc"
        expected = "aaa 42 bbb 3600000042 ccc"
        actual = helpers.process_template(template, 42)
        self.assertEqual(actual, expected)


class TestGetContent(unittest.TestCase):
    """Tests get_content()."""
    def test_happy(self):
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "data")
        actual = helpers.get_content(workdir, "gazdagret.percent")
        expected = "99.44"
        self.assertEqual(actual, expected)


class TestLoadNormalizers(unittest.TestCase):
    """Tests load_normalizers()."""
    def test_happy(self):
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        filters, ref_streets = helpers.load_normalizers(datadir, "gazdagret")
        expected_filters = {
            "Budaörsi út": helpers.Ranges([helpers.Range(137, 165)]),
            "Csiki-hegyek utca": helpers.Ranges([helpers.Range(1, 15), helpers.Range(2, 26)]),
        }
        self.assertEqual(filters, expected_filters)
        expected_streets = {
            'OSM Name 1': 'Ref Name 1',
            'OSM Name 2': 'Ref Name 2'
        }
        self.assertEqual(ref_streets, expected_streets)

    def test_nosuchname(self):
        """Tests when there is no filters file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        filters, ref_streets = helpers.load_normalizers(datadir, "nosuchname")
        self.assertEqual(filters, {})
        self.assertEqual(ref_streets, {})

    def test_empty(self):
        """Tests when the filter file is empty."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        filters, ref_streets = helpers.load_normalizers(datadir, "empty")
        self.assertEqual(filters, {})
        self.assertEqual(ref_streets, {})


class TestGetStreetDetails(unittest.TestCase):
    """Tests get_street_details()."""
    def test_happy(self):
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "Budaörsi út"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual("011", reftelepules)
        self.assertEqual("Budaörsi", street_name)
        self.assertEqual("út", street_type)

    def test_reftelepules_override(self):
        """Tests street-specific reftelepules override."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "Teszt utca"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual("012", reftelepules)
        self.assertEqual("Teszt", street_name)
        self.assertEqual("utca", street_type)

    def test_refstreets(self):
        """Tests OSM -> ref name mapping."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "OSM Name 1"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual("011", reftelepules)
        self.assertEqual("Ref Name", street_name)
        self.assertEqual("1", street_type)

    def test_nosuchrelation(self):
        """Tests a relation without a filter file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "OSM Name 1"
        relation_name = "nosuchrelation"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual("011", reftelepules)
        self.assertEqual("OSM Name", street_name)
        self.assertEqual("1", street_type)

    def test_emptyrelation(self):
        """Tests a relation with an empty filter file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "OSM Name 1"
        relation_name = "empty"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual("011", reftelepules)
        self.assertEqual("OSM Name", street_name)
        self.assertEqual("1", street_type)


class TestHtmlTableFromList(unittest.TestCase):
    """Tests html_table_from_list()."""
    def test_happy(self):
        """Tests the happy path."""
        fro = [["A1", "B1"], ["A2", "B2"]]
        expected = '<table rules="all" frame="border" cellpadding="4" class="sortable">'
        expected += '<tr><th align="left" valign="center"><a href="#">A1</a></th>'
        expected += '<th align="left" valign="center"><a href="#">B1</a></th></tr>'
        expected += '<tr><td align="left" valign="top">A2</td><td align="left" valign="top">B2</td></tr></table>'
        ret = helpers.html_table_from_list(fro)
        self.assertEqual(ret, expected)


class TestTsvToList(unittest.TestCase):
    """Tests tsv_to_list()."""
    def test_happy(self):
        """Tests the happy path."""
        sock = io.StringIO("h1\th2\n\nv1\tv2\n")
        ret = helpers.tsv_to_list(sock)
        self.assertEqual(ret, [['h1', 'h2\n'], ['v1', 'v2\n']])


class TestNormalize(unittest.TestCase):
    """Tests normalize()."""
    def test_happy(self):
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("139", "Budaörsi út", normalizers)
        self.assertEqual(house_numbers, ["139"])

    def test_not_in_range(self):
        """Tests when the number is not in range."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("999", "Budaörsi út", normalizers)
        self.assertEqual(house_numbers, [])

    def test_not_a_number(self):
        """Tests the case when the house number is not a number."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("x", "Budaörsi út", normalizers)
        self.assertEqual(house_numbers, [])

    def test_nofilter(self):
        """Tests the case when there is no filter for this street."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("1", "Budaörs út", normalizers)
        self.assertEqual(house_numbers, ["1"])


class TestGetHouseNumbersFromLst(unittest.TestCase):
    """Tests get_house_numbers_from_lst()."""
    def test_happy(self):
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation_name = "gazdagret"
        street_name = "Törökugrató utca"
        normalizers, _ = helpers.load_normalizers(datadir, "gazdagret")
        ref_street = "Törökugrató utca"
        house_numbers = helpers.get_house_numbers_from_lst(datadir, relation_name, street_name, ref_street, normalizers)
        self.assertEqual(house_numbers, ["1", "2", "7", "10"])


class TestGetHouseNumbersFromCsv(unittest.TestCase):
    """Tests get_house_numbers_from_csv()."""
    def test_happy(self):
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation_name = "gazdagret"
        street_name = "Törökugrató utca"
        normalizers, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.get_house_numbers_from_csv(datadir, relation_name, street_name, normalizers)
        self.assertEqual(house_numbers, ["1", "2"])


class TestGetSuspiciousStreets(unittest.TestCase):
    """Tests get_suspicious_streets()."""
    def test_happy(self):
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation_name = "gazdagret"
        suspicious_streets, done_streets = helpers.get_suspicious_streets(datadir, datadir, relation_name)
        self.assertEqual(suspicious_streets, [('Törökugrató utca', ['7', '10']), ('Tűzkő utca', ['1', '2'])])
        expected = [('OSM Name 1', ['1', '2']), ('Törökugrató utca', ['1', '2']), ('Tűzkő utca', ['9', '10'])]
        self.assertEqual(done_streets, expected)


if __name__ == '__main__':
    unittest.main()
