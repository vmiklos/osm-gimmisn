#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The helpers module contains functionality shared between other modules."""

import re
import os
import hashlib
from typing import Callable, Iterable, List, Sequence, Tuple


class Range:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start, end):
        self.start = start
        self.end = end
        self.isOdd = start % 2 == 1

    def __contains__(self, n):
        if self.isOdd != (n % 2 == 1):
            return False
        if self.start <= n <= self.end:
            return True
        return False


class Ranges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items):
        self.items = items

    def __contains__(self, item):
        for i in self.items:
            if item in i:
                return True
        return False


def sort_numerically(strings: Iterable[str]) -> List[str]:
    """Sorts strings according to their numerical value, not alphabetically."""
    return sorted(strings, key=split_house_number)


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


def sort_streets_csv(data: str) -> str:
    """
    Sorts TSV Overpass street name result with visual partitioning.

    See split_street_line for sorting rules.
    """
    return process_csv_body(sort_streets, data)


def sort_streets(lines: Iterable[str]) -> List[str]:
    """
    Sorts the body of a TSV Overpass street name result with visual partitioning.

    See split_street_line for sorting rules.
    """
    return sorted(lines, key=split_street_line)


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


def process_csv_body(fun: Callable[[Iterable[str]], List[str]], data: str) -> str:
    """
    Process the body of a CSV/TSV with the given function while keeping the header intact.
    """
    lines = data.split('\n')
    header = lines[0] if lines else ''
    body = lines[1:] if lines else ''
    result = [header] + fun(body)
    return '\n'.join(result)


def sort_housenumbers_csv(data: str) -> str:
    """
    Sorts TSV Overpass house numbers result with visual partitioning.

    See split_housenumber_line for sorting rules.
    """
    return process_csv_body(sort_housenumbers, data)


def sort_housenumbers(lines: Iterable[str]) -> List[str]:
    """
    Sorts the body of a TSV Overpass house numbers result with visual partitioning.

    See split_housenumber_line for sorting rules.
    """
    return sorted(lines, key=split_housenumber_line)


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


def get_array_nth(arr: Sequence[str], n: int) -> str:
    """Gets the nth element of arr, returns en empty string on error."""
    return arr[n] if len(arr) > n else ''


def simplify(s: str, spaceDecode: bool = False) -> str:
    """ Handles normalization of a street name."""
    s = s.replace('Á', 'A').replace('á', 'a')
    s = s.replace('É', 'E').replace('é', 'e')
    s = s.replace('Í', 'I').replace('í', 'i')
    s = s.replace('Ó', 'O').replace('ó', 'o')
    s = s.replace('Ö', 'O').replace('ö', 'o')
    s = s.replace('Ő', 'O').replace('ő', 'o')
    s = s.replace('Ú', 'U').replace('ú', 'u')
    s = s.replace('Ü', 'U').replace('ü', 'u')
    s = s.replace('Ű', 'U').replace('ű', 'u')
    s = s.replace('.', '')
    if spaceDecode:
        s = s.replace(' ', '%20')
    else:
        s = s.replace(' ', '_')
    s = s.lower()
    return s


def get_only_in_first(first, second):
    """Returns items which are in first, but not in second."""
    ret = []
    for i in first:
        if i not in second:
            ret.append(i)
    return ret


def get_in_both(first, second):
    """Returns items which are in both first and second."""
    ret = []
    for i in first:
        if i in second:
            ret.append(i)
    return ret


def git_link(version: str, prefix: str) -> str:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    commit_hash = re.sub(".*-g", "", version)
    return "<a href=\"" + prefix + commit_hash + "\">" + version + "</a>"


def get_nth_column(path: str, column: int) -> List[str]:
    """Reads the content of path, interprets its content as tab-separated values, finally returns
    the values of the nth column. If a row has less columns, that's silentely ignored."""
    ret = []

    with open(path) as sock:
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


def get_streets(workdir: str, relation_name: str) -> List[str]:
    """Reads list of streets for an area from OSM."""
    ret = get_nth_column(os.path.join(workdir, "streets-%s.csv" % relation_name), 1)
    ret += get_nth_column(os.path.join(workdir, "street-housenumbers-%s.csv" % relation_name), 1)
    return sorted(set(ret))


def get_url_hash(url: str) -> str:
    """Returns SHA256 hash of an URL."""
    return hashlib.sha256(url.encode('utf-8')).hexdigest()


def get_workdir(config):
    """Gets the directory which is writable."""
    return config.get('wsgi', 'workdir').strip()


def process_template(buf, osmrelation):
    """Turns an overpass query template to an actual query."""
    buf = buf.replace("@RELATION@", str(osmrelation))
    # area is relation + 3600000000 (3600000000 == relation), see js/ide.js
    # in https://github.com/tyrasd/overpass-turbo
    buf = buf.replace("@AREA@", str(3600000000 + osmrelation))
    return buf
