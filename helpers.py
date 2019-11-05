#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The helpers module contains functionality shared between other modules."""

import configparser
import re
import os
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
import yattag  # type: ignore
from i18n import translate as _
import util


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


class RelationFiles:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, datadir: str, workdir: str, name: str):
        self.__datadir = datadir
        self.__workdir = workdir
        self.__name = name

    def get_ref_streets_path(self) -> str:
        """Build the file name of the reference street list of a relation."""
        return os.path.join(self.__workdir, "streets-reference-%s.lst" % self.__name)

    def get_ref_streets_stream(self, mode: str) -> TextIO:
        """Opens the reference street list of a relation."""
        path = self.get_ref_streets_path()
        return cast(TextIO, open(path, mode=mode))

    def get_osm_streets_path(self) -> str:
        """Build the file name of the OSM street list of a relation."""
        return os.path.join(self.__workdir, "streets-%s.csv" % self.__name)

    def get_osm_streets_stream(self, mode: str) -> TextIO:
        """Opens the OSM street list of a relation."""
        path = self.get_osm_streets_path()
        return cast(TextIO, open(path, mode=mode))

    def write_osm_streets(self, result_from_overpass: str) -> None:
        """Writes the result for overpass of Relation.get_osm_streets_query()."""
        result = sort_streets_csv(result_from_overpass)
        with self.get_osm_streets_stream("w") as sock:
            sock.write(result)

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-%s.csv" % self.__name)

    def get_osm_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the OSM house number list of a relation."""
        path = self.get_osm_housenumbers_path()
        return cast(TextIO, open(path, mode=mode))

    def write_osm_housenumbers(self, result_from_overpass: str) -> None:
        """Writes the result for overpass of Relation.get_osm_housenumbers_query()."""
        result = sort_housenumbers_csv(result_from_overpass)
        with self.get_osm_housenumbers_stream(mode="w") as stream:
            stream.write(result)

    def get_ref_housenumbers_path(self) -> str:
        """Build the file name of the reference house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-reference-%s.lst" % self.__name)

    def get_ref_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the reference house number list of a relation."""
        return cast(TextIO, open(self.get_ref_housenumbers_path(), mode=mode))

    def get_housenumbers_percent_path(self) -> str:
        """Builds the file name of the house number percent file of a relation."""
        return os.path.join(self.__workdir, "%s.percent" % self.__name)

    def get_housenumbers_percent_stream(self, mode: str) -> TextIO:
        """Opens the house number percent file of a relation."""
        return cast(TextIO, open(self.get_housenumbers_percent_path(), mode=mode))

    def get_streets_percent_path(self) -> str:
        """Builds the file name of the street percent file of a relation."""
        return os.path.join(self.__workdir, "%s-streets.percent" % self.__name)

    def get_streets_percent_stream(self, mode: str) -> TextIO:
        """Opens the street percent file of a relation."""
        return cast(TextIO, open(self.get_streets_percent_path(), mode=mode))


class RelationConfig:
    """A relation configuration comes directly from static data, not a result of some external query."""
    def __init__(self, parent_config: Dict[str, Any], my_config: Dict[str, Any]) -> None:
        self.__parent = parent_config
        self.__dict = my_config

    def __get_property(self, key: str) -> Any:
        """Gets the value of a property transparently."""
        if key in self.__dict.keys():
            return self.__dict[key]

        if key in self.__parent.keys():
            return self.__parent[key]

        return None

    def is_active(self) -> bool:
        """Gets if the relation is active."""
        return not cast(bool, self.__get_property("inactive"))

    def get_osmrelation(self) -> int:
        """Gets the OSM relation object's ID."""
        return cast(int, self.__get_property("osmrelation"))

    def get_refmegye(self) -> str:
        """Gets the relation's refmegye identifier from reference."""
        return cast(str, self.__get_property("refmegye"))

    def get_reftelepules(self) -> str:
        """Gets the relation's reftelepules identifier from reference."""
        return cast(str, self.__get_property("reftelepules"))

    def should_check_missing_streets(self) -> str:
        """Return value can be 'yes', 'no' and 'only'."""
        if self.__get_property("missing-streets"):
            return cast(str, self.__get_property("missing-streets"))

        return "yes"

    def get_refstreets(self) -> Dict[str, str]:
        """Returns an OSM name -> ref name map."""
        if self.__get_property("refstreets"):
            return cast(Dict[str, str], self.__get_property("refstreets"))
        return {}

    def get_filters(self) -> Dict[str, Any]:
        """Returns a street name -> properties map."""
        if self.__get_property("filters"):
            return cast(Dict[str, Any], self.__get_property("filters"))
        return {}

    def get_filter_street(self, street: str) -> Dict[str, Any]:
        """Returns a street from relation filters."""
        filters = self.get_filters()
        if street in filters.keys():
            return cast(Dict[str, Any], filters[street])

        return {}

    def get_street_is_even_odd(self, street: str) -> bool:
        """Determines in a relation's street is interpolation=all or not."""
        street_props = self.get_filter_street(street)
        interpolation_all = False
        if "interpolation" in street_props:
            if street_props["interpolation"] == "all":
                interpolation_all = True

        return not interpolation_all

    def get_street_reftelepules(self, street: str) -> List[str]:
        """Returns a list of reftelepules values specific to a street."""
        ret = [self.__get_property("reftelepules")]
        if not self.__get_property("filters"):
            return ret

        relation_filters = self.get_filters()
        for filter_street, value in relation_filters.items():
            if filter_street != street:
                continue

            if "reftelepules" in value.keys():
                reftelepules = cast(str, value["reftelepules"])
                ret = [reftelepules]
            if "ranges" in value.keys():
                for street_range in value["ranges"]:
                    street_range_dict = cast(Dict[str, str], street_range)
                    if "reftelepules" in street_range_dict.keys():
                        ret.append(street_range_dict["reftelepules"])

        return sorted(set(ret))

    def get_street_filters(self) -> List[str]:
        """Gets list of streets which are only in reference, but have to be filtered out."""
        if self.__get_property("street-filters"):
            return cast(List[str], self.__get_property("street-filters"))
        return []


class Relation:
    """A relation is a closed polygon on the map."""
    def __init__(self, datadir: str, workdir: str, name: str, parent_config: Dict[str, Any]) -> None:
        self.__datadir = datadir
        self.__workdir = workdir
        self.__name = name
        my_config = {}  # type: Dict[str, Any]
        self.__file = RelationFiles(datadir, workdir, name)
        relation_path = os.path.join(datadir, "relation-%s.yaml" % name)
        if os.path.exists(relation_path):
            with open(relation_path) as sock:
                my_config = yaml.load(sock)
        self.__config = RelationConfig(parent_config, my_config)

    def get_name(self) -> str:
        """Gets the name of the relation."""
        return self.__name

    def get_files(self) -> RelationFiles:
        """Gets access to the file interface."""
        return self.__file

    def get_config(self) -> RelationConfig:
        """Gets access to the config interface."""
        return self.__config

    def get_street_ranges(self) -> Dict[str, Ranges]:
        """Gets a street name -> ranges map, which allows silencing false positives."""
        filter_dict = {}  # type: Dict[str, Ranges]

        filters = self.get_config().get_filters()
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

        return filter_dict

    def get_ref_street_from_osm_street(self, osm_street_name: str) -> str:
        """Maps an OSM street name to a ref street name."""
        refstreets = self.get_config().get_refstreets()

        if osm_street_name in refstreets.keys():
            return refstreets[osm_street_name]

        return osm_street_name

    def get_osm_streets(self) -> List[str]:
        """Reads list of streets for an area from OSM."""
        ret = []  # type: List[str]
        with self.get_files().get_osm_streets_stream("r") as sock:
            ret += get_nth_column(sock, 1)
        if os.path.exists(self.get_files().get_osm_housenumbers_path()):
            with self.get_files().get_osm_housenumbers_stream("r") as sock:
                ret += get_nth_column(sock, 1)
        return sorted(set(ret))

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
        with open(os.path.join(self.__datadir, "streets-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())

    def get_osm_housenumbers(self, street_name: str) -> List[str]:
        """Gets the OSM house number list of a street."""
        house_numbers = []  # type: List[str]
        with self.get_files().get_osm_housenumbers_stream(mode="r") as sock:
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
                house_numbers += normalize(self, tokens[2], street_name, self.get_street_ranges())
        return sort_numerically(set(house_numbers))

    def build_ref_streets(self, reference: Dict[str, Dict[str, List[str]]]) -> List[str]:
        """
        Builds a list of streets from a reference cache.
        """
        refmegye = self.get_config().get_refmegye()
        reftelepules = self.get_config().get_reftelepules()
        return reference[refmegye][reftelepules]

    def write_ref_streets(self, reference: str) -> None:
        """Gets known streets (not their coordinates) from a reference site, based on relation names
        from OSM."""
        memory_cache = util.build_street_reference_cache(reference)

        lst = self.build_ref_streets(memory_cache)

        lst = sorted(set(lst))
        with self.get_files().get_ref_streets_stream("w") as sock:
            for line in lst:
                sock.write(line + "\n")

    def get_ref_streets(self) -> List[str]:
        """Gets streets from reference."""
        streets = []  # type: List[str]
        with self.get_files().get_ref_streets_stream("r") as sock:
            for line in sock.readlines():
                line = line.strip()
                streets.append(line)
        return sorted(set(streets))

    def build_ref_housenumbers(
            self,
            reference: Dict[str, Dict[str, Dict[str, List[str]]]],
            street: str,
            suffix: str
    ) -> List[str]:
        """
        Builds a list of housenumbers from a reference cache.
        This is serialized to disk by write_ref_housenumbers().
        """
        refmegye = self.get_config().get_refmegye()
        street = self.get_ref_street_from_osm_street(street)
        ret = []  # type: List[str]
        for reftelepules in self.get_config().get_street_reftelepules(street):
            if refmegye not in reference.keys():
                continue
            refmegye_dict = reference[refmegye]
            if reftelepules not in refmegye_dict.keys():
                continue
            reftelepules_dict = refmegye_dict[reftelepules]
            if street in reftelepules_dict.keys():
                house_numbers = reference[refmegye][reftelepules][street]
                ret += [street + " " + i + suffix for i in house_numbers]

        return ret

    @staticmethod
    def __get_ref_suffix(index: int) -> str:
        """Determines what suffix should the Nth reference use for hours numbers."""
        if index == 0:
            return ""

        return "*"

    def write_ref_housenumbers(self, references: List[str]) -> None:
        """
        Writes known house numbers (not their coordinates) from a reference, based on street names
        from OSM. Uses build_reference_cache() to build an indexed reference, the result will be
        used by __get_ref_housenumbers().
        """
        # Convert relative paths to absolute ones.
        references = [get_abspath(reference) for reference in references]

        memory_caches = util.build_reference_caches(references)

        streets = self.get_osm_streets()

        lst = []  # type: List[str]
        for street in streets:
            for index, memory_cache in enumerate(memory_caches):
                suffix = Relation.__get_ref_suffix(index)
                lst += self.build_ref_housenumbers(memory_cache, street, suffix)

        lst = sorted(set(lst))
        with self.get_files().get_ref_housenumbers_stream("w") as sock:
            for line in lst:
                sock.write(line + "\n")

    def __get_ref_housenumbers(self) -> Dict[str, List[str]]:
        """Gets house numbers from reference, produced by write_ref_housenumbers()."""
        ret = {}  # type: Dict[str, List[str]]
        lines = []  # type: List[str]
        with self.get_files().get_ref_housenumbers_stream("r") as sock:
            for line in sock.readlines():
                line = line.strip()
                lines.append(line)
        street_ranges = self.get_street_ranges()
        for osm_street_name in self.get_osm_streets():
            house_numbers = []  # type: List[str]
            ref_street_name = self.get_ref_street_from_osm_street(osm_street_name)
            prefix = ref_street_name + " "
            for line in lines:
                if line.startswith(prefix):
                    house_number = line.replace(prefix, '')
                    house_numbers += normalize(self, house_number, osm_street_name, street_ranges)
            ret[osm_street_name] = sort_numerically(set(house_numbers))
        return ret

    def get_missing_housenumbers(self) -> Tuple[List[Tuple[str, List[str]]], List[Tuple[str, List[str]]]]:
        """
        Compares ref and osm house numbers, prints the ones which are in ref, but not in osm.
        Return value is a pair of ongoing and done streets.
        Each of of these is a pair of a street name and a house number list.
        """
        ongoing_streets = []
        done_streets = []

        street_names = self.get_osm_streets()
        all_ref_house_numbers = self.__get_ref_housenumbers()
        for street_name in street_names:
            ref_house_numbers = all_ref_house_numbers[street_name]
            osm_house_numbers = self.get_osm_housenumbers(street_name)
            only_in_reference = get_only_in_first(ref_house_numbers, osm_house_numbers)
            in_both = get_in_both(ref_house_numbers, osm_house_numbers)
            if only_in_reference:
                ongoing_streets.append((street_name, only_in_reference))
            if in_both:
                done_streets.append((street_name, in_both))
        # Sort by length.
        ongoing_streets.sort(key=lambda result: len(result[1]), reverse=True)

        return ongoing_streets, done_streets

    def write_missing_housenumbers(self) -> Tuple[int, int, int, str, List[List[yattag.Doc]]]:
        """Calculate a write stat for the house number coverage of a relation."""
        ongoing_streets, done_streets = self.get_missing_housenumbers()

        todo_count = 0
        table = []
        table.append([util.html_escape(_("Street name")),
                      util.html_escape(_("Missing count")),
                      util.html_escape(_("House numbers"))])
        for result in ongoing_streets:
            # street_name, only_in_ref
            row = []
            row.append(util.html_escape(result[0]))
            row.append(util.html_escape(str(len(result[1]))))

            doc = yattag.Doc()
            if not self.get_config().get_street_is_even_odd(result[0]):
                for index, item in enumerate(result[1]):
                    if index:
                        doc.text(", ")
                    doc.asis(util.color_house_number(item).getvalue())
            else:
                util.format_even_odd(result[1], doc)
            row.append(doc)

            todo_count += len(result[1])
            table.append(row)
        done_count = 0
        for result in done_streets:
            done_count += len(result[1])
        if done_count > 0 or todo_count > 0:
            percent = "%.2f" % (done_count / (done_count + todo_count) * 100)
        else:
            percent = "100.00"

        # Write the bottom line to a file, so the index page show it fast.
        with self.get_files().get_housenumbers_percent_stream("w") as stream:
            stream.write(percent)

        todo_street_count = len(ongoing_streets)
        return todo_street_count, todo_count, done_count, percent, table

    def get_missing_streets(self) -> Tuple[List[str], List[str]]:
        """Tries to find missing streets in a relation."""
        reference_streets = self.get_ref_streets()
        street_blacklist = self.get_config().get_street_filters()
        osm_streets = [self.get_ref_street_from_osm_street(street) for street in self.get_osm_streets()]

        only_in_reference = get_only_in_first(reference_streets, osm_streets)
        only_in_reference = [i for i in only_in_reference if i not in street_blacklist]
        in_both = get_in_both(reference_streets, osm_streets)

        return only_in_reference, in_both

    def write_missing_streets(self) -> Tuple[int, int, str, List[str]]:
        """Calculate a write stat for the street coverage of a relation."""
        todo_streets, done_streets = self.get_missing_streets()
        streets = []
        for street in todo_streets:
            streets.append(street)
        todo_count = len(todo_streets)
        done_count = len(done_streets)
        if done_count > 0 or todo_count > 0:
            percent = "%.2f" % (done_count / (done_count + todo_count) * 100)
        else:
            percent = "100.00"

        # Write the bottom line to a file, so the index page show it fast.
        with self.get_files().get_streets_percent_stream("w") as stream:
            stream.write(percent)

        return todo_count, done_count, percent, streets

    def get_osm_housenumbers_query(self) -> str:
        """Produces a query which lists house numbers in relation."""
        with open(os.path.join(self.__datadir, "street-housenumbers-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())


class Relations:
    """A relations object is a container of named relation objects."""
    def __init__(self, datadir: str, workdir: str) -> None:
        self.__datadir = datadir
        self.__workdir = workdir
        with open(os.path.join(datadir, "relations.yaml")) as sock:
            self.__dict = yaml.load(sock)
        self.__relations = {}  # type: Dict[str, Relation]
        self.__activate_all = False
        with open(os.path.join(datadir, "refmegye-names.yaml")) as stream:
            self.__refmegye_names = yaml.load(stream)
        with open(os.path.join(datadir, "reftelepules-names.yaml")) as stream:
            self.__reftelepules_names = yaml.load(stream)

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

    def get_active_names(self) -> List[str]:
        """Gets a sorted list of active relation names."""
        ret = []  # type: List[Relation]
        for relation in self.get_relations():
            if self.__activate_all or relation.get_config().is_active():
                ret.append(relation)
        return sorted([relation.get_name() for relation in ret])

    def get_relations(self) -> List[Relation]:
        """Gets a list of relations."""
        ret = []  # type: List[Relation]
        for name in self.get_names():
            ret.append(self.get_relation(name))
        return ret

    def activate_all(self, flag: bool) -> None:
        """Sets if inactive=true is ignored or not."""
        self.__activate_all = flag

    def refmegye_get_name(self, refmegye: str) -> str:
        """Produces a UI name for a refmegye."""
        if refmegye in self.__refmegye_names:
            return cast(str, self.__refmegye_names[refmegye])

        return ""

    def refmegye_get_reftelepules_ids(self, refmegye_name: str) -> List[str]:
        """Produces reftelepules IDs of a refmegye."""
        if refmegye_name not in self.__reftelepules_names:
            return []

        refmegye = self.__reftelepules_names[refmegye_name]
        return list(refmegye.keys())

    def reftelepules_get_name(self, refmegye_name: str, reftelepules: str) -> str:
        """Produces a UI name for a reftelepules in refmegye."""
        if refmegye_name not in self.__reftelepules_names:
            return ""

        refmegye = self.__reftelepules_names[refmegye_name]
        if reftelepules not in refmegye:
            return ""

        return cast(str, refmegye[reftelepules])


def sort_numerically(strings: Iterable[str]) -> List[str]:
    """Sorts strings according to their numerical value, not alphabetically."""
    return sorted(strings, key=util.split_house_number)


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
    return (missing_name, name, highway, service, util.split_house_number(oid))


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
            util.split_house_number(housenumber),
            housename, util.split_house_number(cons), tail, util.split_house_number(oid))


def get_array_nth(arr: Sequence[str], index: int) -> str:
    """Gets the nth element of arr, returns en empty string on error."""
    return arr[index] if len(arr) > index else ''


def get_only_in_first(first: List[str], second: List[str]) -> List[str]:
    """Returns items which are in first, but not in second."""
    # Strip suffix that is ignored.
    first_stripped = [re.sub(r"\*$", "", i) for i in first]
    second_stripped = [re.sub(r"\*$", "", i) for i in second]

    ret = []
    for index, item in enumerate(first_stripped):
        if item not in second_stripped:
            ret.append(first[index])
    return ret


def get_in_both(first: List[str], second: List[str]) -> List[str]:
    """Returns items which are in both first and second."""
    # Strip suffix that is ignored.
    first_stripped = [re.sub(r"\*$", "", i) for i in first]
    second_stripped = [re.sub(r"\*$", "", i) for i in second]

    ret = []
    for index, item in enumerate(first_stripped):
        if item in second_stripped:
            ret.append(first[index])
    return ret


def git_link(version: str, prefix: str) -> yattag.Doc:
    """Generates a HTML link based on a website prefix and a git-describe version."""
    commit_hash = re.sub(".*-g", "", version)
    doc = yattag.Doc()
    with doc.tag("a", href=prefix + commit_hash):
        doc.text(version)
    return doc


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


def get_abspath(path: str) -> str:
    """Make a path absolute, taking the repo root as a base dir."""
    if os.path.isabs(path):
        return path

    return os.path.join(os.path.dirname(__file__), path)


def get_workdir(config: configparser.ConfigParser) -> str:
    """Gets the directory which is writable."""
    return get_abspath(config.get('wsgi', 'workdir').strip())


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


def tsv_to_list(sock: TextIO) -> List[List[yattag.Doc]]:
    """Turns a tab-separated table into a list of lists."""
    table = []

    first = True
    type_index = 0
    for line in sock.readlines():
        if not line.strip():
            continue
        if first:
            first = False
            for index, column in enumerate(line.split("\t")):
                if column.strip() == "@type":
                    type_index = index
        cells = [util.html_escape(cell.strip()) for cell in line.split("\t")]
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


def normalize(relation: Relation, house_numbers: str, street_name: str,
              normalizers: Dict[str, Ranges]) -> List[str]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    ret_numbers = []
    # Same as ret_numbers, but if the range is 2-6 and we filter for 2-4, then 6 would be lost, so
    # in-range 4 would not be detected, so this one does not drop 6.
    ret_numbers_nofilter = []
    if ';' in house_numbers:
        separator = ';'
    else:
        separator = '-'

    # Determine suffix which is not normalized away.
    suffix = ""
    if house_numbers.endswith("*"):
        suffix = house_numbers[-1]

    if street_name in normalizers.keys():
        # Have a custom filter.
        normalizer = normalizers[street_name]
    else:
        # Default sanity checks.
        default = [Range(1, 999), Range(2, 998)]
        normalizer = Ranges(default)

    for house_number in house_numbers.split(separator):
        try:
            number = int(re.sub(r"([0-9]+).*", r"\1", house_number))
        except ValueError:
            continue

        ret_numbers_nofilter.append(number)

        if number not in normalizer:
            continue

        ret_numbers.append(number)

    street_is_even_odd = relation.get_config().get_street_is_even_odd(street_name)
    if separator == "-" and util.should_expand_range(ret_numbers_nofilter, street_is_even_odd):
        start = ret_numbers_nofilter[0]
        stop = ret_numbers_nofilter[1]
        if stop == 0:
            ret_numbers = [number for number in [start] if number in normalizer]
        elif street_is_even_odd:
            # Assume that e.g. 2-6 actually means 2, 4 and 6, not only 2 and 4.
            # Closed interval, even only or odd only case.
            ret_numbers = [number for number in range(start, stop + 2, 2) if number in normalizer]
        else:
            # Closed interval, but mixed even and odd.
            ret_numbers = [number for number in range(start, stop + 1, 1) if number in normalizer]

    return [str(number) + suffix for number in ret_numbers]


def make_turbo_query_for_streets(relation: Relation, table: List[List[yattag.Doc]]) -> str:
    """Creates an overpass query that shows all streets from a missing housenumbers table."""
    streets = []  # type: List[str]
    first = True
    for row in table:
        if first:
            first = False
            continue
        streets.append(row[0].getvalue())
    header = """[out:json][timeout:425];
rel(@RELATION@)->.searchRelation;
area(@AREA@)->.searchArea;
("""
    query = util.process_template(header, relation.get_config().get_osmrelation())
    for street in streets:
        query += 'way["name"="' + street + '"](r.searchRelation);\n'
        query += 'way["name"="' + street + '"](area.searchArea);\n'
    query += """);
out body;
>;
out skel qt;"""
    return query


# vim:set shiftwidth=4 softtabstop=4 expandtab:
