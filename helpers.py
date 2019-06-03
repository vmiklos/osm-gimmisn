#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The helpers module contains functionality shared between other modules."""

import re
import os
import hashlib
from typing import Callable, Dict, Iterable, List, Sequence, Tuple
import urllib.parse
import yaml


class Range:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start, end):
        self.start = start
        self.end = end
        self.is_odd = start % 2 == 1

    def __contains__(self, item):
        if self.is_odd != (item % 2 == 1):
            return False
        if self.start <= item <= self.end:
            return True
        return False

    def __repr__(self):
        return "Range(start=%s, end=%s, is_odd=%s)" % (self.start, self.end, self.is_odd)

    def __eq__(self, other):
        if self.start != other.start:
            return False
        if self.end != other.end:
            return False
        return True


class Ranges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items):
        self.items = items

    def __contains__(self, item):
        for i in self.items:
            if item in i:
                return True
        return False

    def __repr__(self):
        return "Ranges(items=%s)" % self.items

    def __eq__(self, other):
        return self.items == other.items


def get_street_details(datadir, street, relation_name):
    """Determines the ref codes, street name and type for a street in a relation."""
    with open(os.path.join(datadir, "relations.yaml")) as sock:
        relations = yaml.load(sock)
    relation = relations[relation_name]
    refmegye = relation["refmegye"]
    reftelepules = relation["reftelepules"]

    refstreets = {}  # type: Dict[str, str]
    if os.path.exists(os.path.join(datadir, "housenumber-filters-%s.yaml" % relation_name)):
        with open(os.path.join(datadir, "housenumber-filters-%s.yaml" % relation_name)) as sock:
            # See if config wants to map:
            root = yaml.load(sock)
            if "refstreets" in root.keys():
                # From OSM name to ref name.
                refstreets = root["refstreets"]
            if "filters" in root.keys():
                # street-specific reftelepules override.
                filters = root["filters"]
                for filter_street, value in filters.items():
                    if "simplify" not in root.keys():
                        # New code path.
                        if filter_street == street and "reftelepules" in value.keys():
                            reftelepules = value["reftelepules"]
                    else:
                        # Old code path
                        street_simple = simplify(street)
                        if filter_street == street_simple and "reftelepules" in value.keys():
                            reftelepules = value["reftelepules"]

    if street in refstreets.keys():
        street = refstreets[street]

    tokens = street.split(' ')
    street_name = " ".join(tokens[:-1])
    street_type = tokens[-1]
    return refmegye, reftelepules, street_name, street_type


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


def get_array_nth(arr: Sequence[str], index: int) -> str:
    """Gets the nth element of arr, returns en empty string on error."""
    return arr[index] if len(arr) > index else ''


def simplify(name: str) -> str:
    """ Handles normalization of a street name."""
    name = name.replace('Á', 'A').replace('á', 'a')
    name = name.replace('É', 'E').replace('é', 'e')
    name = name.replace('Í', 'I').replace('í', 'i')
    name = name.replace('Ó', 'O').replace('ó', 'o')
    name = name.replace('Ö', 'O').replace('ö', 'o')
    name = name.replace('Ő', 'O').replace('ő', 'o')
    name = name.replace('Ú', 'U').replace('ú', 'u')
    name = name.replace('Ü', 'U').replace('ü', 'u')
    name = name.replace('Ű', 'U').replace('ű', 'u')
    name = name.replace('.', '')
    name = name.replace(' ', '_')
    name = name.lower()
    return name


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


def get_content(workdir, path):
    """Gets the content of a file in workdir."""
    ret = ""
    with open(os.path.join(workdir, path)) as sock:
        ret = sock.read()
    return ret


def load_normalizers(datadir: str, relation_name: str) -> Tuple[Dict[str, Ranges], Dict[str, str], bool]:
    """Loads filters which allow silencing false positives."""
    filter_dict = {}  # type: Dict[str, Ranges]
    ref_streets = {}  # type: Dict[str, str]

    path = os.path.join(datadir, "housenumber-filters-%s.yaml" % relation_name)
    if not os.path.exists(path):
        return filter_dict, ref_streets, False

    with open(path) as sock:
        root = yaml.load(sock)

    if "filters" in root.keys():
        filters = root["filters"]
        for street in filters.keys():
            i = []
            if "ranges" not in filters[street]:
                continue
            for start_end in filters[street]["ranges"]:
                i.append(Range(int(start_end["start"]), int(start_end["end"])))
            filter_dict[street] = Ranges(i)

    if "refstreets" in root.keys():
        ref_streets = root["refstreets"]

    use_simplify = False
    if "simplify" in root.keys():
        use_simplify = root["simplify"]

    return filter_dict, ref_streets, use_simplify


def tsv_to_list(sock):
    """Turns a tab-separated table into a list of lists."""
    table = []

    for line in sock.readlines():
        if not line.strip():
            continue
        cells = line.split("\t")
        table.append(cells)

    return table


def html_table_from_list(table):
    """Produces a HTML table from a list of lists."""
    ret = []
    ret.append('<table rules="all" frame="border" cellpadding="4" class="sortable">')
    for row_index, row_content in enumerate(table):
        ret.append("<tr>")
        for cell in row_content:
            if row_index == 0:
                ret.append('<th align="left" valign="center"><a href="#">' + cell + "</a></th>")
            else:
                ret.append('<td align="left" valign="top">' + cell + "</td>")
        ret.append("</tr>")
    ret.append("</table>")
    return "".join(ret)


def get_street_url(datadir, street, prefix, relation_name):
    """Returns URL of a street based on config."""
    refmegye, reftelepules, street_name, street_type = get_street_details(datadir, street, relation_name)
    url = prefix
    parameters = {
        "p_p_id": "wardsearch_WAR_nvinvrportlet",
        "p_p_lifecycle": "2",
        "p_p_state": "normal",
        "p_p_mode": "view",
        "p_p_resource_id": "resourceIdGetHazszam",
        "p_p_cacheability": "cacheLevelPage",
        "p_p_col_id": "column-2",
        "p_p_col_count": "1",
        "_wardsearch_WAR_nvinvrportlet_vlId": "291",
        "_wardsearch_WAR_nvinvrportlet_vltId": "684",
        "_wardsearch_WAR_nvinvrportlet_keywords": "",
        "_wardsearch_WAR_nvinvrportlet_megyeKod": refmegye,
        "_wardsearch_WAR_nvinvrportlet_telepulesKod": reftelepules,
        "_wardsearch_WAR_nvinvrportlet_kozterNev": street_name,
        "_wardsearch_WAR_nvinvrportlet_kozterJelleg": street_type,
    }
    url += "?" + urllib.parse.urlencode(parameters)
    return url


def normalize(house_numbers: str, street_name: str, use_simplify: bool,
              normalizers: Dict[str, Ranges]) -> List[str]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    ret = []
    for house_number in house_numbers.split('-'):
        try:
            number = int(re.sub(r"([0-9]+).*", r"\1", house_number))
        except ValueError:
            continue

        street_simple = street_name
        if use_simplify:
            # Old code path
            street_simple = simplify(street_name)

        if street_simple in normalizers.keys():
            # Have a custom filter.
            normalizer = normalizers[street_simple]
        else:
            # Default sanity checks.
            default = [Range(1, 999), Range(2, 998)]
            normalizer = Ranges(default)
        if number not in normalizer:
            continue

        ret.append(str(number))
    return ret

# vim:set shiftwidth=4 softtabstop=4 expandtab:
