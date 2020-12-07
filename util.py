#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

from enum import Enum
from typing import Any
from typing import Callable
from typing import Dict
from typing import Iterable
from typing import Iterator
from typing import List
from typing import Optional
from typing import Sequence
from typing import Set
from typing import TextIO
from typing import Tuple
from typing import cast
import csv
import locale
import os
import pickle
import re
import urllib.error

import yattag

from i18n import translate as _
import accept_language
import config
import i18n
import overpass_query
import ranges


class LetterSuffixStyle(Enum):
    """Specifies the style of the output of normalize_letter_suffix()."""

    # "42/A"
    UPPER = 1
    # "42a"
    LOWER = 2


class HouseNumberRange:
    """
    A house number range is a string that may expand to one or more HouseNumber instances in the
    future. It can also have a comment.
    """
    def __init__(self, number: str, comment: str) -> None:
        self.__number = number
        self.__comment = comment

    def get_number(self) -> str:
        """Returns the house number (range) string."""
        return self.__number

    def get_comment(self) -> str:
        """Returns the comment."""
        return self.__comment

    def __repr__(self) -> str:
        return "HouseNumberRange(number=%s, comment=%s)" % (self.__number, self.__comment)

    def __eq__(self, other: object) -> bool:
        """Comment is explicitly non-interesting."""
        other_house_number_range = cast(HouseNumberRange, other)
        return self.__number == other_house_number_range.get_number()

    def __lt__(self, other: object) -> bool:
        """Comment is explicitly non-interesting."""
        other_house_number_range = cast(HouseNumberRange, other)
        return self.__number < other_house_number_range.get_number()

    def __hash__(self) -> int:
        """Comment is explicitly non-interesting."""
        return hash(self.__number)


class Street:
    """
    A street has an OSM and a reference name. Ideally the two are the same. Sometimes the reference
    name differs.
    """
    def __init__(self, osm_name: str, ref_name: str = "", show_ref_street: bool = True, osm_id: int = 0) -> None:
        self.__osm_name = osm_name
        self.__ref_name = ref_name
        self.__show_ref_street = show_ref_street
        self.__osm_id = osm_id

    def get_osm_name(self) -> str:
        """Returns the OSM name."""
        return self.__osm_name

    def get_ref_name(self) -> str:
        """Returns the reference name."""
        return self.__ref_name

    def get_osm_id(self) -> int:
        """Returns the OSM (way) id."""
        return self.__osm_id

    def to_html(self) -> yattag.doc.Doc:
        """Writes the street as a HTML string."""
        doc = yattag.doc.Doc()
        doc.text(self.__osm_name)
        if self.__osm_name != self.__ref_name and self.__show_ref_street:
            doc.stag("br")
            doc.text("(")
            doc.text(self.__ref_name)
            doc.text(")")
        return doc

    def __repr__(self) -> str:
        return "Street(osm_name=%s, ref_name=%s)" % (self.__osm_name, self.__ref_name)

    def __eq__(self, other: object) -> bool:
        """OSM id is explicitly non-interesting."""
        other_street = cast(Street, other)
        return self.__osm_name == other_street.get_osm_name()

    def __lt__(self, other: object) -> bool:
        """OSM id is explicitly non-interesting."""
        other_street = cast(Street, other)
        return self.__osm_name < other_street.get_osm_name()

    def __hash__(self) -> int:
        """OSM id is explicitly not interesting."""
        return hash(self.__osm_name)


class HouseNumber:
    """
    A house number is a string which remembers what was its provider range.  E.g. the "1-3" string
    can generate 3 house numbers, all of them with the same range.
    The comment is similar to source, it's ignored during __eq__() and __hash__().
    """
    def __init__(self, number: str, source: str, comment: str = "") -> None:
        self.__number = number
        self.__source = source
        self.__comment = comment

    def get_number(self) -> str:
        """Returns the house number string."""
        return self.__number

    def get_source(self) -> str:
        """Returns the source range."""
        return self.__source

    def get_comment(self) -> str:
        """Returns the comment."""
        return self.__comment

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
        # Check for letter suffix.
        match = re.match(r".*([A-Za-z]+)\*?", house_number)
        if match:
            suffix = match.group(1).lower()
        else:
            # If not, then try digit suggfix, but then only '/' is OK as a separator.
            match = re.match(r"^.*/([0-9])\*?$", house_number)
            if match:
                suffix = "/" + match.group(1)

        house_number = number + suffix
        return house_number in invalids

    @staticmethod
    def has_letter_suffix(house_number: str, source_suffix: str) -> bool:
        """
        Determines if the input is a house number, allowing letter suffixes. This means not only
        '42' is allowed, but also '42a', '42/a' and '42 a'. Everything else is still considered just
        junk after the numbers.
        """
        if source_suffix:
            house_number = house_number[0:-len(source_suffix)]
        # Check for letter suffix.
        letter_match = bool(re.match(r"^([0-9]+)( |/)?[A-Za-z]$", house_number))
        if letter_match:
            return True
        # If not, then try digit suggfix, but then only '/' is OK as a separator.
        return bool(re.match(r"^([0-9]+)/[0-9]$", house_number))

    @staticmethod
    def normalize_letter_suffix(house_number: str, source_suffix: str, style: LetterSuffixStyle) -> str:
        """
        Turn '42A' and '42 A' (and their lowercase versions) into '42/A'.
        """
        if source_suffix:
            house_number = house_number[0:-len(source_suffix)]
        # Check for letter suffix.
        match = re.match(r"^([0-9]+)( |/)?([A-Za-z])$", house_number)
        digit_match = False
        if not match:
            # If not, then try digit suggfix, but then only '/' is OK as a separator.
            match = re.match(r"^([0-9]+)(/)([0-9])$", house_number)
            digit_match = True
            if not match:
                raise ValueError
        groups = match.groups()
        if style == LetterSuffixStyle.UPPER or digit_match:
            return groups[0] + "/" + groups[2].upper() + source_suffix
        return groups[0] + groups[2].lower() + source_suffix


class CsvIO:
    """Like TextIO, but for CSV reading."""
    def __init__(self, stream: TextIO) -> None:
        self.stream = stream
        self.reader = csv.reader(stream, delimiter='\t', quotechar='"')

    def __enter__(self) -> 'CsvIO':
        return self

    def __exit__(self, _exc_type: Any, _exc_value: Any, _exc_traceback: Any) -> bool:
        self.stream.close()
        return True

    def get_rows(self) -> Iterator[List[str]]:
        """Gets access to the rows of the CSV."""
        return self.reader


def split_house_number_range(house_number: HouseNumberRange) -> Tuple[int, str]:
    """Wrapper around split_house_number() for HouseNumberRange objects."""
    return split_house_number(house_number.get_number())


def format_even_odd(only_in_ref: List[HouseNumberRange], doc: Optional[yattag.doc.Doc]) -> List[str]:
    """Separate even and odd numbers, this helps survey in most cases."""
    key = split_house_number_range
    even = sorted([i for i in only_in_ref if int(split_house_number(i.get_number())[0]) % 2 == 0], key=key)
    odd = sorted([i for i in only_in_ref if int(split_house_number(i.get_number())[0]) % 2 == 1], key=key)
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

    even_string = ", ".join([i.get_number() for i in even])
    odd_string = ", ".join([i.get_number() for i in odd])
    elements = []
    if odd_string:
        elements.append(odd_string)
    if even_string:
        elements.append(even_string)
    return elements


def color_house_number(house_number: HouseNumberRange) -> yattag.doc.Doc:
    """Colors a house number according to its suffix."""
    doc = yattag.doc.Doc()
    number = house_number.get_number()
    if not number.endswith("*"):
        doc.text(number)
        return doc
    number = number[:-1]
    title = house_number.get_comment().replace("&#013;", "\n")
    with doc.tag("span", style="color: blue;"):
        if title:
            with doc.tag("abbr", title=title, tabindex="0"):
                doc.text(number)
        else:
            doc.text(number)
    return doc


def build_street_reference_cache(local_streets: str) -> Dict[str, Dict[str, List[str]]]:
    """Builds an in-memory cache from the reference on-disk TSV (street version)."""
    memory_cache: Dict[str, Dict[str, List[str]]] = {}

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

            refcounty, refsettlement, street = line.strip().split("\t")
            # Filter out invalid street type.
            street = re.sub(" null$", "", street)
            if refcounty not in memory_cache.keys():
                memory_cache[refcounty] = {}
            if refsettlement not in memory_cache[refcounty].keys():
                memory_cache[refcounty][refsettlement] = []
            memory_cache[refcounty][refsettlement].append(street)
    with open(disk_cache, "wb") as sock_cache:
        pickle.dump(memory_cache, sock_cache)
    return memory_cache


def get_reference_cache_path(local: str, refcounty: str) -> str:
    """Gets the filename of the (house number) reference cache file."""
    return local + "-" + refcounty + "-v1.pickle"


def build_reference_cache(local: str, refcounty: str) -> Dict[str, Dict[str, Dict[str, List[HouseNumberRange]]]]:
    """Builds an in-memory cache from the reference on-disk TSV (house number version)."""
    memory_cache: Dict[str, Dict[str, Dict[str, List[HouseNumberRange]]]] = {}

    disk_cache = get_reference_cache_path(local, refcounty)
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

            if not line.startswith(refcounty):
                continue

            tokens = line.strip().split("\t")
            refcounty, refsettlement, street, num = tokens[0], tokens[1], tokens[2], tokens[3]
            comment = ""
            if len(tokens) >= 5:
                comment = tokens[4]
            if refcounty not in memory_cache.keys():
                memory_cache[refcounty] = {}
            if refsettlement not in memory_cache[refcounty].keys():
                memory_cache[refcounty][refsettlement] = {}
            if street not in memory_cache[refcounty][refsettlement].keys():
                memory_cache[refcounty][refsettlement][street] = []
            memory_cache[refcounty][refsettlement][street].append(HouseNumberRange(num, comment))
    with open(disk_cache, "wb") as sock_cache:
        pickle.dump(memory_cache, sock_cache)
    return memory_cache


def build_reference_caches(
        references: List[str],
        refcounty: str
) -> List[Dict[str, Dict[str, Dict[str, List[HouseNumberRange]]]]]:
    """Handles a list of references for build_reference_cache()."""
    return [build_reference_cache(reference, refcounty) for reference in references]


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
    """Parses a filter description, like 'filter-for', 'refcounty', '42'."""
    ret: Dict[str, str] = {}
    filter_for = False
    for index, value in enumerate(tokens):
        if value == "filter-for":
            filter_for = True
            continue

        if not filter_for:
            continue

        if value == "incomplete":
            ret[value] = ""
        elif value == "everything":
            ret[value] = ""

        if index + 1 >= len(tokens):
            continue

        if value in ("refcounty", "refsettlement", "relations"):
            ret[value] = tokens[index + 1]

    return ret


def html_escape(text: str) -> yattag.doc.Doc:
    """Factory of yattag.doc.Doc from a string."""
    doc = yattag.doc.Doc()
    doc.text(text)
    return doc


def handle_overpass_error(http_error: urllib.error.HTTPError) -> yattag.doc.Doc:
    """Handles a HTTP error from Overpass."""
    doc = yattag.doc.Doc()
    with doc.tag("div", id="overpass-error"):
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


def gen_link(url: str, label: str) -> yattag.doc.Doc:
    """Generates a link to a URL with a given label."""
    doc = yattag.doc.Doc()
    with doc.tag("a", href=url):
        doc.text(label + "...")

    # Always auto-visit the link for now.
    with doc.tag("script", type="text/javascript"):
        doc.text("window.location.href = \"%s\";" % url)

    return doc


def write_html_header(doc: yattag.doc.Doc) -> None:
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


def html_table_from_list(table: List[List[yattag.doc.Doc]]) -> yattag.doc.Doc:
    """Produces a HTML table from a list of lists."""
    doc = yattag.doc.Doc()
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


def tsv_to_list(stream: CsvIO) -> List[List[yattag.doc.Doc]]:
    """Turns a tab-separated table into a list of lists."""
    table = []

    first = True
    type_index = 0
    for row in stream.get_rows():
        if not row:
            continue
        if first:
            first = False
            for index, column in enumerate(row):
                if column.strip() == "@type":
                    type_index = index
        cells = [html_escape(cell.strip()) for cell in row]
        if cells and type_index:
            # We know the first column is an OSM ID.
            try:
                osm_id = int(cells[0].getvalue())
                osm_type = cells[type_index].getvalue()
                doc = yattag.doc.Doc()
                href = "https://www.openstreetmap.org/{}/{}".format(osm_type, osm_id)
                with doc.tag("a", href=href, target="_blank"):
                    doc.text(str(osm_id))
                cells[0] = doc
            except ValueError:
                # Not an int, ignore.
                pass
        table.append(cells)

    return table


def get_nth_column(
    sock: CsvIO, street_column: int, housenumber_column: int, conscriptionnumber_column: int
) -> List[str]:
    """Reads the content from sock, interprets its content as tab-separated values, finally returns
    the values of the nth column. If a row has less columns, that's silently ignored."""
    ret = []

    first = True
    for row in sock.get_rows():
        if first:
            first = False
            continue

        has_housenumber = housenumber_column < len(row) and row[housenumber_column]
        has_conscriptionnumber = conscriptionnumber_column < len(row) and row[conscriptionnumber_column]
        if len(row) < street_column + 1 or ((not has_housenumber) and (not has_conscriptionnumber)):
            continue

        ret.append(row[street_column])

    return ret


def get_housenumber_ranges(house_numbers: List[HouseNumber]) -> List[HouseNumberRange]:
    """Gets a reference range list for a house number list by looking at what range provided a givne
    house number."""
    ret = []
    for house_number in house_numbers:
        ret.append(HouseNumberRange(house_number.get_source(), house_number.get_comment()))
    return sorted(set(ret))


def git_link(version: str, prefix: str) -> yattag.doc.Doc:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    commit_hash = re.sub(".*-g", "", version)
    doc = yattag.doc.Doc()
    with doc.tag("a", href=prefix + commit_hash):
        doc.text(version)
    return doc


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


def sort_housenumbers_csv(data: str) -> str:
    """
    Sorts TSV Overpass house numbers result with visual partitioning.

    See split_housenumber_line for sorting rules.
    """
    return process_csv_body(sort_housenumbers, data)


def get_only_in_first(first: List[Any], second: List[Any]) -> List[Any]:
    """
    Returns items which are in first, but not in second.
    Any means HouseNumber or str.
    """
    # Strip suffix that is ignored.
    if not first:
        return []

    if isinstance(first[0], HouseNumber):
        first_stripped = [re.sub(r"\*$", "", i.get_number()) for i in first]
        second_stripped = [re.sub(r"\*$", "", i.get_number()) for i in second]
    elif isinstance(first[0], Street):
        first_stripped = [re.sub(r"\*$", "", i.get_osm_name()) for i in first]
        second_stripped = [re.sub(r"\*$", "", i.get_osm_name()) for i in second]
    else:
        first_stripped = [re.sub(r"\*$", "", i) for i in first]
        second_stripped = [re.sub(r"\*$", "", i) for i in second]

    ret = []
    for index, item in enumerate(first_stripped):
        if item not in second_stripped:
            ret.append(first[index])
    return ret


def get_in_both(first: List[Any], second: List[Any]) -> List[Any]:
    """
    Returns items which are in both first and second.
    Any means HouseNumber or str.
    """
    # Strip suffix that is ignored.
    if not first:
        return []

    if isinstance(first[0], HouseNumber):
        first_stripped = [re.sub(r"\*$", "", i.get_number()) for i in first]
        second_stripped = [re.sub(r"\*$", "", i.get_number()) for i in second]
    else:
        first_stripped = [re.sub(r"\*$", "", i) for i in first]
        second_stripped = [re.sub(r"\*$", "", i) for i in second]

    ret = []
    for index, item in enumerate(first_stripped):
        if item in second_stripped:
            ret.append(first[index])
    return ret


def get_content(workdir: str, path: str = "") -> str:
    """Gets the content of a file in workdir."""
    ret = ""
    if path:
        path = os.path.join(workdir, path)
    else:
        path = workdir
    with open(path) as sock:
        ret = sock.read()
    return ret


def get_normalizer(street_name: str, normalizers: Dict[str, ranges.Ranges]) -> ranges.Ranges:
    """Determines the normalizer for a given street."""
    if street_name in normalizers.keys():
        # Have a custom filter.
        normalizer = normalizers[street_name]
    else:
        # Default sanity checks.
        default = [ranges.Range(1, 999), ranges.Range(2, 998)]
        normalizer = ranges.Ranges(default)
    return normalizer


def split_house_number_by_separator(
        house_numbers: str,
        separator: str,
        normalizer: ranges.Ranges
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


def set_locale() -> None:
    """Sets the locale of this Python process automatically based on config, with a UTF-8
    default."""
    if config.Config.has_value("locale"):
        ui_locale = config.Config.get_locale()
    else:
        ui_locale = "hu_HU.UTF-8"
    try:
        locale.setlocale(locale.LC_ALL, ui_locale)
    except locale.Error:
        # Ignore, this happens only on the cut-down CI environment.
        pass


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


def get_valid_settlements() -> Set[str]:
    """Builds a set of valid settlement names."""
    settlements: Set[str] = set()

    with open(config.Config.get_reference_citycounts_path(), "r") as stream:
        first = True
        for line in stream.readlines():
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
    decimal_point = decimal_points.get(i18n.get_language(), ".")
    return formatted.replace(".", str(decimal_point))


# vim:set shiftwidth=4 softtabstop=4 expandtab:
