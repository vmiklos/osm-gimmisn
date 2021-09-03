#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Type hints for rust.so.
"""

from typing import Any
from typing import BinaryIO
from typing import Dict
from typing import List
from typing import Optional
from typing import Set
from typing import Tuple
from typing import TypeVar
from typing import cast
import api


class PyRange:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start: int, end: int, interpolation: str) -> None:
        ...

    def get_start(self) -> int:
        """The smallest integer."""
        ...

    def get_end(self) -> int:
        """The largest integer."""
        ...

    def is_odd(self) -> Optional[bool]:
        """None for all house numbers on one side, bool otherwise."""
        ...

    def __contains__(self, item: int) -> bool:
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        ...


class PyRanges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items: List[PyRange]) -> None:
        ...

    def get_items(self) -> List[PyRange]:
        """The list of contained Range objects."""
        ...

    def __contains__(self, item: int) -> bool:
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        ...


class PyDoc:
    """Generates xml/html documents."""
    def __init__(self) -> None:
        ...

    @staticmethod
    def from_text(text: str) -> 'PyDoc':
        """Factory of yattag.Doc from a string."""
        ...

    def get_value(self) -> str:
        """Gets the escaped value."""
        ...

    def append_value(self, value: str) -> None:
        """Appends escaped content to the value."""
        ...

    def tag(self, name: str, attrs: List[Tuple[str, str]]) -> 'PyTag':
        """Starts a new tag."""
        ...

    def stag(self, name: str, attrs: List[Tuple[str, str]]) -> None:
        """Starts a new tag and closes it as well."""
        ...

    def text(self, text: str) -> None:
        """Appends unescaped content to the document."""
        ...


class PyTag:
    """Starts a tag, which is closed automatically."""
    def __init__(self, doc: PyDoc, name: str, attrs: List[Tuple[str, str]]) -> None:
        ...

    def __enter__(self) -> None:
        ...

    def __exit__(self, tpe: Any, value: Any, traceback: Any) -> None:
        ...

def py_parse(raw_languages: str) -> List[str]:
    """
    Parse a RFC 2616 Accept-Language string.
    https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14

    :param accept_language_str: A string in RFC 2616 format.
    """
    ...

def py_get_version() -> str:
    """Gets the git version."""
    ...

class PyStdFileSystem(api.FileSystem):
    """File system implementation, backed by the Rust stdlib."""
    def __init__(self) -> None:
        ...

    def path_exists(self, path: str) -> bool:
        ...

    def getmtime(self, path: str) -> float:
        ...

    def open_read(self, path: str) -> BinaryIO:
        ...

    def open_write(self, path: str) -> BinaryIO:
        ...

class PyIni:
    """Configuration file reader."""
    def __init__(self, config_path: str, root: str) -> None:
        ...

    def get_workdir(self) -> str:
        """Gets the directory which is writable."""
        ...

    def get_reference_housenumber_paths(self) -> List[str]:
        """Gets the abs paths of ref housenumbers."""
        ...

    def get_reference_street_path(self) -> str:
        """Gets the abs path of ref streets."""
        ...

    def get_reference_citycounts_path(self) -> str:
        """Gets the abs path of ref citycounts."""
        ...

    def get_uri_prefix(self) -> str:
        """Gets the global URI prefix."""
        ...

    def get_tcp_port(self) -> int:
        """Gets the TCP port to be used."""
        ...

    def get_overpass_uri(self) -> str:
        """Gets the URI of the overpass instance to be used."""
        ...

    def get_cron_update_inactive(self) -> bool:
        """Should cron.py update inactive relations?"""
        ...

class PyContext:
    """Context owns global state which is set up once and then read everywhere."""
    def __init__(self, prefix: str) -> None:
        ...

    def get_abspath(self, rel_path: str) -> str:
        """Make a path absolute, taking the repo root as a base dir."""
        ...

    def get_ini(self) -> PyIni:
        """Gets the ini file."""
        ...

    def set_network(self, network: api.Network) -> None:
        """Sets the network implementation."""
        ...

    def get_network(self) -> api.Network:
        """Gets the network implementation."""
        ...

    def set_time(self, time: api.Time) -> None:
        """Sets the time implementation."""
        ...

    def get_time(self) -> api.Time:
        """Gets the time implementation."""
        ...

    def set_subprocess(self, subprocess: api.Subprocess) -> None:
        """Sets the subprocess implementation."""
        ...

    def get_subprocess(self) -> api.Subprocess:
        """Gets the subprocess implementation."""
        ...

    def set_unit(self, unit: api.Unit) -> None:
        """Sets the unit implementation."""
        ...

    def get_unit(self) -> api.Unit:
        """Gets the unit implementation."""
        ...

    def set_file_system(self, file_system: api.FileSystem) -> None:
        """Sets the file system implementation."""
        ...

    def get_file_system(self) -> api.FileSystem:
        """Gets the file system implementation."""
        ...

def py_overpass_query(ctx: PyContext, query: str) -> str:
    """Posts the query string to the overpass API and returns the result string."""
    ...

def py_overpass_query_need_sleep(ctx: PyContext) -> int:
    """Checks if we need to sleep before executing an overpass query."""
    ...

def py_set_language(language: str) -> None:
    """Sets the language of the current thread."""
    ...


def py_get_language() -> str:
    """Gets the language of the current thread."""
    ...


def py_translate(english: str) -> str:
    """Translates English input according to the current UI language."""
    ...

class PyLetterSuffixStyle:
    @staticmethod
    def upper() -> int:
        ...

    @staticmethod
    def lower() -> int:
        ...

class PyHouseNumberRange:
    """
    A house number range is a string that may expand to one or more HouseNumber instances in the
    future. It can also have a comment.
    """
    def __init__(self, number: str, comment: str) -> None:
        ...

    def get_number(self) -> str:
        """Returns the house number (range) string."""
        ...

    def get_comment(self) -> str:
        """Returns the comment."""
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        """Comment is explicitly non-interesting."""
        ...

    def __lt__(self, other: object) -> bool:
        """Comment is explicitly non-interesting."""
        ...

    def __hash__(self) -> int:
        """Comment is explicitly non-interesting."""
        ...

class PyStreet:
    """
    A street has an OSM and a reference name. Ideally the two are the same. Sometimes the reference
    name differs.
    """
    def __init__(
        self, osm_name: str, ref_name: str, show_ref_street: bool, osm_id: int
    ) -> None:
        ...

    @staticmethod
    def from_string(osm_name: str) -> "PyStreet":
        """Constructor that only requires an OSM name."""
        ...

    def get_diff_key(self) -> str:
        """Gets a string that is used while diffing."""
        ...

    def get_osm_name(self) -> str:
        """Returns the OSM name."""
        ...

    def get_ref_name(self) -> str:
        """Returns the reference name."""
        ...

    def get_osm_id(self) -> int:
        """Returns the OSM (way) id."""
        ...

    def set_osm_type(self, osm_type: str) -> None:
        """Sets the OSM type, e.g. 'way'."""
        ...

    def get_osm_type(self) -> str:
        """Returns the OSM type, e.g. 'way'."""
        ...

    def set_source(self, source: str) -> None:
        """Sets the source of this street."""
        ...

    def get_source(self) -> str:
        """Gets the source of this street."""
        ...

    def to_html(self) -> PyDoc:
        """Writes the street as a HTML string."""
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        """OSM id is explicitly non-interesting."""
        ...

    def __lt__(self, other: object) -> bool:
        """OSM id is explicitly non-interesting."""
        ...

    def __hash__(self) -> int:
        """OSM id is explicitly not interesting."""
        ...

class PyHouseNumber:
    """
    A house number is a string which remembers what was its provider range.  E.g. the "1-3" string
    can generate 3 house numbers, all of them with the same range.
    The comment is similar to source, it's ignored during __eq__() and __hash__().
    """
    def __init__(self, number: str, source: str, comment: str) -> None:
        ...

    def get_number(self) -> str:
        """Returns the house number string."""
        ...

    def get_diff_key(self) -> str:
        """Gets a string that is used while diffing."""
        ...

    def get_source(self) -> str:
        """Returns the source range."""
        ...

    def get_comment(self) -> str:
        """Returns the comment."""
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        """Source is explicitly non-interesting."""
        ...

    def __hash__(self) -> int:
        """Source is explicitly non-interesting."""
        ...

    @staticmethod
    def is_invalid(house_number: str, invalids: List[str]) -> bool:
        """Decides if house_number is invalid according to invalids."""
        ...

    @staticmethod
    def has_letter_suffix(house_number: str, source_suffix: str) -> bool:
        """
        Determines if the input is a house number, allowing letter suffixes. This means not only
        '42' is allowed, but also '42a', '42/a' and '42 a'. Everything else is still considered just
        junk after the numbers.
        """
        ...

    @staticmethod
    def normalize_letter_suffix(house_number: str, source_suffix: str, style: int) -> str:
        """
        Turn '42A' and '42 A' (and their lowercase versions) into '42/A'.
        """
        ...

class PyCsvRead:
    def __init__(self, stream: BinaryIO) -> None:
        ...

    def __enter__(self) -> 'PyCsvRead':
        ...

    def __exit__(self, _exc_type: Any, _exc_value: Any, _exc_traceback: Any) -> bool:
        ...

    def get_rows(self) -> List[List[str]]:
        """Gets access to the rows of the CSV."""
        ...

def py_split_house_number(house_number: str) -> Tuple[int, str]:
    """Splits house_number into a numerical and a remainder part."""
    ...

def py_split_house_number_range(house_number: PyHouseNumberRange) -> Tuple[int, str]:
    """Wrapper around split_house_number() for HouseNumberRange objects."""
    ...

def py_format_even_odd(only_in_ref: List[PyHouseNumberRange]) -> List[str]:
    """Formats even and odd numbers."""
    ...


def py_format_even_odd_html(only_in_ref: List[PyHouseNumberRange]) -> PyDoc:
    """Formats even and odd numbers, HTML version."""
    ...


def py_color_house_number(house_number: PyHouseNumberRange) -> PyDoc:
    """Colors a house number according to its suffix."""
    ...

def py_build_street_reference_cache(local_streets: str) -> Dict[str, Dict[str, List[str]]]:
    """Builds an in-memory cache from the reference on-disk TSV (street version)."""
    ...

def py_get_reference_cache_path(local: str, refcounty: str) -> str:
    """Gets the filename of the (house number) reference cache file."""
    ...

def py_build_reference_cache(local: str, refcounty: str) -> Dict[str, Dict[str, Dict[str, List[api.HouseNumberWithComment]]]]:
    """Builds an in-memory cache from the reference on-disk TSV (house number version)."""
    ...

def py_build_reference_caches(
        references: List[str],
        refcounty: str
) -> List[Dict[str, Dict[str, Dict[str, List[api.HouseNumberWithComment]]]]]:
    """Handles a list of references for build_reference_cache()."""
    ...

def py_parse_filters(tokens: List[str]) -> Dict[str, str]:
    """Parses a filter description, like 'filter-for', 'refcounty', '42'."""
    ...

def py_handle_overpass_error(ctx: PyContext, http_error: str) -> PyDoc:
    """Handles a HTTP error from Overpass."""
    ...

def py_setup_localization(headers: List[Tuple[str, str]]) -> str:
    """Provides localized strings for this thread."""
    ...

def py_gen_link(url: str, label: str) -> PyDoc:
    """Generates a link to a URL with a given label."""
    ...

def py_write_html_header(doc: PyDoc) -> None:
    """Produces the verify first line of a HTML output."""
    ...

def py_process_template(buf: str, osmrelation: int) -> str:
    """Turns an overpass query template to an actual query."""
    ...

def py_should_expand_range(numbers: List[int], street_is_even_odd: bool) -> Tuple[bool, int]:
    """Decides if an x-y range should be expanded. Returns a sanitized end value as well."""
    ...

def py_html_table_from_list(table: List[List[PyDoc]]) -> PyDoc:
    """Produces a HTML table from a list of lists."""
    ...

def py_invalid_refstreets_to_html(osm_invalids: List[str], ref_invalids: List[str]) -> PyDoc:
    """Produces HTML enumerations for 2 string lists."""
    ...

def py_invalid_filter_keys_to_html(invalids: List[str]) -> PyDoc:
    """Produces HTML enumerations for a string list."""
    ...

def py_get_column(row: List[PyDoc], column_index: int) -> str:
    """Gets the nth column of row."""
    ...

def py_natnum(column: str) -> int:
    """Interpret the content as an integer."""
    ...

def py_tsv_to_list(stream: PyCsvRead) -> List[List[PyDoc]]:
    """Turns a tab-separated table into a list of lists."""
    ...

def py_get_street_from_housenumber(sock: PyCsvRead) -> List[PyStreet]:
    """
    Reads a house number CSV and extracts streets from rows.
    Returns a list of street objects, with their name, ID and type set.
    """
    ...

def py_get_housenumber_ranges(house_numbers: List[PyHouseNumber]) -> List[PyHouseNumberRange]:
    """Gets a reference range list for a house number list by looking at what range provided a givne
    house number."""
    ...

def py_git_link(version: str, prefix: str) -> PyDoc:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    ...

def py_sort_numerically(strings: List[PyHouseNumber]) -> List[PyHouseNumber]:
    """Sorts strings according to their numerical value, not alphabetically."""
    ...

Diff = TypeVar("Diff", PyHouseNumber, PyStreet)

def py_get_only_in_first(first: List[Diff], second: List[Diff]) -> List[Diff]:
    """Returns items which are in first, but not in second."""
    ...


def py_get_in_both(first: List[Diff], second: List[Diff]) -> List[Diff]:
    """Returns items which are in both first and second."""
    ...

def py_get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
    ...

def py_get_content_with_meta(path: str) -> Tuple[bytes, List[Tuple[str, str]]]:
    """Gets the content of a file in workdir with metadata."""
    ...

def py_get_normalizer(street_name: str, normalizers: Dict[str, PyRanges]) -> PyRanges:
    """Determines the normalizer for a given street."""
    ...

def py_split_house_number_by_separator(
        house_numbers: str,
        separator: str,
        normalizer: PyRanges
) -> Tuple[List[int], List[int]]:
    """Splits a house number string (possibly a range) by a given separator.
    Returns a filtered and a not filtered list of ints."""
    ...

def py_get_city_key(postcode: str, city: str, valid_settlements: Set[str]) -> str:
    """Constructs a city name based on postcode the nominal city."""
    ...

def py_get_sort_key(string: str) -> bytes:
    """Returns a string comparator which allows Unicode-aware lexical sorting."""
    ...

def py_get_valid_settlements(ctx: PyContext) -> Set[str]:
    """Builds a set of valid settlement names."""
    ...

def py_format_percent(english: str) -> str:
    """Formats a percentage, taking locale into account."""
    ...

def py_get_timestamp(path: str) -> float:
    """Gets the timestamp of a file if it exists, 0 otherwise."""
    ...

# vim:set shiftwidth=4 softtabstop=4 expandtab:
