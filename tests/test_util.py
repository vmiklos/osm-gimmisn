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

import test_context

import rust
import util


class TestHandleOverpassError(unittest.TestCase):
    """Tests handle_overpass_error()."""
    def test_no_sleep(self) -> None:
        """Tests the case when no sleep is needed."""
        error = "HTTP Error 404: no such file"
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        doc = util.handle_overpass_error(ctx, error)
        expected = """<div id="overpass-error">Overpass error: HTTP Error 404: no such file</div>"""
        self.assertEqual(doc.get_value(), expected)

    def test_need_sleep(self) -> None:
        """Tests the case when sleep is needed."""
        error = "HTTP Error 404: no such file"
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-wait.txt")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        doc = util.handle_overpass_error(ctx, error)
        expected = """<div id="overpass-error">Overpass error: HTTP Error 404: no such file"""
        expected += """<br />Note: wait for 12 seconds</div>"""
        self.assertEqual(doc.get_value(), expected)


class TestSetupLocalization(unittest.TestCase):
    """Tests setup_localization()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        environ = {"HTTP_ACCEPT_LANGUAGE": "hu,en;q=0.9,en-US;q=0.8"}
        rust.py_set_language("en")
        util.setup_localization(list(environ.items()))
        self.assertEqual(rust.py_get_language(), "hu")

    def test_parse_error(self) -> None:
        """Tests the error path."""
        environ = {"HTTP_ACCEPT_LANGUAGE": ","}
        rust.py_set_language("en")
        util.setup_localization(list(environ.items()))
        self.assertEqual(rust.py_get_language(), "en")


class TestGenLink(unittest.TestCase):
    """Tests gen_link()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        doc = util.gen_link("http://www.example.com", "label")
        expected = '<a href="http://www.example.com">label...</a>'
        self.assertEqual(doc.get_value(), expected)


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
        fro = [[rust.PyDoc.from_text("A1"),
                rust.PyDoc.from_text("B1")],
               [rust.PyDoc.from_text("A2"),
                rust.PyDoc.from_text("B2")]]
        expected = '<table class="sortable">'
        expected += '<tr><th><a href="#">A1</a></th>'
        expected += '<th><a href="#">B1</a></th></tr>'
        expected += '<tr><td>A2</td><td>B2</td></tr></table>'
        ret = util.html_table_from_list(fro).get_value()
        self.assertEqual(ret, expected)


class TestTsvToList(unittest.TestCase):
    """Tests tsv_to_list()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        sock = util.make_csv_io(io.BytesIO(b"h1\th2\n\nv1\tv2\n"))
        ret = util.tsv_to_list(sock)
        self.assertEqual(len(ret), 2)
        row1 = [cell.get_value() for cell in ret[0]]
        self.assertEqual(row1, ['h1', 'h2'])
        row2 = [cell.get_value() for cell in ret[1]]
        self.assertEqual(row2, ['v1', 'v2'])

    def test_type(self) -> None:
        """Tests when a @type column is available."""
        stream = util.make_csv_io(io.BytesIO(b"@id\t@type\n42\tnode\n"))
        ret = util.tsv_to_list(stream)
        self.assertEqual(len(ret), 2)
        row1 = [cell.get_value() for cell in ret[0]]
        self.assertEqual(row1, ["@id", "@type"])
        row2 = [cell.get_value() for cell in ret[1]]
        cell_a2 = '<a href="https://www.openstreetmap.org/node/42" target="_blank">42</a>'
        self.assertEqual(row2, [cell_a2, "node"])

    def test_escape(self) -> None:
        """Tests escaping."""
        sock = util.make_csv_io(io.BytesIO(b"\"h,1\"\th2\n"))
        ret = util.tsv_to_list(sock)
        self.assertEqual(len(ret), 1)
        row1 = [cell.get_value() for cell in ret[0]]
        # Note how this is just h,1 and not "h,1".
        self.assertEqual(row1, ['h,1', 'h2'])

    def test_sort(self) -> None:
        """Tests sorting."""
        csv = b"""addr:street\taddr:housenumber
A street\t1
A street\t10
A street\t9"""
        sock = util.make_csv_io(io.BytesIO(csv))
        ret = util.tsv_to_list(sock)
        # 0th is header
        row3 = [cell.get_value() for cell in ret[3]]
        # Note how 10 is ordered after 9.
        self.assertEqual(row3[1], "10")


class TestHouseNumber(unittest.TestCase):
    """Tests the HouseNumber class."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        house_number = util.make_house_number("1", "1-2", "")
        self.assertEqual(house_number.get_number(), "1")
        self.assertEqual(house_number.get_source(), "1-2")
        self.assertTrue(util.make_house_number("1", "1-2", "") != util.make_house_number("2", "1-2", ""))
        self.assertEqual(len(set([util.make_house_number("1", "1-2", ""),
                                  util.make_house_number("2", "1-2", ""),
                                  util.make_house_number("2", "1-2", "")])), 2)

    def test_is_invalid(self) -> None:
        """Tests is_invalid()."""
        self.assertTrue(rust.PyHouseNumber.is_invalid("15 a", ["15a"]))
        self.assertTrue(rust.PyHouseNumber.is_invalid("15/a", ["15a"]))
        self.assertTrue(rust.PyHouseNumber.is_invalid("15A", ["15a"]))
        self.assertTrue(rust.PyHouseNumber.is_invalid("67/5*", ["67/5"]))

        # Make sure we don't throw an exception on input which does not start with a number.
        self.assertFalse(rust.PyHouseNumber.is_invalid("A", ["15a"]))

    def test_has_letter_suffix(self) -> None:
        """Tests has_letter_suffix()."""
        self.assertTrue(rust.PyHouseNumber.has_letter_suffix("42a", ""))
        self.assertTrue(rust.PyHouseNumber.has_letter_suffix("42 a", ""))
        self.assertTrue(rust.PyHouseNumber.has_letter_suffix("42/a", ""))
        self.assertTrue(rust.PyHouseNumber.has_letter_suffix("42/a*", "*"))
        self.assertTrue(rust.PyHouseNumber.has_letter_suffix("42A", ""))
        self.assertFalse(rust.PyHouseNumber.has_letter_suffix("42 AB", ""))

    def test_normalize_letter_suffix(self) -> None:
        """Tests normalize_letter_suffix()."""
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42a", "", rust.PyLetterSuffixStyle.upper()), "42/A")
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42 a", "", rust.PyLetterSuffixStyle.upper()), "42/A")
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42/a", "", rust.PyLetterSuffixStyle.upper()), "42/A")
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42/A", "", rust.PyLetterSuffixStyle.upper()), "42/A")
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42/A*", "*", rust.PyLetterSuffixStyle.upper()), "42/A*")
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42 A", "", rust.PyLetterSuffixStyle.upper()), "42/A")
        with self.assertRaises(ValueError):
            rust.PyHouseNumber.normalize_letter_suffix("x", "", rust.PyLetterSuffixStyle.upper())
        self.assertEqual(rust.PyHouseNumber.normalize_letter_suffix("42/A", "", rust.PyLetterSuffixStyle.lower()), "42a")


class TestGetHousenumberRanges(unittest.TestCase):
    """Tests get_housenumber_ranges()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        house_numbers = [
            util.make_house_number("25", "25", ""),
            util.make_house_number("27", "27-37", ""),
            util.make_house_number("29", "27-37", ""),
            util.make_house_number("31", "27-37", ""),
            util.make_house_number("33", "27-37", ""),
            util.make_house_number("35", "27-37", ""),
            util.make_house_number("37", "27-37", ""),
            util.make_house_number("31*", "31*", ""),
        ]
        ranges = util.get_housenumber_ranges(house_numbers)
        range_names = [i.get_number() for i in ranges]
        self.assertEqual(range_names, ["25", "27-37", "31*"])


class TestGitLink(unittest.TestCase):
    """Tests git_link()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        actual = util.git_link("v1-151-g64ecc85", "http://www.example.com/").get_value()
        expected = "<a href=\"http://www.example.com/64ecc85\">v1-151-g64ecc85</a>"
        self.assertEqual(actual, expected)


class TestSortNumerically(unittest.TestCase):
    """Tests sort_numerically()."""
    def test_numbers(self) -> None:
        """Tests numbers."""
        ascending = util.sort_numerically([util.make_house_number('1', '', ''),
                                           util.make_house_number('20', '', ''),
                                           util.make_house_number('3', '', '')])
        self.assertEqual([i.get_number() for i in ascending], ['1', '3', '20'])

    def test_alpha_suffix(self) -> None:
        """Tests numbers with suffixes."""
        ascending = util.sort_numerically([util.make_house_number('1a', '', ''),
                                           util.make_house_number('20a', '', ''),
                                           util.make_house_number('3a', '', '')])
        self.assertEqual([i.get_number() for i in ascending], ['1a', '3a', '20a'])

    def test_alpha(self) -> None:
        """Tests just suffixes."""
        ascending = util.sort_numerically([util.make_house_number('a', '', ''),
                                           util.make_house_number('c', '', ''),
                                           util.make_house_number('b', '', '')])
        self.assertEqual([i.get_number() for i in ascending], ['a', 'b', 'c'])


class TestGetContent(unittest.TestCase):
    """Tests get_content()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = util.get_content(workdir + "/gazdagret.percent").decode("utf-8")
        expected = "54.55"
        self.assertEqual(actual, expected)

    def test_one_arg(self) -> None:
        """Tests the case when only one argument is given."""
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        actual = util.get_content(os.path.join(workdir, "gazdagret.percent")).decode("utf-8")
        expected = "54.55"
        self.assertEqual(actual, expected)


class TestStreet(unittest.TestCase):
    """Tests Street."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        street = util.make_street("foo", "bar", show_ref_street=True, osm_id=0)
        self.assertEqual(street.get_ref_name(), "bar")
        self.assertEqual(street.to_html().get_value(), "foo<br />(bar)")


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
        # We already use 'with':
        # pylint: disable=consider-using-with
        with util.make_csv_io(open("tests/workdir/street-housenumbers-gh964.csv", "rb")) as stream:
            actual = util.get_street_from_housenumber(stream)
        # This is picked up from addr:place because addr:street was empty.
        self.assertEqual(actual, [rust.PyStreet.from_string(osm_name="Tolvajos tanya")])


class TestInvalidFilterKeysToHtml(unittest.TestCase):
    """Tests invalid_filter_keys_to_html()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ret = util.invalid_filter_keys_to_html(["foo"])
        self.assertIn("<li>", ret.get_value())

    def test_empty(self) -> None:
        """Tests when the arg is empty."""
        ret = util.invalid_filter_keys_to_html([])
        self.assertEqual(ret.get_value(), "")


class TestGetColumn(unittest.TestCase):
    """Tests get_column()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        # id, street name, housenumber
        row = [
            rust.PyDoc.from_text("42"),
            rust.PyDoc.from_text("A street"),
            rust.PyDoc.from_text("1"),
        ]
        self.assertEqual(util.get_column(row, 1), "A street")
        self.assertEqual(util.natnum(util.get_column(row, 2)), 1)
        # Too large column index -> first column.
        self.assertEqual(util.get_column(row, 3), "42")

    def test_junk(self) -> None:
        """Tests the 'housenumber is junk' case."""
        # id, street name, housenumber
        row = [
            rust.PyDoc.from_text("42"),
            rust.PyDoc.from_text("A street"),
            rust.PyDoc.from_text("fixme"),
        ]
        self.assertEqual(util.natnum(util.get_column(row, 2)), 0)


class TestGetTimestamp(unittest.TestCase):
    """Tests get_timestamp()."""
    def test_no_such_file(self) -> None:
        """Tests what happens when the file is not there."""
        self.assertEqual(util.get_timestamp(""), 0)


class TestGetLexicalSortKey(unittest.TestCase):
    """Tests get_lexical_sort_key()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        # This is less naive than the classic "a, "á", "b", "c" list.
        strings = ["Kőpor", "Kórház"]
        strings.sort(key=util.get_sort_key)
        self.assertEqual(strings, ["Kórház", "Kőpor"])


if __name__ == '__main__':
    unittest.main()
