#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

from typing import Callable
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import Set
from typing import Tuple
from typing import TypeVar
from typing import Union
import locale
import os
import re
import email.utils

import yattag

from rust import py_translate as tr
import context
import rust


HouseNumberRange = rust.PyHouseNumberRange
Street = rust.PyStreet
HouseNumber = rust.PyHouseNumber
CsvIO = rust.PyCsvRead
split_house_number = rust.py_split_house_number
split_house_number_range = rust.py_split_house_number_range
format_even_odd = rust.py_format_even_odd
format_even_odd_html = rust.py_format_even_odd_html
color_house_number = rust.py_color_house_number
build_street_reference_cache = rust.py_build_street_reference_cache
get_reference_cache_path = rust.py_get_reference_cache_path
build_reference_cache = rust.py_build_reference_cache
build_reference_caches = rust.py_build_reference_caches
parse_filters = rust.py_parse_filters
handle_overpass_error = rust.py_handle_overpass_error
setup_localization = rust.py_setup_localization
gen_link = rust.py_gen_link
write_html_header = rust.py_write_html_header
process_template = rust.py_process_template
should_expand_range = rust.py_should_expand_range
html_table_from_list = rust.py_html_table_from_list

HouseNumbers = List[HouseNumber]
NumberedStreet = Tuple[Street, HouseNumbers]
NumberedStreets = List[NumberedStreet]


def invalid_refstreets_to_html(invalids: Tuple[List[str], List[str]]) -> yattag.Doc:
    """Produces HTML enumerations for 2 string lists."""
    doc = yattag.Doc()
    osm_invalids, ref_invalids = invalids
    if osm_invalids:
        doc.stag("br", [])
        with doc.tag("div", [("id", "osm-invalids-container")]):
            doc.text(tr("Warning: broken OSM <-> reference mapping, the following OSM names are invalid:"))
            with doc.tag("ul", []):
                for osm_invalid in osm_invalids:
                    with doc.tag("li", []):
                        doc.text(osm_invalid)
    if ref_invalids:
        doc.stag("br", [])
        with doc.tag("div", [("id", "ref-invalids-container")]):
            doc.text(tr("Warning: broken OSM <-> reference mapping, the following reference names are invalid:"))
            with doc.tag("ul", []):
                for ref_invalid in ref_invalids:
                    with doc.tag("li", []):
                        doc.text(ref_invalid)
    if osm_invalids or ref_invalids:
        doc.stag("br", [])
        doc.text(tr("Note: an OSM name is invalid if it's not in the OSM database."))
        doc.text(tr("A reference name is invalid if it's in the OSM database."))
    return doc


def invalid_filter_keys_to_html(invalids: List[str]) -> yattag.Doc:
    """Produces HTML enumerations for a string list."""
    doc = yattag.Doc()
    if invalids:
        doc.stag("br", [])
        with doc.tag("div", [("id", "osm-filter-key-invalids-container")]):
            doc.text(tr("Warning: broken filter key name, the following key names are not OSM names:"))
            with doc.tag("ul", []):
                for invalid in invalids:
                    with doc.tag("li", []):
                        doc.text(invalid)
    return doc


def get_column(row: List[yattag.Doc], column_index: int, natnum: bool) -> Union[str, int]:
    """Gets the nth column of row, possibly interpreting the content as an integer."""
    ret = ""
    if column_index >= len(row):
        ret = row[0].get_value()
    else:
        ret = row[column_index].get_value()
    if natnum:
        try:
            number = ret
            match = re.match(r"([0-9]+).*", number)
            if match:
                number = match.group(1)
            return int(number)
        except ValueError:
            return 0
    return ret


def tsv_to_list(stream: CsvIO) -> List[List[yattag.Doc]]:
    """Turns a tab-separated table into a list of lists."""
    table = []

    first = True
    columns: Dict[str, int] = {}
    for row in stream.get_rows():
        if first:
            first = False
            for index, label in enumerate(row):
                columns[label] = index
        cells = [yattag.Doc.from_text(cell.strip()) for cell in row]
        if cells and "@type" in columns:
            # We know the first column is an OSM ID.
            try:
                osm_id = int(cells[0].get_value())
                osm_type = cells[columns["@type"]].get_value()
                doc = yattag.Doc()
                href = "https://www.openstreetmap.org/{}/{}".format(osm_type, osm_id)
                with doc.tag("a", [("href", href), ("target", "_blank")]):
                    doc.text(str(osm_id))
                cells[0] = doc
            except ValueError:
                # Not an int, ignore.
                pass
        table.append(cells)

    if "addr:street" in columns and "addr:housenumber" in columns:
        header = table[0]
        table = table[1:]
        table.sort(key=lambda row: get_column(row, columns["addr:housenumber"], natnum=True))
        table.sort(key=lambda row: get_column(row, columns["addr:street"], natnum=False))
        table = [header] + table

    return table


def get_street_from_housenumber(sock: CsvIO) -> List[Street]:
    """
    Reads a house number CSV and extracts streets from rows.
    Returns a list of street objects, with their name, ID and type set.
    """
    ret = []

    first = True
    columns: Dict[str, int] = {}
    for row in sock.get_rows():
        if first:
            first = False
            for index, label in enumerate(row):
                columns[label] = index
            continue

        has_housenumber = row[columns["addr:housenumber"]]
        has_conscriptionnumber = row[columns["addr:conscriptionnumber"]]
        if (not has_housenumber) and (not has_conscriptionnumber):
            continue
        street_name = row[columns["addr:street"]]
        if not street_name and "addr:place" in columns:
            street_name = row[columns["addr:place"]]
        if not street_name:
            continue

        osm_type = row[columns["@type"]]
        try:
            osm_id = int(row[0])
        except ValueError:
            osm_id = 0
        street = Street(osm_name=street_name, ref_name="", show_ref_street=True, osm_id=osm_id)
        street.set_osm_type(osm_type)
        street.set_source(tr("housenumber"))
        ret.append(street)

    return ret


def get_housenumber_ranges(house_numbers: List[HouseNumber]) -> List[HouseNumberRange]:
    """Gets a reference range list for a house number list by looking at what range provided a givne
    house number."""
    ret = []
    for house_number in house_numbers:
        ret.append(HouseNumberRange(house_number.get_source(), house_number.get_comment()))
    return sorted(set(ret))


def git_link(version: str, prefix: str) -> yattag.Doc:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    commit_hash = re.sub(".*-g([0-9a-f]+)(-modified)?", r"\1", version)
    doc = yattag.Doc()
    with doc.tag("a", [("href", prefix + commit_hash)]):
        doc.text(version)
    return doc


def sort_numerically(strings: Iterable[HouseNumber]) -> List[HouseNumber]:
    """Sorts strings according to their numerical value, not alphabetically."""
    return sorted(strings, key=lambda x: split_house_number(x.get_number()))


Diff = TypeVar("Diff", HouseNumber, Street)


def get_only_in_first(first: List[Diff], second: List[Diff]) -> List[Diff]:
    """
    Returns items which are in first, but not in second.
    """
    # Strip suffix that is ignored.
    if not first:
        return []

    first_stripped = [i.get_diff_key() for i in first]
    second_stripped = [i.get_diff_key() for i in second]

    ret = []
    for index, item in enumerate(first_stripped):
        if item not in second_stripped:
            ret.append(first[index])
    return ret


def get_in_both(first: List[Diff], second: List[Diff]) -> List[Diff]:
    """
    Returns items which are in both first and second.
    """
    # Strip suffix that is ignored.
    if not first:
        return []

    first_stripped = [i.get_diff_key() for i in first]
    second_stripped = [i.get_diff_key() for i in second]

    ret = []
    for index, item in enumerate(first_stripped):
        if item in second_stripped:
            ret.append(first[index])
    return ret


def get_content(workdir: str, path: str = "", extra_headers: Optional[List[Tuple[str, str]]] = None) -> bytes:
    """Gets the content of a file in workdir."""
    ret = bytes()
    if path:
        path = os.path.join(workdir, path)
    else:
        path = workdir
    with open(path, "rb") as sock:
        ret = sock.read()
        if extra_headers is not None:
            stat = os.fstat(sock.fileno())
            modified = email.utils.formatdate(stat.st_mtime, usegmt=True)
            extra_headers.append(("Last-Modified", modified))
    return ret


def get_normalizer(street_name: str, normalizers: Dict[str, rust.PyRanges]) -> rust.PyRanges:
    """Determines the normalizer for a given street."""
    if street_name in normalizers.keys():
        # Have a custom filter.
        normalizer = normalizers[street_name]
    else:
        # Default sanity checks.
        default = [
            rust.PyRange(1, 999, interpolation=""),
            rust.PyRange(2, 998, interpolation=""),
        ]
        normalizer = rust.PyRanges(default)
    return normalizer


def split_house_number_by_separator(
        house_numbers: str,
        separator: str,
        normalizer: rust.PyRanges
) -> Tuple[List[int], List[int]]:
    """Splits a house number string (possibly a range) by a given separator.
    Returns a filtered and a not filtered list of ints."""
    ret_numbers = []
    # Same as ret_numbers, but if the range is 2-6 and we filter for 2-4, then 6 would be lost, so
    # in-range 4 would not be detected, so this one does not drop 6.
    ret_numbers_nofilter = []

    for house_number in house_numbers.split(separator):
        try:
            number = int(re.sub(r"([0-9]+).*", r"\1", house_number))
        except ValueError:
            continue

        ret_numbers_nofilter.append(number)

        if number not in normalizer:
            continue

        ret_numbers.append(number)

    return ret_numbers, ret_numbers_nofilter


def get_city_key(postcode: str, city: str, valid_settlements: Set[str]) -> str:
    """Constructs a city name based on postcode the nominal city."""
    city = city.lower()

    if city and postcode.startswith("1"):
        district = int(postcode[1:3])
        if 1 <= district <= 23:
            return city + "_" + postcode[1:3]
        return city

    if city in valid_settlements or city == "budapest":
        return city
    if city:
        return "_Invalid"
    return "_Empty"


def get_valid_settlements(ctx: context.Context) -> Set[str]:
    """Builds a set of valid settlement names."""
    settlements: Set[str] = set()

    with open(ctx.get_ini().get_reference_citycounts_path(), "r") as stream:
        first = True
        for line in stream:
            if first:
                first = False
                continue
            cells = line.strip().split('\t')
            if not cells[0]:
                continue
            settlements.add(cells[0])

    return settlements


def format_percent(english: str) -> str:
    """Formats a percentage, taking locale into account."""
    parsed = float(english)
    formatted = '{0:.2f}%'.format(parsed)
    decimal_points = {
        "hu": ",",
    }
    decimal_point = decimal_points.get(rust.py_get_language(), ".")
    return formatted.replace(".", str(decimal_point))


def get_timestamp(path: str) -> float:
    """Gets the timestamp of a file if it exists, 0 otherwise."""
    try:
        return os.path.getmtime(path)
    except FileNotFoundError:
        return 0


def get_lexical_sort_key() -> Callable[[str], str]:
    """Returns a string comparator which allows Unicode-aware lexical sorting."""
    # This is good enough for now, English and Hungarian is all we support and this handles both.
    locale.setlocale(locale.LC_ALL, "hu_HU.UTF-8")
    return locale.strxfrm


def to_bytes(string: str) -> bytes:
    """Encodes the string to UTF-8."""
    return string.encode("utf-8")


def from_bytes(array: bytes) -> str:
    """Decodes the string from UTF-8."""
    return array.decode("utf-8")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
