#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The areas module contains the Relations class and associated functionality."""

import os
from typing import Any
from typing import Dict
from typing import List
from typing import TextIO
from typing import Tuple
from typing import cast
import pickle
import yattag

from i18n import translate as _
import ranges
import util


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
        result = util.sort_streets_csv(result_from_overpass)
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
        result = util.sort_housenumbers_csv(result_from_overpass)
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

    def set_active(self, active: bool) -> None:
        """Sets if the relation is active."""
        self.__dict["inactive"] = not active

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

    def get_alias(self) -> List[str]:
        """Gets the alias(es) of the relation: alternative names which are also accepted."""
        return cast(List[str], self.__get_property("alias"))

    def should_check_missing_streets(self) -> str:
        """Return value can be 'yes', 'no' and 'only'."""
        if self.__get_property("missing-streets"):
            return cast(str, self.__get_property("missing-streets"))

        return "yes"

    def should_check_housenumber_letters(self) -> bool:
        """Do we care if 42/B is missing when 42/A is provided?."""
        if self.__get_property("housenumber-letters"):
            return cast(bool, self.__get_property("housenumber-letters"))

        return False

    def set_housenumber_letters(self, housenumber_letters: bool) -> None:
        """Sets the housenumber_letters property from code."""
        self.__dict["housenumber-letters"] = housenumber_letters

    def set_letter_suffix_style(self, letter_suffix_style: util.LetterSuffixStyle) -> None:
        """Sets the letter suffix style."""
        self.__dict["letter-suffix-style"] = letter_suffix_style

    def get_letter_suffix_style(self) -> util.LetterSuffixStyle:
        """Gets the letter suffix style."""
        if self.__get_property("letter-suffix-style"):
            return cast(util.LetterSuffixStyle, self.__get_property("letter-suffix-style"))
        return util.LetterSuffixStyle.UPPER

    def get_refstreets(self) -> Dict[str, str]:
        """Returns an OSM name -> ref name map."""
        if self.__get_property("refstreets"):
            return cast(Dict[str, str], self.__get_property("refstreets"))
        return {}

    def set_filters(self, filters: Dict[str, Any]) -> None:
        """Sets the 'filters' key from code."""
        self.__dict["filters"] = filters

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
    def __init__(
            self,
            workdir: str,
            name: str,
            parent_config: Dict[str, Any],
            yaml_cache: Dict[str, Any]
    ) -> None:
        self.__workdir = workdir
        self.__name = name
        my_config: Dict[str, Any] = {}
        self.__file = RelationFiles(util.get_abspath("data"), workdir, name)
        relation_path = "relation-%s.yaml" % name
        # Intentionally don't require this cache to be present, it's fine to omit it for simple
        # relations.
        if relation_path in yaml_cache:
            my_config = yaml_cache[relation_path]
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

    def get_street_ranges(self) -> Dict[str, ranges.Ranges]:
        """Gets a street name -> ranges map, which allows silencing false positives."""
        filter_dict: Dict[str, ranges.Ranges] = {}

        filters = self.get_config().get_filters()
        for street in filters.keys():
            interpolation = ""
            if "interpolation" in filters[street]:
                interpolation = filters[street]["interpolation"]
            i = []
            if "ranges" not in filters[street]:
                continue
            for start_end in filters[street]["ranges"]:
                i.append(ranges.Range(int(start_end["start"]), int(start_end["end"]), interpolation))
            filter_dict[street] = ranges.Ranges(i)

        return filter_dict

    def get_street_invalid(self) -> Dict[str, List[str]]:
        """Gets a street name -> invalid map, which allows silencing individual false positives."""
        invalid_dict: Dict[str, List[str]] = {}

        filters = self.get_config().get_filters()
        for street in filters.keys():
            if "invalid" not in filters[street]:
                continue
            invalid_dict[street] = filters[street]["invalid"]

        return invalid_dict

    def get_ref_street_from_osm_street(self, osm_street_name: str) -> str:
        """Maps an OSM street name to a ref street name."""
        refstreets = self.get_config().get_refstreets()

        if osm_street_name in refstreets.keys():
            return refstreets[osm_street_name]

        return osm_street_name

    def get_osm_streets(self) -> List[str]:
        """Reads list of streets for an area from OSM."""
        ret: List[str] = []
        with self.get_files().get_osm_streets_stream("r") as sock:
            ret += util.get_nth_column(sock, 1)
        if os.path.exists(self.get_files().get_osm_housenumbers_path()):
            with self.get_files().get_osm_housenumbers_stream("r") as sock:
                ret += util.get_nth_column(sock, 1)
        return sorted(set(ret))

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
        datadir = util.get_abspath("data")
        with open(os.path.join(datadir, "streets-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())

    def get_osm_housenumbers(self, street_name: str) -> List[util.HouseNumber]:
        """Gets the OSM house number list of a street."""
        house_numbers: List[util.HouseNumber] = []
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
                for house_number in tokens[2].split(';'):
                    house_numbers += normalize(self, house_number, street_name, self.get_street_ranges())
        return util.sort_numerically(set(house_numbers))

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
        # Convert relative path to absolute one.
        reference = util.get_abspath(reference)

        memory_cache = util.build_street_reference_cache(reference)

        lst = self.build_ref_streets(memory_cache)

        lst = sorted(set(lst))
        with self.get_files().get_ref_streets_stream("w") as sock:
            for line in lst:
                sock.write(line + "\n")

    def get_ref_streets(self) -> List[str]:
        """Gets streets from reference."""
        streets: List[str] = []
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
        ret: List[str] = []
        for reftelepules in self.get_config().get_street_reftelepules(street):
            if refmegye not in reference.keys():
                continue
            refmegye_dict = reference[refmegye]
            if reftelepules not in refmegye_dict.keys():
                continue
            reftelepules_dict = refmegye_dict[reftelepules]
            if street in reftelepules_dict.keys():
                house_numbers = reference[refmegye][reftelepules][street]
                ret += [street + "\t" + i + suffix for i in house_numbers]

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
        references = [util.get_abspath(reference) for reference in references]

        memory_caches = util.build_reference_caches(references)

        streets = self.get_osm_streets()

        lst: List[str] = []
        for street in streets:
            for index, memory_cache in enumerate(memory_caches):
                suffix = Relation.__get_ref_suffix(index)
                lst += self.build_ref_housenumbers(memory_cache, street, suffix)

        lst = sorted(set(lst))
        with self.get_files().get_ref_housenumbers_stream("w") as sock:
            for line in lst:
                sock.write(line + "\n")

    def __get_ref_housenumbers(self) -> Dict[str, List[util.HouseNumber]]:
        """Gets house numbers from reference, produced by write_ref_housenumbers()."""
        ret: Dict[str, List[util.HouseNumber]] = {}
        lines: List[str] = []
        with self.get_files().get_ref_housenumbers_stream("r") as sock:
            for line in sock.readlines():
                line = line.strip()
                lines.append(line)
        street_ranges = self.get_street_ranges()
        streets_invalid = self.get_street_invalid()
        for osm_street_name in self.get_osm_streets():
            house_numbers: List[util.HouseNumber] = []
            ref_street_name = self.get_ref_street_from_osm_street(osm_street_name)
            prefix = ref_street_name + "\t"
            street_invalid: List[str] = []
            if osm_street_name in streets_invalid.keys():
                street_invalid = streets_invalid[osm_street_name]
            for line in lines:
                if line.startswith(prefix):
                    house_number = line.replace(prefix, '')
                    normalized = normalize(self, house_number, osm_street_name, street_ranges)
                    normalized = \
                        [i for i in normalized if not util.HouseNumber.is_invalid(i.get_number(), street_invalid)]
                    house_numbers += normalized
            ret[osm_street_name] = util.sort_numerically(set(house_numbers))
        return ret

    def get_missing_housenumbers(
            self
    ) -> Tuple[List[Tuple[str, List[util.HouseNumber]]], List[Tuple[str, List[util.HouseNumber]]]]:
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
            only_in_reference = util.get_only_in_first(ref_house_numbers, osm_house_numbers)
            in_both = util.get_in_both(ref_house_numbers, osm_house_numbers)
            if only_in_reference:
                ongoing_streets.append((street_name, only_in_reference))
            if in_both:
                done_streets.append((street_name, in_both))
        # Sort by length.
        ongoing_streets.sort(key=lambda result: len(result[1]), reverse=True)

        return ongoing_streets, done_streets

    def write_missing_housenumbers(self) -> Tuple[int, int, int, str, List[List[yattag.Doc]]]:
        """
        Calculate a write stat for the house number coverage of a relation.
        Returns a tuple of: todo street count, todo count, done count, percent and table.
        """
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
            number_ranges = util.get_housenumber_ranges(result[1])
            row.append(util.html_escape(str(len(number_ranges))))

            doc = yattag.Doc()
            if not self.get_config().get_street_is_even_odd(result[0]):
                for index, item in enumerate(sorted(number_ranges, key=util.split_house_number)):
                    if index:
                        doc.text(", ")
                    doc.asis(util.color_house_number(item).getvalue())
            else:
                util.format_even_odd(number_ranges, doc)
            row.append(doc)

            todo_count += len(number_ranges)
            table.append(row)
        done_count = 0
        for result in done_streets:
            number_ranges = util.get_housenumber_ranges(result[1])
            done_count += len(number_ranges)
        if done_count > 0 or todo_count > 0:
            percent = "%.2f" % (done_count / (done_count + todo_count) * 100)
        else:
            percent = "100.00"

        # Write the bottom line to a file, so the index page show it fast.
        with self.get_files().get_housenumbers_percent_stream("w") as stream:
            stream.write(percent)

        return len(ongoing_streets), todo_count, done_count, percent, table

    def get_missing_streets(self) -> Tuple[List[str], List[str]]:
        """Tries to find missing streets in a relation."""
        reference_streets = self.get_ref_streets()
        street_blacklist = self.get_config().get_street_filters()
        osm_streets = [self.get_ref_street_from_osm_street(street) for street in self.get_osm_streets()]

        only_in_reference = util.get_only_in_first(reference_streets, osm_streets)
        only_in_reference = [i for i in only_in_reference if i not in street_blacklist]
        in_both = util.get_in_both(reference_streets, osm_streets)

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
        datadir = util.get_abspath("data")
        with open(os.path.join(datadir, "street-housenumbers-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())


class Relations:
    """A relations object is a container of named relation objects."""
    def __init__(self, workdir: str) -> None:
        self.__workdir = workdir
        datadir = util.get_abspath("data")
        with open(os.path.join(datadir, "yamls.pickle"), "rb") as stream:
            self.__yaml_cache: Dict[str, Any] = pickle.load(stream)
        self.__dict = self.__yaml_cache["relations.yaml"]
        self.__relations: Dict[str, Relation] = {}
        self.__activate_all = False
        self.__refmegye_names = self.__yaml_cache["refmegye-names.yaml"]
        self.__reftelepules_names = self.__yaml_cache["reftelepules-names.yaml"]

    def get_workdir(self) -> str:
        """Gets the workdir directory path."""
        return self.__workdir

    def get_relation(self, name: str) -> Relation:
        """Gets the relation that has the specified name."""
        if name not in self.__relations.keys():
            if name not in self.__dict.keys():
                self.__dict[name] = {}
            self.__relations[name] = Relation(self.__workdir,
                                              name,
                                              self.__dict[name],
                                              self.__yaml_cache)
        return self.__relations[name]

    def get_names(self) -> List[str]:
        """Gets a sorted list of relation names."""
        return sorted(self.__dict.keys())

    def get_active_names(self) -> List[str]:
        """Gets a sorted list of active relation names."""
        ret: List[Relation] = []
        for relation in self.get_relations():
            if self.__activate_all or relation.get_config().is_active():
                ret.append(relation)
        return sorted([relation.get_name() for relation in ret])

    def get_relations(self) -> List[Relation]:
        """Gets a list of relations."""
        ret: List[Relation] = []
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

    def get_aliases(self) -> Dict[str, str]:
        """Provide an alias -> real name map of relations."""
        ret: Dict[str, str] = {}
        for relation in self.get_relations():
            aliases = relation.get_config().get_alias()
            if aliases:
                name = relation.get_name()
                for alias in aliases:
                    ret[alias] = name
        return ret


def normalize(relation: Relation, house_numbers: str, street_name: str,
              normalizers: Dict[str, ranges.Ranges]) -> List[util.HouseNumber]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    if ';' in house_numbers:
        separator = ';'
    else:
        separator = '-'

    # Determine suffix which is not normalized away.
    suffix = ""
    if house_numbers.endswith("*"):
        suffix = house_numbers[-1]

    normalizer = util.get_normalizer(street_name, normalizers)

    ret_numbers, ret_numbers_nofilter = util.split_house_number_by_separator(house_numbers, separator, normalizer)

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

    check_housenumber_letters = len(ret_numbers) == 1 and relation.get_config().should_check_housenumber_letters()
    if check_housenumber_letters and util.HouseNumber.has_letter_suffix(house_numbers, suffix):
        style = relation.get_config().get_letter_suffix_style()
        normalized = util.HouseNumber.normalize_letter_suffix(house_numbers, suffix, style)
        return [util.HouseNumber(normalized, normalized)]
    return [util.HouseNumber(str(number) + suffix, house_numbers) for number in ret_numbers]


def make_turbo_query_for_streets(relation: Relation, table: List[List[yattag.Doc]]) -> str:
    """Creates an overpass query that shows all streets from a missing housenumbers table."""
    streets: List[str] = []
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
