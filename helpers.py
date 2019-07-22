#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The helpers module contains functionality shared between other modules."""

import configparser
import re
import os
import pickle
from typing import Any
from typing import Callable
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import Sequence
from typing import TextIO
from typing import Tuple
from typing import cast
import yaml


class Range:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start: int, end: int, interpolation: str = "") -> None:
        self.__start = start
        self.__end = end
        self.__is_odd = start % 2 == 1  # type: Optional[bool]
        if interpolation == "all":
            self.__is_odd = None

    def get_start(self) -> int:
        """The smallest integer."""
        return self.__start

    def get_end(self) -> int:
        """The largest integer."""
        return self.__end

    def is_odd(self) -> Optional[bool]:
        """None for all house numbers on one side, bool otherwise."""
        return self.__is_odd

    def __contains__(self, item: int) -> bool:
        if (self.__is_odd is not None) and self.__is_odd != (item % 2 == 1):
            return False
        if self.__start <= item <= self.__end:
            return True
        return False

    def __repr__(self) -> str:
        return "Range(start=%s, end=%s, is_odd=%s)" % (self.__start, self.__end, self.__is_odd)

    def __eq__(self, other: object) -> bool:
        other_range = cast(Range, other)
        if self.__start != other_range.get_start():
            return False
        if self.__end != other_range.get_end():
            return False
        if self.__is_odd != other_range.is_odd():
            return False
        return True


class Ranges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items: List[Range]) -> None:
        self.__items = items

    def get_items(self) -> List[Range]:
        """The list of contained Range objects."""
        return self.__items

    def __contains__(self, item: int) -> bool:
        for i in self.__items:
            if item in i:
                return True
        return False

    def __repr__(self) -> str:
        return "Ranges(items=%s)" % self.__items

    def __eq__(self, other: object) -> bool:
        other_ranges = cast(Ranges, other)
        return self.__items == other_ranges.get_items()


class Relation:
    """A relation is a closed polygon on the map."""
    def __init__(self, datadir: str, workdir: str, name: str, parent: Dict[str, Any]) -> None:
        self.__workdir = workdir
        self.__name = name
        self.__parent = parent
        self.__dict = {}  # type: Dict[str, Any]
        relation_path = os.path.join(datadir, "relation-%s.yaml" % name)
        if os.path.exists(relation_path):
            with open(relation_path) as sock:
                self.__dict = yaml.load(sock)

    def get_property(self, key: str) -> Any:
        """Gets the value of a property transparently."""
        if key in self.__dict.keys():
            return self.__dict[key]

        return self.__parent[key]

    def has_property(self, key: str) -> bool:
        """Finds out if a given property is available, transparently."""
        if key in self.__dict.keys():
            return True

        return key in self.__parent.keys()

    def should_check_missing_streets(self) -> str:
        """Return value can be yes, no and only. Current default is "no"."""
        if self.has_property("suspicious-relations"):
            return cast(str, self.get_property("suspicious-relations"))

        return "no"

    def get_osm_streets_stream(self, mode: str) -> TextIO:
        """Opens the OSM street list of a relation."""
        path = os.path.join(self.__workdir, "streets-%s.csv" % self.__name)
        return cast(TextIO, open(path, mode=mode))

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-%s.csv" % self.__name)

    def get_osm_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the OSM house number list of a relation."""
        path = self.get_osm_housenumbers_path()
        return cast(TextIO, open(path, mode=mode))

    def get_ref_housenumbers_path(self) -> str:
        """Build the file name of the reference house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-reference-%s.lst" % self.__name)

    def get_ref_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the reference house number list of a relation."""
        return cast(TextIO, open(self.get_ref_housenumbers_path(), mode=mode))


class Relations:
    """A relations object is a container of named relation objects."""
    def __init__(self, datadir: str, workdir: str) -> None:
        self.__datadir = datadir
        self.__workdir = workdir
        with open(os.path.join(datadir, "relations.yaml")) as sock:
            self.__dict = yaml.load(sock)
        self.__relations = {}  # type: Dict[str, Relation]

    def get_datadir(self) -> str:
        """Gets the datadir directory path."""
        return self.__datadir

    def get_workdir(self) -> str:
        """Gets the workdir directory path."""
        return self.__workdir

    def get_relation(self, name: str) -> Relation:
        """Gets the relation that has the specified name."""
        if name not in self.__relations.keys():
            self.__relations[name] = Relation(self.__datadir, self.__workdir, name, self.__dict[name])
        return self.__relations[name]

    def get_names(self) -> List[str]:
        """Gets a sorted list of relation names."""
        return sorted(self.__dict.keys())

    def get_values(self) -> List[Any]:
        """Gets a list of relations."""
        return cast(List[Any], self.__dict.values())


def get_reftelepules_list_from_yaml(
        reftelepules_list: List[str],
        value: Dict[str, Any]
) -> List[str]:
    """Determines street-level and range-level reftelepules overrides."""
    if "reftelepules" in value.keys():
        reftelepules = cast(str, value["reftelepules"])
        reftelepules_list = [reftelepules]
    if "ranges" in value.keys():
        for street_range in value["ranges"]:
            street_range_dict = cast(Dict[str, str], street_range)
            if "reftelepules" in street_range_dict.keys():
                reftelepules_list.append(street_range_dict["reftelepules"])

    return reftelepules_list


def parse_relation_yaml(
        root: Dict[str, Any],
        street: str,
        refstreets: Dict[str, str],
        reftelepules_list: List[str]
) -> Tuple[Dict[str, str], List[str]]:
    """Parses the yaml of a single relation."""
    if "refstreets" in root.keys():
        # From OSM name to ref name.
        refstreets = cast(Dict[str, str], root["refstreets"])
    if "filters" in root.keys():
        # street-specific reftelepules override.
        filters = cast(Dict[str, Any], root["filters"])
        for filter_street, value in filters.items():
            if filter_street == street:
                reftelepules_list = get_reftelepules_list_from_yaml(reftelepules_list, value)

    return refstreets, reftelepules_list


def get_street_details(
        datadir: str,
        relations: Relations,
        street: str,
        relation_name: str
) -> Tuple[str, List[str], str, str]:
    """Determines the ref codes, street name and type for a street in a relation."""
    relation = relations.get_relation(relation_name)
    refmegye = relation.get_property("refmegye")
    reftelepules_list = [relation.get_property("reftelepules")]

    refstreets = {}  # type: Dict[str, str]
    root = relation_init(datadir, relation_name)
    if root:
        refstreets, reftelepules_list = parse_relation_yaml(root, street, refstreets, reftelepules_list)

    if street in refstreets.keys():
        street = refstreets[street]

    tokens = street.split(' ')
    street_name = " ".join(tokens[:-1])
    street_type = tokens[-1]
    return refmegye, sorted(set(reftelepules_list)), street_name, street_type


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


def get_only_in_first(first: List[Any], second: List[Any]) -> List[Any]:
    """Returns items which are in first, but not in second."""
    ret = []
    for i in first:
        if i not in second:
            ret.append(i)
    return ret


def get_in_both(first: List[Any], second: List[Any]) -> List[Any]:
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


def get_osm_streets(relations: Relations, relation_name: str) -> List[str]:
    """Reads list of streets for an area from OSM."""
    ret = []  # type: List[str]
    relation = relations.get_relation(relation_name)
    with relation.get_osm_streets_stream("r") as sock:
        ret += get_nth_column(sock, 1)
    if os.path.exists(relation.get_osm_housenumbers_path()):
        with relation.get_osm_housenumbers_stream("r") as sock:
            ret += get_nth_column(sock, 1)
    return sorted(set(ret))


def get_workdir(config: configparser.ConfigParser) -> str:
    """Gets the directory which is writable."""
    return config.get('wsgi', 'workdir').strip()


def process_template(buf: str, osmrelation: int) -> str:
    """Turns an overpass query template to an actual query."""
    buf = buf.replace("@RELATION@", str(osmrelation))
    # area is relation + 3600000000 (3600000000 == relation), see js/ide.js
    # in https://github.com/tyrasd/overpass-turbo
    buf = buf.replace("@AREA@", str(3600000000 + osmrelation))
    return buf


def get_content(workdir: str, path: str) -> str:
    """Gets the content of a file in workdir."""
    ret = ""
    with open(os.path.join(workdir, path)) as sock:
        ret = sock.read()
    return ret


def load_normalizers(datadir: str, relation_name: str) -> Tuple[Dict[str, Ranges], Dict[str, str], List[str]]:
    """Loads filters which allow silencing false positives. The return value is a tuple of the
    normalizers itself and an OSM name -> ref name dictionary."""
    filter_dict = {}  # type: Dict[str, Ranges]
    ref_streets = {}  # type: Dict[str, str]
    street_filters = []  # type: List[str]

    root = relation_init(datadir, relation_name)
    if not root:
        return filter_dict, ref_streets, street_filters

    if "filters" in root.keys():
        filters = root["filters"]
        for street in filters.keys():
            interpolation = ""
            if "interpolation" in filters[street]:
                interpolation = filters[street]["interpolation"]
            i = []
            if "ranges" not in filters[street]:
                continue
            for start_end in filters[street]["ranges"]:
                i.append(Range(int(start_end["start"]), int(start_end["end"]), interpolation))
            filter_dict[street] = Ranges(i)

    if "refstreets" in root.keys():
        ref_streets = root["refstreets"]

    if "street-filters" in root.keys():
        street_filters = root["street-filters"]

    return filter_dict, ref_streets, street_filters


def tsv_to_list(sock: TextIO) -> List[List[str]]:
    """Turns a tab-separated table into a list of lists."""
    table = []

    for line in sock.readlines():
        if not line.strip():
            continue
        cells = line.split("\t")
        table.append(cells)

    return table


def html_table_from_list(table: List[List[str]]) -> str:
    """Produces a HTML table from a list of lists."""
    ret = []
    ret.append('<table class="sortable">')
    for row_index, row_content in enumerate(table):
        ret.append("<tr>")
        for cell in row_content:
            if row_index == 0:
                ret.append('<th><a href="#">' + cell + "</a></th>")
            else:
                ret.append('<td>' + cell + "</td>")
        ret.append("</tr>")
    ret.append("</table>")
    return "".join(ret)


def normalize(house_numbers: str, street_name: str,
              normalizers: Dict[str, Ranges]) -> List[str]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    ret = []
    for house_number in house_numbers.split('-'):
        try:
            number = int(re.sub(r"([0-9]+).*", r"\1", house_number))
        except ValueError:
            continue

        if street_name in normalizers.keys():
            # Have a custom filter.
            normalizer = normalizers[street_name]
        else:
            # Default sanity checks.
            default = [Range(1, 999), Range(2, 998)]
            normalizer = Ranges(default)
        if number not in normalizer:
            continue

        ret.append(str(number))
    return ret


def get_house_numbers_from_lst(
        relations: Relations,
        relation_name: str,
        street_name: str,
        ref_street: str,
        normalizers: Dict[str, Ranges]
) -> List[str]:
    """Gets house numbers from reference."""
    house_numbers = []  # type: List[str]
    lst_street_name = ref_street
    prefix = lst_street_name + " "
    relation = relations.get_relation(relation_name)
    with relation.get_ref_housenumbers_stream("r") as sock:
        for line in sock.readlines():
            line = line.strip()
            if line.startswith(prefix):
                house_number = line.replace(prefix, '')
                house_numbers += normalize(house_number, street_name, normalizers)
    return sort_numerically(set(house_numbers))


def get_reference_streets_path(workdir: str, relation_name: str) -> str:
    """Build the file name of the reference street list of a relation."""
    return os.path.join(workdir, "streets-reference-%s.lst" % relation_name)


def get_reference_streets(workdir: str, relation_name: str, mode: str) -> TextIO:
    """Opens the reference street list of a relation."""
    path = get_reference_streets_path(workdir, relation_name)
    return cast(TextIO, open(path, mode=mode))


def get_streets_from_lst(workdir: str, relation_name: str) -> List[str]:
    """Gets streets from reference."""
    streets = []  # type: List[str]
    with get_reference_streets(workdir, relation_name, "r") as sock:
        for line in sock.readlines():
            line = line.strip()
            streets.append(line)
    return sorted(set(streets))


def get_house_numbers_from_csv(
        relation: Relation,
        street_name: str,
        normalizers: Dict[str, Ranges]
) -> List[str]:
    """Gets the OSM house number list of a street."""
    house_numbers = []  # type: List[str]
    with relation.get_osm_housenumbers_stream(mode="r") as sock:
        first = True
        for line in sock.readlines():
            if first:
                first = False
                continue
            tokens = line.strip().split('\t')
            if len(tokens) < 3:
                continue
            if tokens[1] != street_name:
                continue
            house_numbers += normalize(tokens[2], street_name, normalizers)
    return sort_numerically(set(house_numbers))


def get_suspicious_streets(
        datadir: str,
        relations: Relations,
        relation_name: str
) -> Tuple[List[Tuple[str, List[str]]], List[Tuple[str, List[str]]]]:
    """Tries to find streets which do have at least one house number, but are suspicious as other
    house numbers are probably missing."""
    suspicious_streets = []
    done_streets = []

    street_names = get_osm_streets(relations, relation_name)
    normalizers, ref_streets = load_normalizers(datadir, relation_name)[:2]
    for street_name in street_names:
        ref_street = street_name
        # See if we need to map the OSM name to ref name.
        if street_name in ref_streets.keys():
            ref_street = ref_streets[street_name]

        reference_house_numbers = get_house_numbers_from_lst(relations, relation_name, street_name,
                                                             ref_street, normalizers)
        osm_house_numbers = get_house_numbers_from_csv(relations.get_relation(relation_name),
                                                       street_name,
                                                       normalizers)
        only_in_reference = get_only_in_first(reference_house_numbers, osm_house_numbers)
        in_both = get_in_both(reference_house_numbers, osm_house_numbers)
        if only_in_reference:
            suspicious_streets.append((street_name, only_in_reference))
        if in_both:
            done_streets.append((street_name, in_both))
    # Sort by length.
    suspicious_streets.sort(key=lambda result: len(result[1]), reverse=True)

    return suspicious_streets, done_streets


def get_suspicious_relations(relations: Relations, relation_name: str) -> Tuple[List[str], List[str]]:
    """Tries to find missing streets in a relation."""
    reference_streets = get_streets_from_lst(relations.get_workdir(), relation_name)
    _, ref_streets, street_blacklist = load_normalizers(relations.get_datadir(), relation_name)
    osm_streets = []
    for street in get_osm_streets(relations, relation_name):
        if street in ref_streets.keys():
            street = ref_streets[street]
        osm_streets.append(street)

    only_in_reference = get_only_in_first(reference_streets, osm_streets)
    only_in_reference = [i for i in only_in_reference if i not in street_blacklist]
    in_both = get_in_both(reference_streets, osm_streets)

    return only_in_reference, in_both


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


def house_numbers_of_street(
        relations: Relations,
        reference: Dict[str, Dict[str, Dict[str, List[str]]]],
        relation_name: str,
        street: str
) -> List[str]:
    """Gets house numbers for a street locally."""
    refmegye, reftelepules_list, street_name, street_type = get_street_details(relations.get_datadir(),
                                                                               relations,
                                                                               street,
                                                                               relation_name)
    street = street_name + " " + street_type
    ret = []  # type: List[str]
    for reftelepules in reftelepules_list:
        if street in reference[refmegye][reftelepules].keys():
            house_numbers = reference[refmegye][reftelepules][street]
            ret += [street + " " + i for i in house_numbers]

    return ret


def streets_of_relation(
        relations: Relations,
        reference: Dict[str, Dict[str, List[str]]],
        relation_name: str
) -> List[str]:
    """Gets street names for a relation from a reference."""
    relation = relations.get_relation(relation_name)
    refmegye = relation.get_property("refmegye")
    reftelepules = relation.get_property("reftelepules")

    return reference[refmegye][reftelepules]


def get_reference_housenumbers(relations: Relations, reference: str, relation_name: str) -> None:
    """Gets known house numbers (not their coordinates) from a reference site, based on street names
    from OSM."""
    relation = relations.get_relation(relation_name)
    memory_cache = build_reference_cache(reference)

    streets = get_osm_streets(relations, relation_name)

    lst = []  # type: List[str]
    for street in streets:
        lst += house_numbers_of_street(relations, memory_cache, relation_name, street)

    lst = sorted(set(lst))
    with relation.get_ref_housenumbers_stream("w") as sock:
        for line in lst:
            sock.write(line + "\n")


def get_sorted_reference_streets(relations: Relations, reference: str, relation_name: str) -> None:
    """Gets known streets (not their coordinates) from a reference site, based on relation names
    from OSM."""
    memory_cache = build_street_reference_cache(reference)

    lst = streets_of_relation(relations, memory_cache, relation_name)

    lst = sorted(set(lst))
    with get_reference_streets(relations.get_workdir(), relation_name, "w") as sock:
        for line in lst:
            sock.write(line + "\n")


def relation_init(datadir: str, relation_name: str) -> Dict[str, Any]:
    """Returns a relation from a yaml path."""
    relation_path = os.path.join(datadir, "relation-%s.yaml" % relation_name)
    if os.path.exists(relation_path):
        with open(relation_path) as sock:
            root = {}  # type: Dict[str, Any]
            root = yaml.load(sock)
            return root

    return {}


def relation_get_filters(relation: Dict[str, Any]) -> Dict[str, Any]:
    """Returns filters from a relation."""
    if "filters" in relation.keys():
        return cast(Dict[str, Any], relation["filters"])

    return {}


def relation_filters_get_street(filters: Dict[str, Any], street: str) -> Dict[str, Any]:
    """Returns a street from relation filters."""
    if street in filters.keys():
        return cast(Dict[str, Any], filters[street])

    return {}


def relation_street_is_even_odd(street: Dict[str, Any]) -> bool:
    """Determines in a relation's street is interpolation=all or not."""
    interpolation_all = False
    if "interpolation" in street:
        if street["interpolation"] == "all":
            interpolation_all = True

    return not interpolation_all


def get_streets_query(datadir: str, relations: Relations, relation: str) -> str:
    """Produces a query which lists streets in relation."""
    with open(os.path.join(datadir, "streets-template.txt")) as sock:
        return process_template(sock.read(), relations.get_relation(relation).get_property("osmrelation"))


def write_streets_result(relations: Relations, relation_name: str, result_from_overpass: str) -> None:
    """Writes the result for overpass of get_streets_query()."""
    result = sort_streets_csv(result_from_overpass)
    relation = relations.get_relation(relation_name)
    with relation.get_osm_streets_stream("w") as sock:
        sock.write(result)


def get_street_housenumbers_query(datadir: str, relations: Relations, relation: str) -> str:
    """Produces a query which lists house numbers in relation."""
    with open(os.path.join(datadir, "street-housenumbers-template.txt")) as sock:
        return process_template(sock.read(), relations.get_relation(relation).get_property("osmrelation"))


def write_street_housenumbers(relation: Relation, result_from_overpass: str) -> None:
    """Writes the result for overpass of get_street_housenumbers_query()."""
    result = sort_housenumbers_csv(result_from_overpass)
    with relation.get_osm_housenumbers_stream(mode="w") as sock:
        sock.write(result)


def format_even_odd(only_in_ref: List[str]) -> List[str]:
    """Separate even and odd numbers, this helps survey in most cases."""
    even = sorted([i for i in only_in_ref if int(i) % 2 == 0], key=int)
    even_string = ", ".join(even)
    odd = sorted([i for i in only_in_ref if int(i) % 2 == 1], key=int)
    odd_string = ", ".join(odd)
    elements = []
    if odd_string:
        elements.append(odd_string)
    if even_string:
        elements.append(even_string)
    return elements


def write_suspicious_streets_result(
        relations: Relations,
        relation: str
) -> Tuple[int, int, int, str, List[List[str]]]:
    """Calculate a write stat for the house number coverage of a relation."""
    suspicious_streets, done_streets = get_suspicious_streets(relations.get_datadir(), relations, relation)

    relation_filters = relation_get_filters(relation_init(relations.get_datadir(), relation))

    todo_count = 0
    table = []
    table.append(["Utcanév", "Hiányzik db", "Házszámok"])
    for result in suspicious_streets:
        # street_name, only_in_ref
        row = []
        row.append(result[0])
        row.append(str(len(result[1])))

        if not relation_street_is_even_odd(relation_filters_get_street(relation_filters, result[0])):
            row.append(", ".join(result[1]))
        else:
            row.append("<br/>".join(format_even_odd(result[1])))

        todo_count += len(result[1])
        table.append(row)
    done_count = 0
    for result in done_streets:
        done_count += len(result[1])
    if done_count > 0 or todo_count > 0:
        percent = "%.2f" % (done_count / (done_count + todo_count) * 100)
    else:
        percent = "N/A"

    # Write the bottom line to a file, so the index page show it fast.
    with open(os.path.join(relations.get_workdir(), relation + ".percent"), "w") as sock:
        sock.write(percent)

    todo_street_count = len(suspicious_streets)
    return todo_street_count, todo_count, done_count, percent, table


def write_missing_relations_result(relations: Relations, relation: str) -> Tuple[int, int, str, List[str]]:
    """Calculate a write stat for the street coverage of a relation."""
    todo_streets, done_streets = get_suspicious_relations(relations, relation)
    streets = []
    for street in todo_streets:
        streets.append(street)
    todo_count = len(todo_streets)
    done_count = len(done_streets)
    if done_count > 0 or todo_count > 0:
        percent = "%.2f" % (done_count / (done_count + todo_count) * 100)
    else:
        percent = "N/A"

    # Write the bottom line to a file, so the index page show it fast.
    with open(os.path.join(relations.get_workdir(), relation + "-streets.percent"), "w") as sock:
        sock.write(percent)

    return todo_count, done_count, percent, streets


def refmegye_get_name(refmegye: str) -> str:
    """Produces a UI name for a refmegye."""
    names = {
        '01': 'Budapest',
        '14': 'Pest megye',
    }
    if refmegye in names.keys():
        return names[refmegye]

    return ""

# vim:set shiftwidth=4 softtabstop=4 expandtab:
