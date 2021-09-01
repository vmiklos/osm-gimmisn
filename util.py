#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

from typing import Callable
from typing import Dict
from typing import List
from typing import Optional
from typing import Set
from typing import Tuple
from typing import TypeVar
import locale
import os
import re
import email.utils

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
invalid_refstreets_to_html = rust.py_invalid_refstreets_to_html
invalid_filter_keys_to_html = rust.py_invalid_filter_keys_to_html
get_column = rust.py_get_column
natnum = rust.py_natnum
tsv_to_list = rust.py_tsv_to_list
get_street_from_housenumber = rust.py_get_street_from_housenumber
get_housenumber_ranges = rust.py_get_housenumber_ranges
git_link = rust.py_git_link
sort_numerically = rust.py_sort_numerically

HouseNumbers = List[HouseNumber]
NumberedStreet = Tuple[Street, HouseNumbers]
NumberedStreets = List[NumberedStreet]


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
