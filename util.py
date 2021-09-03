#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

from typing import List
from typing import Tuple
import os

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
get_only_in_first = rust.py_get_only_in_first
get_in_both = rust.py_get_in_both
get_content = rust.py_get_content
get_content_with_meta = rust.py_get_content_with_meta
get_normalizer = rust.py_get_normalizer
split_house_number_by_separator = rust.py_split_house_number_by_separator
get_city_key = rust.py_get_city_key
get_sort_key = rust.py_get_sort_key
get_valid_settlements = rust.py_get_valid_settlements

HouseNumbers = List[HouseNumber]
NumberedStreet = Tuple[Street, HouseNumbers]
NumberedStreets = List[NumberedStreet]


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


def to_bytes(string: str) -> bytes:
    """Encodes the string to UTF-8."""
    return string.encode("utf-8")


def from_bytes(array: bytes) -> str:
    """Decodes the string from UTF-8."""
    return array.decode("utf-8")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
