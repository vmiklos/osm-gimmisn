#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

from typing import Any
from typing import Dict
from typing import List
from typing import Set
from typing import Tuple

import api
import rust


HouseNumberRange = rust.PyHouseNumberRange
Street = rust.PyStreet
HouseNumber = rust.PyHouseNumber
CsvIO = rust.PyCsvRead


def split_house_number(house_number: str) -> Tuple[int, str]:
    """Splits house_number into a numerical and a remainder part."""
    return rust.py_split_house_number(house_number)


def format_even_odd(only_in_ref: List[HouseNumberRange]) -> List[str]:
    """Formats even and odd numbers."""
    return rust.py_format_even_odd(only_in_ref)


def build_street_reference_cache(local_streets: str) -> Dict[str, Dict[str, List[str]]]:
    """Builds an in-memory cache from the reference on-disk TSV (street version)."""
    return rust.py_build_street_reference_cache(local_streets)


def get_reference_cache_path(local: str, refcounty: str) -> str:
    """Gets the filename of the (house number) reference cache file."""
    return rust.py_get_reference_cache_path(local, refcounty)


def build_reference_cache(local: str, refcounty: str) -> Dict[str, Dict[str, Dict[str, List[api.HouseNumberWithComment]]]]:
    """Builds an in-memory cache from the reference on-disk TSV (house number version)."""
    return rust.py_build_reference_cache(local, refcounty)


def parse_filters(tokens: List[str]) -> Dict[str, str]:
    """Parses a filter description, like 'filter-for', 'refcounty', '42'."""
    return rust.py_parse_filters(tokens)


def handle_overpass_error(ctx: rust.PyContext, http_error: str) -> rust.PyDoc:
    """Handles a HTTP error from Overpass."""
    return rust.py_handle_overpass_error(ctx, http_error)


def setup_localization(headers: List[Tuple[str, str]]) -> str:
    """Provides localized strings for this thread."""
    return rust.py_setup_localization(headers)


def gen_link(url: str, label: str) -> rust.PyDoc:
    """Generates a link to a URL with a given label."""
    return rust.py_gen_link(url, label)


def write_html_header(doc: rust.PyDoc) -> None:
    """Produces the verify first line of a HTML output."""
    return rust.py_write_html_header(doc)


def process_template(buf: str, osmrelation: int) -> str:
    """Turns an overpass query template to an actual query."""
    return rust.py_process_template(buf, osmrelation)


def html_table_from_list(table: List[List[rust.PyDoc]]) -> rust.PyDoc:
    """Produces a HTML table from a list of lists."""
    return rust.py_html_table_from_list(table)


def invalid_refstreets_to_html(osm_invalids: List[str], ref_invalids: List[str]) -> rust.PyDoc:
    """Produces HTML enumerations for 2 string lists."""
    return rust.py_invalid_refstreets_to_html(osm_invalids, ref_invalids)


def invalid_filter_keys_to_html(invalids: List[str]) -> rust.PyDoc:
    """Produces HTML enumerations for a string list."""
    return rust.py_invalid_filter_keys_to_html(invalids)


def get_column(row: List[rust.PyDoc], column_index: int) -> str:
    """Gets the nth column of row."""
    return rust.py_get_column(row, column_index)


def natnum(column: str) -> int:
    """Interpret the content as an integer."""
    return rust.py_natnum(column)


def tsv_to_list(stream: rust.PyCsvRead) -> List[List[rust.PyDoc]]:
    """Turns a tab-separated table into a list of lists."""
    return rust.py_tsv_to_list(stream)


def get_street_from_housenumber(sock: rust.PyCsvRead) -> List[rust.PyStreet]:
    """
    Reads a house number CSV and extracts streets from rows.
    Returns a list of street objects, with their name, ID and type set.
    """
    return rust.py_get_street_from_housenumber(sock)


def get_housenumber_ranges(house_numbers: List[rust.PyHouseNumber]) -> List[rust.PyHouseNumberRange]:
    """Gets a reference range list for a house number list by looking at what range provided a givne
    house number."""
    return rust.py_get_housenumber_ranges(house_numbers)


def git_link(version: str, prefix: str) -> rust.PyDoc:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    return rust.py_git_link(version, prefix)


def sort_numerically(strings: List[rust.PyHouseNumber]) -> List[rust.PyHouseNumber]:
    """Sorts strings according to their numerical value, not alphabetically."""
    return rust.py_sort_numerically(strings)


def get_in_both(first: List[Any], second: List[Any]) -> List[Any]:
    """Returns items which are in both first and second."""
    return rust.py_get_in_both(first, second)


def get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
    return rust.py_get_content(path)


def get_content_with_meta(path: str) -> Tuple[bytes, List[Tuple[str, str]]]:
    """Gets the content of a file in workdir with metadata."""
    return rust.py_get_content_with_meta(path)


def get_city_key(postcode: str, city: str, valid_settlements: Set[str]) -> str:
    """Constructs a city name based on postcode the nominal city."""
    return rust.py_get_city_key(postcode, city, valid_settlements)


def get_sort_key(string: str) -> bytes:
    """Returns a string comparator which allows Unicode-aware lexical sorting."""
    return rust.py_get_sort_key(string)


def get_valid_settlements(ctx: rust.PyContext) -> Set[str]:
    """Builds a set of valid settlement names."""
    return rust.py_get_valid_settlements(ctx)


def format_percent(english: str) -> str:
    """Formats a percentage, taking locale into account."""
    return rust.py_format_percent(english)


def get_timestamp(path: str) -> float:
    """Gets the timestamp of a file if it exists, 0 otherwise."""
    return rust.py_get_timestamp(path)


HouseNumbers = List[HouseNumber]
NumberedStreet = Tuple[Street, HouseNumbers]
NumberedStreets = List[NumberedStreet]


def to_bytes(string: str) -> bytes:
    """Encodes the string to UTF-8."""
    return string.encode("utf-8")


def from_bytes(array: bytes) -> str:
    """Decodes the string from UTF-8."""
    return array.decode("utf-8")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
