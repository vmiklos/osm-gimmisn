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


class PyDoc:
    """Generates xml/html documents."""
    def __init__(self) -> None:
        ...

    def get_value(self) -> str:
        """Gets the escaped value."""
        ...

    def append_value(self, value: str) -> None:
        """Appends escaped content to the value."""
        ...

    def stag(self, name: str, attrs: List[Tuple[str, str]]) -> None:
        """Starts a new tag and closes it as well."""
        ...

    def text(self, text: str) -> None:
        """Appends unescaped content to the document."""
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
    @staticmethod
    def from_string(osm_name: str) -> "PyStreet":
        """Constructor that only requires an OSM name."""
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
    def get_number(self) -> str:
        """Returns the house number string."""
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

def py_get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
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

    def get_streets_percent_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street percent file of a relation for reading."""
        ...

    def get_streets_additional_count_write_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the street additional count file of a relation for writing."""
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

    def get_street_refsettlement(self, street: str) -> List[str]:
        """Returns a list of refsettlement values specific to a street."""
        ...

    def get_street_filters(self) -> List[str]:
        """Gets list of streets which are only in reference, but have to be filtered out."""
        ...

    def get_osm_street_filters(self) -> List[str]:
        """Gets list of streets which are only in OSM, but have to be filtered out."""
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

    def get_osm_streets(self, sorted_result: bool) -> List[PyStreet]:
        """Reads list of streets for an area from OSM."""
        ...

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
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

    def write_additional_streets(self) -> List[PyStreet]:
        """Calculate and write stat for the unexpected street coverage of a relation."""
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

    def get_relations(self) -> List[PyRelation]:
        """Gets a list of relations."""
        ...

def py_missing_housenumbers_main(argv: List[str], stdout: BinaryIO, ctx: PyContext) -> None:
    """Commandline interface."""
    ...

def py_handle_exception(
        environ: Dict[str, str],
        error: str
) -> Tuple[str, List[Tuple[str, str]], bytes]:
    """Displays an unhandled exception on the page."""
    ...

def py_get_request_uri(environ: Dict[str, str], ctx: PyContext, relations: PyRelations) -> str:
    """Finds out the request URI."""
    ...

def py_handle_no_osm_housenumbers(prefix: str, relation_name: str) -> PyDoc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    ...

def py_handle_no_ref_housenumbers(prefix: str, relation_name: str) -> PyDoc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    ...

def py_handle_github_webhook(data: bytes, ctx: PyContext) -> PyDoc:
    """Handles a GitHub style webhook."""
    ...

def py_is_missing_housenumbers_txt_cached(ctx: PyContext, relation: PyRelation) -> bool:
    """Decides if we have an up to date plain text cache entry or not."""
    ...

def py_get_missing_housenumbers_txt(ctx: PyContext, relation: PyRelation) -> str:
    """Gets the cached plain text of the missing housenumbers for a relation."""
    ...

def py_handle_main_housenr_additional_count(ctx: PyContext, relation: PyRelation) -> PyDoc:
    """Handles the housenumber additional count part of the main page."""
    ...

def py_application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: PyContext
) -> Tuple[str, List[Tuple[str, str]], bytes]:
    """The entry point of this WSGI app."""
    ...

def py_get_topcities(ctx: PyContext, src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    ...

def py_setup_logging(ctx: PyContext) -> None:
    """Sets up logging."""
    ...

def py_update_osm_streets(ctx: PyContext, relations: PyRelations, update: bool) -> None:
    """Update the OSM street list of all relations."""
    ...

def py_update_stats_count(ctx: PyContext, today: str) -> None:
    """Counts the # of all house numbers as of today."""
    ...

def py_update_stats_topusers(ctx: PyContext, today: str) -> None:
    """Counts the top housenumber editors as of today."""
    ...

def py_update_stats(ctx: PyContext, overpass: bool) -> None:
    """Performs the update of country-level stats."""
    ...

def py_our_main(ctx: PyContext, relations: PyRelations, mode: str, update: bool, overpass: bool) -> None:
    """Performs the actual nightly task."""
    ...

def py_cron_main(argv: List[str], stdout: BinaryIO, ctx: PyContext) -> None:
    """Commandline interface."""
    ...

# vim:set shiftwidth=4 softtabstop=4 expandtab:
