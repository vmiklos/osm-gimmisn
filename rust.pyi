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

def py_build_street_reference_cache(local_streets: str) -> Dict[str, Dict[str, List[str]]]:
    """Builds an in-memory cache from the reference on-disk TSV (street version)."""
    ...

def py_get_reference_cache_path(local: str, refcounty: str) -> str:
    """Gets the filename of the (house number) reference cache file."""
    ...

def py_build_reference_cache(local: str, refcounty: str) -> Dict[str, Dict[str, Dict[str, List[api.HouseNumberWithComment]]]]:
    """Builds an in-memory cache from the reference on-disk TSV (house number version)."""
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

def py_html_table_from_list(table: List[List[PyDoc]]) -> PyDoc:
    """Produces a HTML table from a list of lists."""
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

def py_get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
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

class PyRelationFiles:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, workdir: str, name: str):
        ...

    def get_ref_streets_path(self) -> str:
        """Build the file name of the reference street list of a relation."""
        ...

    def get_osm_streets_path(self) -> str:
        """Build the file name of the OSM street list of a relation."""
        ...

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        ...

    def get_ref_housenumbers_path(self) -> str:
        """Build the file name of the reference house number list of a relation."""
        ...

    def get_housenumbers_percent_path(self) -> str:
        """Builds the file name of the house number percent file of a relation."""
        ...

    def get_housenumbers_htmlcache_path(self) -> str:
        """Builds the file name of the house number HTML cache file of a relation."""
        ...

    def get_streets_percent_path(self) -> str:
        """Builds the file name of the street percent file of a relation."""
        ...

    def get_streets_additional_count_path(self) -> str:
        """Builds the file name of the street additional count file of a relation."""
        ...

    def get_housenumbers_additional_count_path(self) -> str:
        """Builds the file name of the housenumber additional count file of a relation."""
        ...

    def get_ref_streets_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the reference street list of a relation for reading."""
        ...

    def get_osm_streets_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the OSM street list of a relation for reading."""
        ...

    def get_osm_housenumbers_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the OSM house number list of a relation for reading."""
        ...

    def get_ref_housenumbers_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the reference house number list of a relation for reading."""
        ...

    def get_housenumbers_percent_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the house number percent file of a relation for reading."""
        ...

    def get_streets_percent_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street percent file of a relation for reading."""
        ...

    def get_streets_additional_count_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street additional count file of a relation for reading."""
        ...

    def get_streets_additional_count_write_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street additional count file of a relation for writing."""
        ...

    def get_housenumbers_additional_count_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the housenumbers additional count file of a relation for reading."""
        ...

    def get_housenumbers_additional_count_write_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the housenumbers additional count file of a relation for writing."""
        ...

    def write_osm_streets(self, ctx: PyContext, result: str) -> int:
        """Writes the result for overpass of Relation.get_osm_streets_query()."""
        ...

    def write_osm_housenumbers(self, ctx: PyContext, result: str) -> int:
        """Writes the result for overpass of Relation.get_osm_housenumbers_query()."""
        ...

class PyRelationConfig:
    """A relation configuration comes directly from static data, not a result of some external query."""
    def set_active(self, active: bool) -> None:
        """Sets if the relation is active."""
        ...

    def is_active(self) -> bool:
        """Gets if the relation is active."""
        ...

    def get_osmrelation(self) -> int:
        """Gets the OSM relation object's ID."""
        ...

    def get_refcounty(self) -> str:
        """Gets the relation's refcounty identifier from reference."""
        ...

    def get_refsettlement(self) -> str:
        """Gets the relation's refsettlement identifier from reference."""
        ...

    def get_alias(self) -> List[str]:
        """Gets the alias(es) of the relation: alternative names which are also accepted."""
        ...

    def should_check_missing_streets(self) -> str:
        """Return value can be 'yes', 'no' and 'only'."""
        ...

    def should_check_additional_housenumbers(self) -> bool:
        """Do we care if 42 is in OSM when it's not in the ref?."""
        ...

    def set_housenumber_letters(self, housenumber_letters: bool) -> None:
        """Sets the housenumber_letters property from code."""
        ...

    def set_letter_suffix_style(self, letter_suffix_style: int) -> None:
        """Sets the letter suffix style."""
        ...

    def get_letter_suffix_style(self) -> int:
        """Gets the letter suffix style."""
        ...

    def get_refstreets(self) -> Dict[str, str]:
        """Returns an OSM name -> ref name map."""
        ...

    def set_filters(self, filters: str) -> None:
        """Sets the 'filters' key from code."""
        ...

    def get_filters(self) -> Optional[str]:
        """Returns a street name -> properties map."""
        ...

    def get_street_is_even_odd(self, street: str) -> bool:
        """Determines in a relation's street is interpolation=all or not."""
        ...

    def should_show_ref_street(self, osm_street_name: str) -> bool:
        """Decides is a ref street should be shown for an OSM street."""
        ...

    def get_street_refsettlement(self, street: str) -> List[str]:
        """Returns a list of refsettlement values specific to a street."""
        ...

    def get_street_filters(self) -> List[str]:
        """Gets list of streets which are only in reference, but have to be filtered out."""
        ...

    def get_osm_street_filters(self) -> List[str]:
        """Gets list of streets which are only in OSM, but have to be filtered out."""
        ...

    def build_ref_streets(self, reference: Dict[str, Dict[str, List[str]]]) -> List[str]:
        """
        Builds a list of streets from a reference cache.
        """
        ...

    def get_ref_street_from_osm_street(self, osm_street_name: str) -> str:
        """Maps an OSM street name to a ref street name."""
        ...

class PyRelation:
    """A relation is a closed polygon on the map."""
    def get_name(self) -> str:
        """Gets the name of the relation."""
        ...

    def get_files(self) -> PyRelationFiles:
        """Gets access to the file interface."""
        ...

    def get_config(self) -> PyRelationConfig:
        """Gets access to the config interface."""
        ...

    def set_config(self, config: PyRelationConfig) -> None:
        """Sets the config interface."""
        ...

    def get_street_ranges(self) -> Dict[str, PyRanges]:
        """Gets a street name -> ranges map, which allows silencing false positives."""
        ...

    def should_show_ref_street(self, osm_street_name: str) -> bool:
        """Decides is a ref street should be shown for an OSM street."""
        ...

    def get_osm_streets(self, sorted_result: bool) -> List[PyStreet]:
        """Reads list of streets for an area from OSM."""
        ...

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
        ...

    def get_osm_housenumbers(self, street_name: str) -> List[PyHouseNumber]:
        """Gets the OSM house number list of a street."""
        ...

    def write_ref_streets(self, reference: str) -> None:
        """Gets known streets (not their coordinates) from a reference site, based on relation names
        from OSM."""
        ...

    def get_ref_streets(self) -> List[str]:
        """Gets streets from reference."""
        ...

    def build_ref_housenumbers(
            self,
            reference: Dict[str, Dict[str, Dict[str, List[api.HouseNumberWithComment]]]],
            street: str,
            suffix: str
    ) -> List[str]:
        """
        Builds a list of housenumbers from a reference cache.
        This is serialized to disk by write_ref_housenumbers().
        """
        ...

    def write_ref_housenumbers(self, references: List[str]) -> None:
        """
        Writes known house numbers (not their coordinates) from a reference, based on street names
        from OSM. Uses build_reference_cache() to build an indexed reference, the result will be
        used by get_ref_housenumbers().
        """
        ...

    def get_missing_housenumbers(self) -> Tuple[List[Tuple[PyStreet, List[PyHouseNumber]]], List[Tuple[PyStreet, List[PyHouseNumber]]]]:
        """
        Compares ref and osm house numbers, prints the ones which are in ref, but not in osm.
        Return value is a pair of ongoing and done streets.
        Each of of these is a pair of a street name and a house number list.
        """
        ...

    def get_missing_streets(self) -> Tuple[List[str], List[str]]:
        """Tries to find missing streets in a relation."""
        ...

    def get_additional_streets(self, sorted_result: bool) -> List[PyStreet]:
        """Tries to find additional streets in a relation."""
        ...

    def write_missing_streets(self) -> Tuple[int, int, str, List[str]]:
        """Calculate and write stat for the street coverage of a relation."""
        ...

    def write_additional_streets(self) -> List[PyStreet]:
        """Calculate and write stat for the unexpected street coverage of a relation."""
        ...

    def write_missing_housenumbers(self) -> Tuple[int, int, int, str, List[List[PyDoc]]]:
        """
        Calculate a write stat for the house number coverage of a relation.
        Returns a tuple of: todo street count, todo count, done count, percent and table.
        """
        ...

    def get_additional_housenumbers(self) -> List[Tuple[PyStreet, List[PyHouseNumber]]]:
        """
        Compares ref and osm house numbers, prints the ones which are in osm, but not in ref.
        Return value is a list of streets.
        Each of of these is a pair of a street name and a house number list.
        """
        ...

    def get_osm_housenumbers_query(self) -> str:
        """Produces a query which lists house numbers in relation."""
        ...

    def get_invalid_refstreets(self) -> Tuple[List[str], List[str]]:
        """Returns invalid osm names and ref names."""
        ...

    def get_invalid_filter_keys(self) -> List[str]:
        """Returns invalid filter key names (street not in OSM)."""
        ...

class PyRelations:
    """A relations object is a container of named relation objects."""
    def __init__(self, ctx: PyContext) -> None:
        ...

    def get_workdir(self) -> str:
        """Gets the workdir directory path."""
        ...

    def get_relation(self, name: str) -> PyRelation:
        """Gets the relation that has the specified name."""
        ...

    def set_relation(self, name: str, relation: PyRelation) -> None:
        """Sets a relation for testing."""
        ...

    def get_names(self) -> List[str]:
        """Gets a sorted list of relation names."""
        ...

    def get_active_names(self) -> List[str]:
        """Gets a sorted list of active relation names."""
        ...

    def activate_all(self, flag: bool) -> None:
        """Sets if inactive=true is ignored or not."""
        ...

    def get_relations(self) -> List[PyRelation]:
        """Gets a list of relations."""
        ...

    def limit_to_refcounty(self, refcounty: Optional[str]) -> None:
        """If refcounty is not None, forget about all relations outside that refcounty."""
        ...

    def limit_to_refsettlement(self, refsettlement: Optional[str]) -> None:
        """If refsettlement is not None, forget about all relations outside that refsettlement."""
        ...

    def refcounty_get_name(self, refcounty: str) -> str:
        """Produces a UI name for a refcounty."""
        ...

    def refcounty_get_refsettlement_ids(self, refcounty_name: str) -> List[str]:
        """Produces refsettlement IDs of a refcounty."""
        ...

    def refsettlement_get_name(self, refcounty_name: str, refsettlement: str) -> str:
        """Produces a UI name for a refsettlement in refcounty."""
        ...

    def get_aliases(self) -> Dict[str, str]:
        """Provide an alias -> real name map of relations."""
        ...

def py_make_turbo_query_for_streets(relation: PyRelation, streets: List[str]) -> str:
    """Creates an overpass query that shows all streets from a missing housenumbers table."""
    ...

def py_missing_housenumbers_main(argv: List[str], stdout: BinaryIO, ctx: PyContext) -> None:
    """Commandline interface."""
    ...

def py_get_footer(last_updated: str) -> PyDoc:
    """Produces the end of the page."""
    ...

def py_fill_missing_header_items(
    ctx: PyContext,
    streets: str,
    additional_housenumbers: bool,
    relation_name: str,
    items: List[PyDoc]
) -> List[PyDoc]:
    """Generates the 'missing house numbers/streets' part of the header."""
    ...

def py_get_toolbar(
        ctx: PyContext,
        relations: Optional[PyRelations],
        function: str,
        relation_name: str,
        relation_osmid: int
) -> PyDoc:
    """Produces the start of the page. Note that the content depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    ...

def py_handle_static(ctx: PyContext, request_uri: str) -> Tuple[bytes, str, List[Tuple[str, str]]]:
    """Handles serving static content."""
    ...

class PyResponse:
    """A HTTP response, to be sent by send_response()."""
    def __init__(self, content_type: str, status: str, output_bytes: bytes, headers: List[Tuple[str, str]]) -> None:
        ...

    def get_content_type(self) -> str:
        """Gets the Content-type value."""
        ...

    def get_status(self) -> str:
        """Gets the HTTP status."""
        ...

    def get_output_bytes(self) -> bytes:
        """Gets the encoded output."""
        ...

    def get_headers(self) -> List[Tuple[str, str]]:
        """Gets the HTTP headers."""
        ...

def py_send_response(
        environ: Dict[str, str],
        response: PyResponse
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Turns an output string into a byte array and sends it."""
    ...

def py_handle_exception(
        environ: Dict[str, str],
        error: str
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Displays an unhandled exception on the page."""
    ...

def py_handle_404() -> PyDoc:
    """Displays a not-found page."""
    ...

def py_format_timestamp(timestamp: float) -> str:
    """Formats timestamp as UI date-time."""
    ...

def py_handle_stats(ctx: PyContext, relations: PyRelations, request_uri: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/."""
    ...

def py_get_request_uri(environ: Dict[str, str], ctx: PyContext, relations: PyRelations) -> str:
    """Finds out the request URI."""
    ...

def py_check_existing_relation(ctx: PyContext, relations: PyRelations, request_uri: str) -> PyDoc:
    """Prevents serving outdated data from a relation that has been renamed."""
    ...

def py_handle_no_osm_housenumbers(prefix: str, relation_name: str) -> PyDoc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    ...

def py_handle_no_ref_housenumbers(prefix: str, relation_name: str) -> PyDoc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    ...

def py_handle_github_webhook(stream: BinaryIO, ctx: PyContext) -> PyDoc:
    """Handles a GitHub style webhook."""
    ...

def py_is_missing_housenumbers_html_cached(ctx: PyContext, relation: PyRelation) -> bool:
    """Decides if we have an up to date HTML cache entry or not."""
    ...

def py_get_missing_housenumbers_html(ctx: PyContext, relation: PyRelation) -> PyDoc:
    """Gets the cached HTML of the missing housenumbers for a relation."""
    ...

def py_get_additional_housenumbers_html(ctx: PyContext, relation: PyRelation) -> PyDoc:
    """Gets the cached HTML of the additional housenumbers for a relation."""
    ...

def py_is_missing_housenumbers_txt_cached(ctx: PyContext, relation: PyRelation) -> bool:
    """Decides if we have an up to date plain text cache entry or not."""
    ...

def py_get_missing_housenumbers_txt(ctx: PyContext, relation: PyRelation) -> str:
    """Gets the cached plain text of the missing housenumbers for a relation."""
    ...

def py_our_application_json(
        environ: Dict[str, str],
        ctx: PyContext,
        relations: PyRelations,
        request_uri: str
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Dispatches json requests based on their URIs."""
    ...

def py_additional_streets_view_txt(
    ctx: PyContext,
    relations: PyRelations,
    request_uri: str,
    chkl: bool
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt."""
    ...

def py_additional_streets_view_result(
    ctx: PyContext,
    relations: PyRelations,
    request_uri: str
) -> PyDoc:
    """Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result."""
    ...

def py_additional_housenumbers_view_result(
    ctx: PyContext,
    relations: PyRelations,
    request_uri: str
) -> PyDoc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/budapest_11/view-result."""
    ...

def py_additional_streets_view_turbo(relations: PyRelations, request_uri: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo."""
    ...

def py_handle_streets(ctx: PyContext, relations: PyRelations, request_uri: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/streets/ormezo/view-query."""
    ...

def py_handle_street_housenumbers(ctx: PyContext, relations: PyRelations, request_uri: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    ...

def py_missing_streets_view_result(ctx: PyContext, relations: PyRelations, request_uri: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/missing-streets/budapest_11/view-result."""
    ...

def py_missing_housenumbers_view_txt(ctx: PyContext, relations: PyRelations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.txt."""
    ...

def py_missing_housenumbers_view_chkl(
        ctx: PyContext, relations: PyRelations, request_uri: str
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.chkl."""
    ...

def py_missing_streets_view_txt(
    ctx: PyContext, relations: PyRelations, request_uri: str, chkl: bool
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-result.txt."""
    ...

def py_missing_streets_update(ctx: PyContext, relations: PyRelations, relation_name: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/update-result."""
    ...

def py_handle_missing_housenumbers(ctx: PyContext, relations: PyRelations, request_uri: str) -> PyDoc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-[result|query]."""
    ...

# vim:set shiftwidth=4 softtabstop=4 expandtab:
