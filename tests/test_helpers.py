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

import helpers


class TestSortNumerically(unittest.TestCase):
    """Tests sort_numerically()."""
    def test_numbers(self) -> None:
        """Tests numbers."""
        ascending = helpers.sort_numerically(['1', '20', '3'])
        self.assertEqual(ascending, ['1', '3', '20'])

    def test_alpha_suffix(self) -> None:
        """Tests numbers with suffixes."""
        ascending = helpers.sort_numerically(['1a', '20a', '3a'])
        self.assertEqual(ascending, ['1a', '3a', '20a'])

    def test_alpha(self) -> None:
        """Tests just suffixes."""
        ascending = helpers.sort_numerically(['a', 'c', 'b'])
        self.assertEqual(ascending, ['a', 'b', 'c'])


class TestSplitHouseNumber(unittest.TestCase):
    """Tests split_house_number()."""
    def test_only_number(self) -> None:
        """Tests just numbers."""
        self.assertEqual(helpers.split_house_number('42'), (42, ''))

    def test_number_alpha(self) -> None:
        """Tests numbers and suffixes."""
        self.assertEqual(helpers.split_house_number('42ab'), (42, 'ab'))

    def test_alpha(self) -> None:
        """Tests just suffixes."""
        self.assertEqual(helpers.split_house_number('a'), (0, 'a'))


class TestSortStreetsCsv(unittest.TestCase):
    """Tests sort_streets_csv()."""
    def test_single_field(self) -> None:
        """Tests a single column."""
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(helpers.sort_streets_csv(unsorted), expected)

    def test_two_fields(self) -> None:
        """Tests 2 columns."""
        unsorted = 'head\n1\tb\n2\ta'
        expected = 'head\n2\ta\n1\tb'
        self.assertEqual(helpers.sort_streets_csv(unsorted), expected)


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
        self.assertEqual(helpers.sort_streets(unsorted), expected)

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
        self.assertEqual(helpers.sort_streets(unsorted), sort)


class TestSortHouseNumbersCsv(unittest.TestCase):
    """Tests sort_housenumbers_csv()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        unsorted = 'head\n2\n1'
        expected = 'head\n1\n2'
        self.assertEqual(helpers.sort_housenumbers_csv(unsorted), expected)


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
        self.assertEqual(helpers.sort_housenumbers(unsorted), expected)


class TestInBoth(unittest.TestCase):
    """Tests get_in_both()."""
    def test_happy(self) -> None:
        """Tests that happy path."""
        self.assertEqual(helpers.get_in_both([1, 2, 3], [2, 3, 4]), [2, 3])


class TestOnlyInFirst(unittest.TestCase):
    """Tests get_only_in_first()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(helpers.get_only_in_first([1, 2, 3], [3, 4]), [1, 2])


class TestGitLink(unittest.TestCase):
    """Tests git_link()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        actual = helpers.git_link("v1-151-g64ecc85", "http://www.example.com/")
        expected = "<a href=\"http://www.example.com/64ecc85\">v1-151-g64ecc85</a>"
        self.assertEqual(actual, expected)


class TestGetOsmStreets(unittest.TestCase):
    """Tests get_osm_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = helpers.get_osm_streets(workdir, "test")
        expected = ['B1', 'B2', 'HB1', 'HB2']
        self.assertEqual(actual, expected)

    def test_no_house_number(self) -> None:
        """Tests the case when we have streets, but no house numbers."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = helpers.get_osm_streets(workdir, "ujbuda")
        expected = ['OSM Name 1', 'Törökugrató utca', 'Tűzkő utca']
        self.assertEqual(actual, expected)


class TestRange(unittest.TestCase):
    """Tests Range."""
    def test_isodd_bad(self) -> None:
        """Tests an odd range with an even number."""
        test = helpers.Range(1, 3)
        self.assertFalse(2 in test)

    def test_range_bad(self) -> None:
        """Tests an odd range with a large number."""
        test = helpers.Range(1, 3)
        self.assertFalse(5 in test)

    def test_happy(self) -> None:
        """Tests the happy path."""
        test = helpers.Range(1, 5)
        self.assertTrue(1 in test)
        self.assertTrue(3 in test)
        self.assertTrue(5 in test)
        self.assertEqual(test.get_start(), 1)
        self.assertEqual(test.get_end(), 5)

    def test_eq(self) -> None:
        """Tests equality code."""
        self.assertTrue(helpers.Range(1, 5) != helpers.Range(3, 5))
        self.assertTrue(helpers.Range(1, 5) != helpers.Range(1, 3))
        self.assertTrue(helpers.Range(1, 3) != helpers.Range(1, 3, interpolation="all"))

    def test_interpolation_all(self) -> None:
        """Tests the interpolation modes."""
        self.assertFalse(2 in helpers.Range(1, 3))
        self.assertTrue(2 in helpers.Range(1, 3, interpolation="all"))


class TestRanges(unittest.TestCase):
    """Tests Ranges."""
    def test_a(self) -> None:
        """Tests when the arg is in the first range."""
        test = helpers.Ranges([helpers.Range(0, 0), helpers.Range(1, 1)])
        self.assertTrue(0 in test)

    def test_b(self) -> None:
        """Tests when the arg is in the second range."""
        test = helpers.Ranges([helpers.Range(0, 0), helpers.Range(1, 1)])
        self.assertTrue(1 in test)

    def test_ab(self) -> None:
        """Tests when the arg is in both ranges."""
        test = helpers.Ranges([helpers.Range(1, 1), helpers.Range(1, 1)])
        self.assertTrue(1 in test)

    def test_none(self) -> None:
        """Tests when the arg is in neither ranges."""
        test = helpers.Ranges([helpers.Range(0, 0), helpers.Range(1, 1)])
        self.assertFalse(2 in test)


class TestGetWorkdir(unittest.TestCase):
    """Tests get_workdir()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        config = configparser.ConfigParser()
        config.read_dict({"wsgi": {"workdir": "/path/to/workdir"}})
        actual = helpers.get_workdir(config)
        expected = "/path/to/workdir"
        self.assertEqual(actual, expected)


class TestProcessTemplate(unittest.TestCase):
    """Tests process_template()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        template = "aaa @RELATION@ bbb @AREA@ ccc"
        expected = "aaa 42 bbb 3600000042 ccc"
        actual = helpers.process_template(template, 42)
        self.assertEqual(actual, expected)


class TestGetStreetsQuery(unittest.TestCase):
    """Tests get_streets_query()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relations = helpers.Relations(datadir)
        relation = "gazdagret"
        ret = helpers.get_streets_query(datadir, relations, relation)
        self.assertEqual(ret, 'aaa 2713748 bbb 3602713748 ccc\n')


class TestGetStreetHousenumbersQuery(unittest.TestCase):
    """Tests get_street_housenumbers_query()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relations = helpers.Relations(datadir)
        relation = "gazdagret"
        ret = helpers.get_street_housenumbers_query(datadir, relations, relation)
        self.assertEqual(ret, 'housenr aaa 2713748 bbb 3602713748 ccc\n')


class TestWriteStreetsResult(unittest.TestCase):
    """Tests write_streets_result()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation = "gazdagret"
        result_from_overpass = "@id\tname\n1\tTűzkő utca\n2\tTörökugrató utca\n3\tOSM Name 1\n4\tHamzsabégi út\n"
        expected = helpers.get_content(workdir, "streets-gazdagret.csv")
        helpers.write_streets_result(workdir, relation, result_from_overpass)
        actual = helpers.get_content(workdir, "streets-gazdagret.csv")
        self.assertEqual(actual, expected)


class TestWriteStreetHousenumbersResult(unittest.TestCase):
    """Tests write_street_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation = "gazdagret"
        result_from_overpass = "@id\taddr:street\taddr:housenumber\n"
        result_from_overpass += "1\tTörökugrató utca\t1\n"
        result_from_overpass += "1\tTörökugrató utca\t2\n"
        result_from_overpass += "1\tTűzkő utca\t9\n"
        result_from_overpass += "1\tTűzkő utca\t10\n"
        result_from_overpass += "1\tOSM Name 1\t1\n"
        result_from_overpass += "1\tOSM Name 1\t2\n"
        result_from_overpass += "1\tOnly In OSM utca\t1\n"
        expected = helpers.get_content(workdir, "street-housenumbers-gazdagret.csv")
        helpers.write_street_housenumbers(workdir, relation, result_from_overpass)
        actual = helpers.get_content(workdir, "street-housenumbers-gazdagret.csv")
        self.assertEqual(actual, expected)


class TestGetContent(unittest.TestCase):
    """Tests get_content()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = helpers.get_content(workdir, "gazdagret.percent")
        expected = "54.55"
        self.assertEqual(actual, expected)


class TestLoadNormalizers(unittest.TestCase):
    """Tests load_normalizers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        filters, ref_streets, street_blacklist = helpers.load_normalizers(datadir, "gazdagret")
        expected_filters = {
            "Budaörsi út": helpers.Ranges([helpers.Range(137, 165)]),
            "Csiki-hegyek utca": helpers.Ranges([helpers.Range(1, 15), helpers.Range(2, 26)]),
            'Hamzsabégi út': helpers.Ranges([helpers.Range(start=1, end=2, interpolation="all")])
        }
        self.assertEqual(filters, expected_filters)
        expected_streets = {
            'OSM Name 1': 'Ref Name 1',
            'OSM Name 2': 'Ref Name 2'
        }
        self.assertEqual(ref_streets, expected_streets)
        self.assertEqual(street_blacklist, ['Only In Ref Nonsense utca'])

    def test_nosuchname(self) -> None:
        """Tests when there is no filters file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        filters, ref_streets, street_blacklist = helpers.load_normalizers(datadir, "nosuchname")
        self.assertEqual(filters, {})
        self.assertEqual(ref_streets, {})
        self.assertEqual(street_blacklist, [])

    def test_empty(self) -> None:
        """Tests when the filter file is empty."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        filters, ref_streets, street_blacklist = helpers.load_normalizers(datadir, "empty")
        self.assertEqual(filters, {})
        self.assertEqual(ref_streets, {})
        self.assertEqual(street_blacklist, [])


class TestGetStreetDetails(unittest.TestCase):
    """Tests get_street_details()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "Budaörsi út"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual(["011"], reftelepules)
        self.assertEqual("Budaörsi", street_name)
        self.assertEqual("út", street_type)

    def test_reftelepules_override(self) -> None:
        """Tests street-specific reftelepules override."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "Teszt utca"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual(["012"], reftelepules)
        self.assertEqual("Teszt", street_name)
        self.assertEqual("utca", street_type)

    def test_refstreets(self) -> None:
        """Tests OSM -> ref name mapping."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "OSM Name 1"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual(["011"], reftelepules)
        self.assertEqual("Ref Name", street_name)
        self.assertEqual("1", street_type)

    def test_nosuchrelation(self) -> None:
        """Tests a relation without a filter file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "OSM Name 1"
        relation_name = "nosuchrelation"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual(["011"], reftelepules)
        self.assertEqual("OSM Name", street_name)
        self.assertEqual("1", street_type)

    def test_emptyrelation(self) -> None:
        """Tests a relation with an empty filter file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "OSM Name 1"
        relation_name = "empty"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual(["011"], reftelepules)
        self.assertEqual("OSM Name", street_name)
        self.assertEqual("1", street_type)

    def test_range_level_override(self) -> None:
        """Tests the reftelepules range-level override."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        street = "Csiki-hegyek utca"
        relation_name = "gazdagret"
        refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
        self.assertEqual("01", refmegye)
        self.assertEqual(["011", "013"], reftelepules)
        self.assertEqual("Csiki-hegyek", street_name)
        self.assertEqual("utca", street_type)


class TestHtmlTableFromList(unittest.TestCase):
    """Tests html_table_from_list()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        fro = [["A1", "B1"], ["A2", "B2"]]
        expected = '<table class="sortable">'
        expected += '<tr><th><a href="#">A1</a></th>'
        expected += '<th><a href="#">B1</a></th></tr>'
        expected += '<tr><td>A2</td><td>B2</td></tr></table>'
        ret = helpers.html_table_from_list(fro)
        self.assertEqual(ret, expected)


class TestTsvToList(unittest.TestCase):
    """Tests tsv_to_list()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        sock = io.StringIO("h1\th2\n\nv1\tv2\n")
        ret = helpers.tsv_to_list(sock)
        self.assertEqual(ret, [['h1', 'h2\n'], ['v1', 'v2\n']])


class TestNormalize(unittest.TestCase):
    """Tests normalize()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("139", "Budaörsi út", normalizers)
        self.assertEqual(house_numbers, ["139"])

    def test_not_in_range(self) -> None:
        """Tests when the number is not in range."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("999", "Budaörsi út", normalizers)
        self.assertEqual(house_numbers, [])

    def test_not_a_number(self) -> None:
        """Tests the case when the house number is not a number."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("x", "Budaörsi út", normalizers)
        self.assertEqual(house_numbers, [])

    def test_nofilter(self) -> None:
        """Tests the case when there is no filter for this street."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        normalizers, _, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.normalize("1", "Budaörs út", normalizers)
        self.assertEqual(house_numbers, ["1"])


class TestGetHouseNumbersFromLst(unittest.TestCase):
    """Tests get_house_numbers_from_lst()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        street_name = "Törökugrató utca"
        normalizers, _, _ = helpers.load_normalizers(datadir, "gazdagret")
        ref_street = "Törökugrató utca"
        house_numbers = helpers.get_house_numbers_from_lst(workdir, relation_name, street_name, ref_street, normalizers)
        self.assertEqual(house_numbers, ["1", "2", "7", "10"])


class TestGetStreetsFromLst(unittest.TestCase):
    """Tests get_streets_from_lst()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        house_numbers = helpers.get_streets_from_lst(workdir, relation_name)
        self.assertEqual(house_numbers, ['Hamzsabégi út',
                                         'Only In Ref Nonsense utca',
                                         'Only In Ref utca',
                                         'Ref Name 1',
                                         'Törökugrató utca',
                                         'Tűzkő utca'])


class TestGetHouseNumbersFromCsv(unittest.TestCase):
    """Tests get_house_numbers_from_csv()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        street_name = "Törökugrató utca"
        normalizers, _, _ = helpers.load_normalizers(datadir, "gazdagret")
        house_numbers = helpers.get_house_numbers_from_csv(workdir, relation_name, street_name, normalizers)
        self.assertEqual(house_numbers, ["1", "2"])


class TestGetSuspiciousStreets(unittest.TestCase):
    """Tests get_suspicious_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        suspicious_streets, done_streets = helpers.get_suspicious_streets(datadir, workdir, relation_name)
        self.assertEqual(suspicious_streets, [('Törökugrató utca', ['7', '10']),
                                              ('Tűzkő utca', ['1', '2']),
                                              ('Hamzsabégi út', ['1'])])
        expected = [('OSM Name 1', ['1', '2']), ('Törökugrató utca', ['1', '2']), ('Tűzkő utca', ['9', '10'])]
        self.assertEqual(done_streets, expected)


class TestGetSuspiciousRelations(unittest.TestCase):
    """Tests get_suspicious_relations()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        only_in_reference, in_both = helpers.get_suspicious_relations(datadir, workdir, relation_name)

        # Not that 'Only In Ref Nonsense utca' is missing from this list.
        self.assertEqual(only_in_reference, ['Only In Ref utca'])

        self.assertEqual(in_both, ['Hamzsabégi út', 'Ref Name 1', 'Törökugrató utca', 'Tűzkő utca'])


class TestWriteSuspicousStreetsResult(unittest.TestCase):
    """Tests write_suspicious_streets_result()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        expected = helpers.get_content(workdir, "gazdagret.percent")
        ret = helpers.write_suspicious_streets_result(datadir, workdir, relation_name)
        todo_street_count, todo_count, done_count, percent, table = ret
        self.assertEqual(todo_street_count, 3)
        self.assertEqual(todo_count, 5)
        self.assertEqual(done_count, 6)
        self.assertEqual(percent, '54.55')
        self.assertEqual(table, [['Utcanév', 'Hiányzik db', 'Házszámok'],
                                 ['Törökugrató utca', '2', '7<br/>10'],
                                 ['Tűzkő utca', '2', '1<br/>2'],
                                 ['Hamzsabégi út', '1', '1']])
        actual = helpers.get_content(workdir, "gazdagret.percent")
        self.assertEqual(actual, expected)

    def test_empty(self) -> None:
        """Tests the case when percent can't be determined."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "empty"
        ret = helpers.write_suspicious_streets_result(datadir, workdir, relation_name)
        _todo_street_count, _todo_count, _done_count, percent, _table = ret
        self.assertEqual(percent, 'N/A')
        os.unlink(os.path.join(workdir, "empty.percent"))


class TestWriteMissingRelationsResult(unittest.TestCase):
    """Tests write_missing_relations_result()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        expected = helpers.get_content(workdir, "gazdagret-streets.percent")
        ret = helpers.write_missing_relations_result(datadir, workdir, relation_name)
        todo_count, done_count, percent, streets = ret
        self.assertEqual(todo_count, 1)
        self.assertEqual(done_count, 4)
        self.assertEqual(percent, '80.00')
        self.assertEqual(streets, ['Only In Ref utca'])
        actual = helpers.get_content(workdir, "gazdagret-streets.percent")
        self.assertEqual(actual, expected)

    def test_empty(self) -> None:
        """Tests the case when percent can't be determined."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "empty"
        ret = helpers.write_missing_relations_result(datadir, workdir, relation_name)
        _todo_count, _done_count, percent, _streets = ret
        self.assertEqual(percent, 'N/A')
        os.unlink(os.path.join(workdir, "empty-streets.percent"))


class TestBuildReferenceCache(unittest.TestCase):
    """Tests build_reference_cache()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        memory_cache = helpers.build_reference_cache(refpath)
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
        helpers.build_reference_cache(refpath)
        memory_cache = helpers.build_reference_cache(refpath)
        expected = {'01': {'011': {'Hamzsabégi út': ['1'],
                                   'Ref Name 1': ['1', '2'],
                                   'Törökugrató utca': ['1', '10', '2', '7'],
                                   'Tűzkő utca': ['1', '10', '2', '9']}}}
        self.assertEqual(memory_cache, expected)
        os.unlink(refpath + ".pickle")


class TestBuildStreetReferenceCache(unittest.TestCase):
    """Tests build_street_reference_cache()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "utcak_20190514.tsv")
        memory_cache = helpers.build_street_reference_cache(refpath)
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
        helpers.build_street_reference_cache(refpath)
        memory_cache = helpers.build_street_reference_cache(refpath)
        expected = {'01': {'011': ['Törökugrató utca',
                                   'Tűzkő utca',
                                   'Ref Name 1',
                                   'Only In Ref utca',
                                   'Only In Ref Nonsense utca',
                                   'Hamzsabégi út']}}
        self.assertEqual(memory_cache, expected)
        os.unlink(refpath + ".pickle")


class TestHouseNumbersOfStreet(unittest.TestCase):
    """Tests house_numbers_of_street()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        memory_cache = helpers.build_reference_cache(refpath)
        relation_name = "gazdagret"
        street = "Törökugrató utca"
        ret = helpers.house_numbers_of_street(datadir, memory_cache, relation_name, street)
        self.assertEqual(ret, ['Törökugrató utca 1', 'Törökugrató utca 10', 'Törökugrató utca 2', 'Törökugrató utca 7'])

    def test_missing(self) -> None:
        """Tests the case when the street is not in the reference."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        memory_cache = helpers.build_reference_cache(refpath)
        relation_name = "gazdagret"
        street = "No such utca"
        ret = helpers.house_numbers_of_street(datadir, memory_cache, relation_name, street)
        self.assertEqual(ret, [])


class TestStreetsOfRelation(unittest.TestCase):
    """Tests streets_of_relation()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "utcak_20190514.tsv")
        memory_cache = helpers.build_street_reference_cache(refpath)
        relation_name = "gazdagret"
        ret = helpers.streets_of_relation(datadir, memory_cache, relation_name)
        self.assertEqual(ret, ['Törökugrató utca',
                               'Tűzkő utca',
                               'Ref Name 1',
                               'Only In Ref utca',
                               'Only In Ref Nonsense utca',
                               'Hamzsabégi út'])


class TestGetReferenceHousenumbers(unittest.TestCase):
    """Tests get_reference_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        expected = helpers.get_content(workdir, "street-housenumbers-reference-gazdagret.lst")
        helpers.get_reference_housenumbers(refpath, datadir, workdir, relation_name)
        actual = helpers.get_content(workdir, "street-housenumbers-reference-gazdagret.lst")
        self.assertEqual(actual, expected)


class TestGetSortedReferenceStreets(unittest.TestCase):
    """Tests get_sorted_reference_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "utcak_20190514.tsv")
        datadir = os.path.join(os.path.dirname(__file__), "data")
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        relation_name = "gazdagret"
        expected = helpers.get_content(workdir, "streets-reference-gazdagret.lst")
        helpers.get_sorted_reference_streets(refpath, datadir, workdir, relation_name)
        actual = helpers.get_content(workdir, "streets-reference-gazdagret.lst")
        self.assertEqual(actual, expected)


class TestRelations(unittest.TestCase):
    """Tests the Relations class."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relations = helpers.Relations(datadir)
        self.assertEqual(relations.get_names(), ['empty', 'gazdagret', 'nosuchrelation', "ujbuda"])
        self.assertEqual([13, 42, 221998, 2713748], sorted([i["osmrelation"] for i in relations.get_values()]))


class TestGetRelationMissingStreets(unittest.TestCase):
    """Tests get_relation_missing_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation_name = "ujbuda"
        ret = helpers.get_relation_missing_streets(datadir, relation_name)
        self.assertEqual(ret, "only")

    def test_empty(self) -> None:
        """Tests the default value."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation_name = "empty"
        ret = helpers.get_relation_missing_streets(datadir, relation_name)
        self.assertEqual(ret, "no")

    def test_nosuchrelation(self) -> None:
        """Tests a relation without a filter file."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation_name = "nosuchrelation"
        ret = helpers.get_relation_missing_streets(datadir, relation_name)
        self.assertEqual(ret, "no")


class TestRefmegyeGetName(unittest.TestCase):
    """Tests refmegye_get_name()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(helpers.refmegye_get_name("01"), "Budapest")
        self.assertEqual(helpers.refmegye_get_name("99"), "")


class TestFormatEvenOdd(unittest.TestCase):
    """Tests format_even_odd()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        self.assertEqual(helpers.format_even_odd(["1", "2"]), ["1", "2"])

    def test_only_odd(self) -> None:
        """Tests when we have odd numbers only."""
        self.assertEqual(helpers.format_even_odd(["1", "3"]), ["1, 3"])

    def test_only_even(self) -> None:
        """Tests when we have even numbers only."""
        self.assertEqual(helpers.format_even_odd(["2", "4"]), ["2, 4"])


class TestRelationStreetIsEvenOdd(unittest.TestCase):
    """Tests relation_street_is_even_odd()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        datadir = os.path.join(os.path.dirname(__file__), "data")
        relation = helpers.relation_init(datadir, "gazdagret")
        filters = helpers.relation_get_filters(relation)
        street = helpers.relation_filters_get_street(filters, "Hamzsabégi út")
        self.assertFalse(helpers.relation_street_is_even_odd(street))

        street = helpers.relation_filters_get_street(filters, "Teszt utca")
        self.assertTrue(helpers.relation_street_is_even_odd(street))


if __name__ == '__main__':
    unittest.main()
