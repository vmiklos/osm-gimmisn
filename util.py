#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains free functions shared between other modules."""

from typing import Any
from typing import Callable
from typing import Dict
from typing import List
from typing import Iterable
from typing import Optional
from typing import Sequence
from typing import TextIO
from typing import Tuple
from typing import cast
import os
import pickle
import re
import urllib.error
from enum import Enum

import yattag  # type: ignore

import accept_language
from i18n import translate as _
import i18n
import overpass_query


class LetterSuffixStyle(Enum):
    """Specifies the style of the output of normalize_letter_suffix()."""

    # "42/A"
    UPPER = 1
    # "42a"
    LOWER = 2


class HouseNumber:
    """
    A house number is a string which remembers what was its provider range.  E.g. the "1-3" string
    can generate 3 house numbers, all of them with the same range.
    """
    def __init__(self, number: str, source: str) -> None:
        self.__number = number
        self.__source = source

    def get_number(self) -> str:
        """Returns the house number string."""
        return self.__number

    def get_source(self) -> str:
        """Returns the source range."""
        return self.__source

    def __repr__(self) -> str:
        return "HouseNumber(number=%s, source=%s)" % (self.__number, self.__source)

    def __eq__(self, other: object) -> bool:
        """Source is explicitly non-interesting."""
        other_house_number = cast(HouseNumber, other)
        return self.__number == other_house_number.get_number()

    def __hash__(self) -> int:
        """Source is explicitly non-interesting."""
        return hash(self.__number)

    @staticmethod
    def is_invalid(house_number: str, invalids: List[str]) -> bool:
        """Decides if house_number is invalid according to invalids."""
        if house_number in invalids:
            return True

        number = ""
        match = re.match(r"([0-9]+).*", house_number)
        if match:
            number = match.group(1)
        suffix = ""
        match = re.match(r".*([A-Za-z]+)", house_number)
        if match:
            suffix = match.group(1).lower()

        house_number = number + suffix
        return house_number in invalids

    @staticmethod
    def has_letter_suffix(house_number: str) -> bool:
        """
        Determines if the input is a house number, allowing letter suffixes. This means not only
        '42' is allowed, but also '42a', '42/a' and '42 a'. Everything else is still considered just
        junk after the numbers.
        """
        return bool(re.match(r"^([0-9]+)( |/)?[A-Za-z]$", house_number))

    @staticmethod
    def normalize_letter_suffix(house_number: str, style: LetterSuffixStyle) -> str:
        """
        Turn '42A' and '42 A' (and their lowercase versions) into '42/A'.
        """
        match = re.match(r"^([0-9]+)( |/)?([A-Za-z])$", house_number)
        if not match:
            raise ValueError
        groups = match.groups()
        if style == LetterSuffixStyle.UPPER:
            return groups[0] + "/" + groups[2].upper()
        return groups[0] + groups[2].lower()


def format_even_odd(only_in_ref: List[str], doc: Optional[yattag.Doc]) -> List[str]:
    """Separate even and odd numbers, this helps survey in most cases."""
    key = split_house_number
    even = sorted([i for i in only_in_ref if int(split_house_number(i)[0]) % 2 == 0], key=key)
    odd = sorted([i for i in only_in_ref if int(split_house_number(i)[0]) % 2 == 1], key=key)
    if doc:
        if odd:
            for index, elem in enumerate(odd):
                if index:
                    doc.text(", ")
                doc.asis(color_house_number(elem).getvalue())
        if even:
            if odd:
                doc.stag("br")
            for index, elem in enumerate(even):
                if index:
                    doc.text(", ")
                doc.asis(color_house_number(elem).getvalue())
        return []

    even_string = ", ".join(even)
    odd_string = ", ".join(odd)
    elements = []
    if odd_string:
        elements.append(odd_string)
    if even_string:
        elements.append(even_string)
    return elements


def color_house_number(fro: str) -> yattag.Doc:
    """Colors a house number according to its suffix."""
    doc = yattag.Doc()
    if not fro.endswith("*"):
        doc.text(fro)
        return doc
    with doc.tag("span", style="color: blue;"):
        doc.text(fro[:-1])
    return doc


def build_street_reference_cache(local_streets: str) -> Dict[str, Dict[str, List[str]]]:
    """Builds an in-memory cache from the reference on-disk TSV (street version)."""
    memory_cache = {}  # type: Dict[str, Dict[str, List[str]]]

    disk_cache = local_streets + ".pickle"
    if os.path.exists(disk_cache):
        with open(disk_cache, "rb") as sock_cache:
            memory_cache = pickle.load(sock_cache)
            return memory_cache

    with open(local_streets, "r") as sock:
        first = True
        while True:
            line = sock.readline()
            if first:
                first = False
                continue

            if not line:
                break

            refmegye, reftelepules, street = line.strip().split("\t")
            # Filter out invalid street type.
            street = re.sub(" null$", "", street)
            if refmegye not in memory_cache.keys():
                memory_cache[refmegye] = {}
            if reftelepules not in memory_cache[refmegye].keys():
                memory_cache[refmegye][reftelepules] = []
            memory_cache[refmegye][reftelepules].append(street)
    with open(disk_cache, "wb") as sock_cache:
        pickle.dump(memory_cache, sock_cache)
    return memory_cache


def build_reference_cache(local: str) -> Dict[str, Dict[str, Dict[str, List[str]]]]:
    """Builds an in-memory cache from the reference on-disk TSV (house number version)."""
    memory_cache = {}  # type: Dict[str, Dict[str, Dict[str, List[str]]]]

    disk_cache = local + ".pickle"
    if os.path.exists(disk_cache):
        with open(disk_cache, "rb") as sock_cache:
            memory_cache = pickle.load(sock_cache)
            return memory_cache

    with open(local, "r") as sock:
        first = True
        while True:
            line = sock.readline()
            if first:
                first = False
                continue

            if not line:
                break

            refmegye, reftelepules, street, num = line.strip().split("\t")
            if refmegye not in memory_cache.keys():
                memory_cache[refmegye] = {}
            if reftelepules not in memory_cache[refmegye].keys():
                memory_cache[refmegye][reftelepules] = {}
            if street not in memory_cache[refmegye][reftelepules].keys():
                memory_cache[refmegye][reftelepules][street] = []
            memory_cache[refmegye][reftelepules][street].append(num)
    with open(disk_cache, "wb") as sock_cache:
        pickle.dump(memory_cache, sock_cache)
    return memory_cache


def build_reference_caches(references: List[str]) -> List[Dict[str, Dict[str, Dict[str, List[str]]]]]:
    """Handles a list of references for build_reference_cache()."""
    return [build_reference_cache(reference) for reference in references]


def split_house_number(house_number: str) -> Tuple[int, str]:
    """Splits house_number into a numerical and a remainder part."""
    match = re.search(r"^([0-9]*)([^0-9].*|)$", house_number)
    if not match:  # pragma: no cover
        return (0, '')
    number = 0
    try:
        number = int(match.group(1))
    except ValueError:
        pass
    return (number, match.group(2))


def parse_filters(tokens: List[str]) -> Dict[str, str]:
    """Parses a filter description, like 'filter-for', 'refmegye', '42'."""
    ret = {}  # type: Dict[str, str]
    filter_for = False
    for index, value in enumerate(tokens):
        if value == "filter-for":
            filter_for = True
            continue

        if not filter_for:
            continue

        if value == "incomplete":
            ret[value] = ""

        if index + 1 >= len(tokens):
            continue

        if value in ("refmegye", "reftelepules"):
            ret[value] = tokens[index + 1]

    return ret


def html_escape(text: str) -> yattag.Doc:
    """Factory of yattag.Doc from a string."""
    doc = yattag.Doc()
    doc.text(text)
    return doc


def handle_overpass_error(http_error: urllib.error.HTTPError) -> yattag.Doc:
    """Handles a HTTP error from Overpass."""
    doc = yattag.Doc()
    doc.text(_("Overpass error: {0}").format(str(http_error)))
    sleep = overpass_query.overpass_query_need_sleep()
    if sleep:
        doc.stag("br")
        doc.text(_("Note: wait for {} seconds").format(sleep))
    return doc


def setup_localization(environ: Dict[str, Any]) -> str:
    """Provides localized strings for this thread."""
    # Set up localization.
    languages = environ.get("HTTP_ACCEPT_LANGUAGE")
    if languages:
        parsed = accept_language.parse_accept_language(languages)
        if parsed:
            language = parsed[0].language
            i18n.set_language(language)
            return cast(str, language)
    return ""


def gen_link(url: str, label: str) -> yattag.Doc:
    """Generates a link to a URL with a given label."""
    doc = yattag.Doc()
    with doc.tag("a", href=url):
        doc.text(label + "...")

    # Always auto-visit the link for now.
    with doc.tag("script", type="text/javascript"):
        doc.text("window.location.href = \"%s\";" % url)

    return doc


def write_html_header(doc: yattag.Doc) -> None:
    """Produces the verify first line of a HTML output."""
    doc.asis("<!DOCTYPE html>\n")


def process_template(buf: str, osmrelation: int) -> str:
    """Turns an overpass query template to an actual query."""
    buf = buf.replace("@RELATION@", str(osmrelation))
    # area is relation + 3600000000 (3600000000 == relation), see js/ide.js
    # in https://github.com/tyrasd/overpass-turbo
    buf = buf.replace("@AREA@", str(3600000000 + osmrelation))
    return buf


def should_expand_range(numbers: List[int], street_is_even_odd: bool) -> bool:
    """Decides if an x-y range should be expanded."""
    if len(numbers) != 2:
        return False

    if numbers[1] < numbers[0]:
        # E.g. 42-1, -1 is just a suffix to be ignored.
        numbers[1] = 0
        return True

    # If there is a parity mismatch, ignore.
    if street_is_even_odd and numbers[0] % 2 != numbers[1] % 2:
        return False

    # Assume that 0 is just noise.
    if numbers[0] == 0:
        return False

    # Ranges larger than this are typically just noise in the input data.
    if numbers[1] > 1000 or numbers[1] - numbers[0] > 24:
        return False

    return True


def html_table_from_list(table: List[List[yattag.Doc]]) -> yattag.Doc:
    """Produces a HTML table from a list of lists."""
    doc = yattag.Doc()
    with doc.tag("table", klass="sortable"):
        for row_index, row_content in enumerate(table):
            with doc.tag("tr"):
                for cell in row_content:
                    if row_index == 0:
                        with doc.tag("th"):
                            with doc.tag("a", href="#"):
                                doc.text(cell.getvalue())
                    else:
                        with doc.tag("td"):
                            doc.asis(cell.getvalue())
    return doc


def tsv_to_list(stream: TextIO) -> List[List[yattag.Doc]]:
    """Turns a tab-separated table into a list of lists."""
    table = []

    first = True
    type_index = 0
    for line in stream.readlines():
        if not line.strip():
            continue
        if first:
            first = False
            for index, column in enumerate(line.split("\t")):
                if column.strip() == "@type":
                    type_index = index
        cells = [html_escape(cell.strip()) for cell in line.split("\t")]
        if cells and type_index:
            # We know the first column is an OSM ID.
            try:
                osm_id = int(cells[0].getvalue())
                osm_type = cells[type_index].getvalue()
                doc = yattag.Doc()
                href = "https://www.openstreetmap.org/{}/{}".format(osm_type, osm_id)
                with doc.tag("a", href=href, target="_blank"):
                    doc.text(osm_id)
                cells[0] = doc
            except ValueError:
                # Not an int, ignore.
                pass
        table.append(cells)

    return table


def get_nth_column(sock: TextIO, column: int) -> List[str]:
    """Reads the content from sock, interprets its content as tab-separated values, finally returns
    the values of the nth column. If a row has less columns, that's silently ignored."""
    ret = []

    first = True
    for line in sock.readlines():
        if first:
            first = False
            continue

        tokens = line.strip().split('\t')
        if len(tokens) < column + 1:
            continue

        ret.append(tokens[column])

    return ret


def get_housenumber_ranges(house_numbers: List[HouseNumber]) -> List[str]:
    """Gets a reference range list for a house number list by looking at what range provided a givne
    house number."""
    ret = []
    for house_number in house_numbers:
        ret.append(house_number.get_source())
    return sorted(set(ret))


def git_link(version: str, prefix: str) -> yattag.Doc:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    commit_hash = re.sub(".*-g", "", version)
    doc = yattag.Doc()
    with doc.tag("a", href=prefix + commit_hash):
        doc.text(version)
    return doc


def get_abspath(path: str) -> str:
    """Make a path absolute, taking the repo root as a base dir."""
    if os.path.isabs(path):
        return path

    return os.path.join(os.path.dirname(__file__), path)


def sort_numerically(strings: Iterable[HouseNumber]) -> List[HouseNumber]:
    """Sorts strings according to their numerical value, not alphabetically."""
    return sorted(strings, key=lambda x: split_house_number(x.get_number()))


def process_csv_body(fun: Callable[[Iterable[str]], List[str]], data: str) -> str:
    """
    Process the body of a CSV/TSV with the given function while keeping the header intact.
    """
    lines = data.split('\n')
    header = lines[0] if lines else ''
    body = lines[1:] if lines else ''
    result = [header] + fun(body)
    return '\n'.join(result)


def get_array_nth(arr: Sequence[str], index: int) -> str:
    """Gets the nth element of arr, returns en empty string on error."""
    return arr[index] if len(arr) > index else ''


def split_street_line(line: str) -> Tuple[bool, str, str, str, Tuple[int, str]]:
    """
    Augment TSV Overpass street name result lines to aid sorting.

    It prepends a bool to indicate whether the street is missing a name, thus
    streets with missing names are ordered last.
    oid is interpreted numerically while other fields are taken alphabetically.
    """
    field = line.split('\t')
    oid = get_array_nth(field, 0)
    name = get_array_nth(field, 1)
    highway = get_array_nth(field, 2)
    service = get_array_nth(field, 3)
    missing_name = name == ''
    return (missing_name, name, highway, service, split_house_number(oid))


def sort_streets(lines: Iterable[str]) -> List[str]:
    """
    Sorts the body of a TSV Overpass street name result with visual partitioning.

    See split_street_line for sorting rules.
    """
    return sorted(lines, key=split_street_line)


def sort_streets_csv(data: str) -> str:
    """
    Sorts TSV Overpass street name result with visual partitioning.

    See split_street_line for sorting rules.
    """
    return process_csv_body(sort_streets, data)


def split_housenumber_line(line: str) -> Tuple[str, bool, bool, str, Tuple[int, str], str,
                                               Tuple[int, str], Iterable[str], Tuple[int, str]]:
    """
    Augment TSV Overpass house numbers result lines to aid sorting.

    It prepends two bools to indicate whether an entry is missing either a house number, a house name
    or a conscription number.
    Entries lacking either a house number or all of the above IDs come first.
    The following fields are interpreted numerically: oid, house number, conscription number.
    """
    field = line.split('\t')

    oid = get_array_nth(field, 0)
    street = get_array_nth(field, 1)
    housenumber = get_array_nth(field, 2)
    postcode = get_array_nth(field, 3)
    housename = get_array_nth(field, 4)
    cons = get_array_nth(field, 5)
    tail = field[6:] if len(field) > 6 else []

    have_housenumber = housenumber != ''
    have_houseid = have_housenumber or housename != '' or cons != ''
    return (postcode, have_houseid, have_housenumber, street,
            split_house_number(housenumber),
            housename, split_house_number(cons), tail, split_house_number(oid))


def sort_housenumbers(lines: Iterable[str]) -> List[str]:
    """
    Sorts the body of a TSV Overpass house numbers result with visual partitioning.

    See split_housenumber_line for sorting rules.
    """
    return sorted(lines, key=split_housenumber_line)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
