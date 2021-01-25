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
from typing import Optional
from typing import TextIO
from typing import Tuple
from typing import cast
import pickle
import yattag

from i18n import translate as _
import config
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

    def __get_osm_streets_stream(self, mode: str) -> TextIO:
        """Opens the OSM street list of a relation."""
        path = self.get_osm_streets_path()
        return cast(TextIO, open(path, mode=mode))

    def get_osm_streets_csv_stream(self) -> util.CsvIO:
        """Gets a CSV reader for the OSM street list."""
        return util.CsvIO(self.__get_osm_streets_stream("r"))

    def write_osm_streets(self, result_from_overpass: str) -> None:
        """Writes the result for overpass of Relation.get_osm_streets_query()."""
        result = util.sort_streets_csv(result_from_overpass)
        with self.__get_osm_streets_stream("w") as sock:
            sock.write(result)

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        return os.path.join(self.__workdir, "street-housenumbers-%s.csv" % self.__name)

    def __get_osm_housenumbers_stream(self, mode: str) -> TextIO:
        """Opens the OSM house number list of a relation."""
        path = self.get_osm_housenumbers_path()
        return cast(TextIO, open(path, mode=mode))

    def get_osm_housenumbers_csv_stream(self) -> util.CsvIO:
        """Gets a CSV reader for the OSM house number list."""
        return util.CsvIO(self.__get_osm_housenumbers_stream("r"))

    def write_osm_housenumbers(self, result_from_overpass: str) -> None:
        """Writes the result for overpass of Relation.get_osm_housenumbers_query()."""
        result = util.sort_housenumbers_csv(result_from_overpass)
        with self.__get_osm_housenumbers_stream(mode="w") as stream:
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

    def get_streets_additional_count_path(self) -> str:
        """Builds the file name of the street additional count file of a relation."""
        return os.path.join(self.__workdir, "%s-additional-streets.count" % self.__name)

    def get_streets_additional_count_stream(self, mode: str) -> TextIO:
        """Opens the street additional count file of a relation."""
        return cast(TextIO, open(self.get_streets_additional_count_path(), mode=mode))


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

    def get_refcounty(self) -> str:
        """Gets the relation's refcounty identifier from reference."""
        return cast(str, self.__get_property("refcounty"))

    def get_refsettlement(self) -> str:
        """Gets the relation's refsettlement identifier from reference."""
        return cast(str, self.__get_property("refsettlement"))

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

    def get_street_refsettlement(self, street: str) -> List[str]:
        """Returns a list of refsettlement values specific to a street."""
        ret = [self.__get_property("refsettlement")]
        if not self.__get_property("filters"):
            return ret

        relation_filters = self.get_filters()
        for filter_street, value in relation_filters.items():
            if filter_street != street:
                continue

            if "refsettlement" in value.keys():
                refsettlement = cast(str, value["refsettlement"])
                ret = [refsettlement]
            if "ranges" in value.keys():
                for street_range in value["ranges"]:
                    street_range_dict = cast(Dict[str, str], street_range)
                    if "refsettlement" in street_range_dict.keys():
                        ret.append(street_range_dict["refsettlement"])

        return sorted(set(ret))

    def get_street_filters(self) -> List[str]:
        """Gets list of streets which are only in reference, but have to be filtered out."""
        if self.__get_property("street-filters"):
            return cast(List[str], self.__get_property("street-filters"))
        return []

    def get_osm_street_filters(self) -> List[str]:
        """Gets list of streets which are only in OSM, but have to be filtered out."""
        if self.__get_property("osm-street-filters"):
            return cast(List[str], self.__get_property("osm-street-filters"))
        return []

    def build_ref_streets(self, reference: Dict[str, Dict[str, List[str]]]) -> List[str]:
        """
        Builds a list of streets from a reference cache.
        """
        refcounty = self.get_refcounty()
        refsettlement = self.get_refsettlement()
        return reference[refcounty][refsettlement]


def get_ref_street_from_osm_street(relation_config: RelationConfig, osm_street_name: str) -> str:
    """Maps an OSM street name to a ref street name."""
    refstreets = relation_config.get_refstreets()

    if osm_street_name in refstreets.keys():
        return refstreets[osm_street_name]

    return osm_street_name


def get_osm_street_from_ref_street(relation_config: RelationConfig, ref_street_name: str) -> str:
    """Maps a reference street name to an OSM street name."""
    refstreets = relation_config.get_refstreets()
    reverse = {v: k for k, v in refstreets.items()}

    if ref_street_name in reverse.keys():
        return reverse[ref_street_name]

    return ref_street_name


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
        self.__file = RelationFiles(config.get_abspath("data"), workdir, name)
        relation_path = "relation-%s.yaml" % name
        # Intentionally don't require this cache to be present, it's fine to omit it for simple
        # relations.
        if relation_path in yaml_cache:
            my_config = yaml_cache[relation_path]
        self.__config = RelationConfig(parent_config, my_config)
        # osm street name -> house number list map, so we don't have to read the on-disk list of the
        # relation again and again for each street.
        self.__osm_housenumbers: Dict[str, List[util.HouseNumber]] = {}

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

    def should_show_ref_street(self, osm_street_name: str) -> bool:
        """Decides is a ref street should be shown for an OSM street."""
        street_props = self.get_config().get_filter_street(osm_street_name)
        show_ref_street = True
        if "show-refstreet" in street_props:
            show_ref_street = street_props["show-refstreet"]

        return show_ref_street

    def get_osm_streets(self, sorted_result: bool = True) -> List[util.Street]:
        """Reads list of streets for an area from OSM."""
        ret: List[util.Street] = []
        with self.get_files().get_osm_streets_csv_stream() as sock:
            first = True
            for row in sock.get_rows():
                if first:
                    first = False
                    continue
                # 0: @id, 1: name, 6: @type
                street = util.Street(osm_id=int(row[0]), osm_name=row[1])
                if len(row) > 6:
                    street.set_osm_type(row[6])
                street.set_source(_("street"))
                ret.append(street)
        if os.path.exists(self.get_files().get_osm_housenumbers_path()):
            with self.get_files().get_osm_housenumbers_csv_stream() as sock:
                ret += util.get_street_from_housenumber(sock, 1, 2, 5)
        if sorted_result:
            return sorted(set(ret))
        return ret

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
        datadir = config.get_abspath("data")
        with open(os.path.join(datadir, "streets-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())

    def get_osm_housenumbers(self, street_name: str) -> List[util.HouseNumber]:
        """Gets the OSM house number list of a street."""
        if not self.__osm_housenumbers:
            # This function gets called for each & every street, make sure we read the file only
            # once.
            house_numbers: Dict[str, List[util.HouseNumber]] = {}
            with self.get_files().get_osm_housenumbers_csv_stream() as sock:
                first = True
                for row in sock.get_rows():
                    if first:
                        first = False
                        continue
                    if len(row) < 3:
                        continue
                    street = row[1]
                    for house_number in row[2].split(';'):
                        if street not in house_numbers:
                            house_numbers[street] = []
                        house_numbers[street] += normalize(self, house_number, street, self.get_street_ranges())
            for key, value in house_numbers.items():
                self.__osm_housenumbers[key] = util.sort_numerically(set(value))
        if street_name not in self.__osm_housenumbers:
            self.__osm_housenumbers[street_name] = []
        return self.__osm_housenumbers[street_name]

    def write_ref_streets(self, reference: str) -> None:
        """Gets known streets (not their coordinates) from a reference site, based on relation names
        from OSM."""
        memory_cache = util.build_street_reference_cache(reference)

        lst = self.get_config().build_ref_streets(memory_cache)

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
            reference: Dict[str, Dict[str, Dict[str, List[util.HouseNumberRange]]]],
            street: str,
            suffix: str
    ) -> List[str]:
        """
        Builds a list of housenumbers from a reference cache.
        This is serialized to disk by write_ref_housenumbers().
        """
        refcounty = self.get_config().get_refcounty()
        street = get_ref_street_from_osm_street(self.get_config(), street)
        ret: List[str] = []
        for refsettlement in self.get_config().get_street_refsettlement(street):
            if refcounty not in reference.keys():
                continue
            refcounty_dict = reference[refcounty]
            if refsettlement not in refcounty_dict.keys():
                continue
            refsettlement_dict = refcounty_dict[refsettlement]
            if street in refsettlement_dict.keys():
                house_numbers = reference[refcounty][refsettlement][street]
                ret += [street + "\t" + i.get_number() + suffix + "\t" + i.get_comment() for i in house_numbers]

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
        memory_caches = util.build_reference_caches(references, self.get_config().get_refcounty())

        streets = [i.get_osm_name() for i in self.get_osm_streets()]

        lst: List[str] = []
        for street in streets:
            for index, memory_cache in enumerate(memory_caches):
                suffix = Relation.__get_ref_suffix(index)
                lst += self.build_ref_housenumbers(memory_cache, street, suffix)

        lst = sorted(set(lst))
        with self.get_files().get_ref_housenumbers_stream("w") as sock:
            for line in lst:
                sock.write(line + "\n")

    def __normalize_invalids(self, osm_street_name: str, street_invalid: List[str]) -> List[str]:
        """Normalizes an 'invalid' list."""
        if self.get_config().should_check_housenumber_letters():
            return street_invalid

        normalized_invalid: List[str] = []
        street_ranges = self.get_street_ranges()
        for i in street_invalid:
            normalizeds = normalize(self, i, osm_street_name, street_ranges)
            # normalize() may return an empty list if the number is out of range.
            if normalizeds:
                normalized_invalid.append(normalizeds[0].get_number())
        return normalized_invalid

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
        for osm_street in self.get_osm_streets():
            osm_street_name = osm_street.get_osm_name()
            house_numbers: List[util.HouseNumber] = []
            ref_street_name = get_ref_street_from_osm_street(self.get_config(), osm_street_name)
            prefix = ref_street_name + "\t"
            street_invalid: List[str] = []
            if osm_street_name in streets_invalid.keys():
                street_invalid = streets_invalid[osm_street_name]

                # Simplify invalid items by default, so the 42a markup can be used, no matter what
                # is the value of housenumber-letters.
                street_invalid = self.__normalize_invalids(osm_street_name, street_invalid)

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
    ) -> Tuple[List[Tuple[util.Street, List[util.HouseNumber]]], List[Tuple[util.Street, List[util.HouseNumber]]]]:
        """
        Compares ref and osm house numbers, prints the ones which are in ref, but not in osm.
        Return value is a pair of ongoing and done streets.
        Each of of these is a pair of a street name and a house number list.
        """
        ongoing_streets = []
        done_streets = []

        osm_street_names = self.get_osm_streets()
        all_ref_house_numbers = self.__get_ref_housenumbers()
        for osm_street in osm_street_names:
            osm_street_name = osm_street.get_osm_name()
            ref_house_numbers = all_ref_house_numbers[osm_street_name]
            osm_house_numbers = self.get_osm_housenumbers(osm_street_name)
            only_in_reference = util.get_only_in_first(ref_house_numbers, osm_house_numbers)
            in_both = util.get_in_both(ref_house_numbers, osm_house_numbers)
            ref_street_name = get_ref_street_from_osm_street(self.get_config(), osm_street_name)
            street = util.Street(osm_street_name, ref_street_name, self.should_show_ref_street(osm_street_name))
            if only_in_reference:
                ongoing_streets.append((street, only_in_reference))
            if in_both:
                done_streets.append((street, in_both))
        # Sort by length.
        ongoing_streets.sort(key=lambda result: len(result[1]), reverse=True)

        return ongoing_streets, done_streets

    def write_missing_housenumbers(self) -> Tuple[int, int, int, str, List[List[yattag.doc.Doc]]]:
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
        rows = []
        for result in ongoing_streets:
            # street, only_in_ref
            row = []
            row.append(result[0].to_html())
            number_ranges = util.get_housenumber_ranges(result[1])
            row.append(util.html_escape(str(len(number_ranges))))

            doc = yattag.doc.Doc()
            if not self.get_config().get_street_is_even_odd(result[0].get_osm_name()):
                for index, item in enumerate(sorted(number_ranges, key=util.split_house_number_range)):
                    if index:
                        doc.text(", ")
                    doc.asis(util.color_house_number(item).getvalue())
            else:
                util.format_even_odd(number_ranges, doc)
            row.append(doc)

            todo_count += len(number_ranges)
            rows.append(row)

        # It's possible that get_housenumber_ranges() reduces the # of house numbers, e.g. 2, 4 and
        # 6 may be turned into 2-6, which is just 1 item. Sort by the 2nd col, which is the new
        # number of items.
        table += sorted(rows, reverse=True, key=lambda cells: int(cells[1].getvalue()))

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
        osm_streets = [get_ref_street_from_osm_street(self.get_config(), street.get_osm_name())
                       for street in self.get_osm_streets()]

        only_in_reference = util.get_only_in_first(reference_streets, osm_streets)
        only_in_reference = [i for i in only_in_reference if i not in street_blacklist]
        in_both = util.get_in_both(reference_streets, osm_streets)

        return only_in_reference, in_both

    def get_additional_streets(self, sorted_result: bool = True) -> List[util.Street]:
        """Tries to find additional streets in a relation."""
        ref_streets = [get_osm_street_from_ref_street(self.get_config(), street) for street in self.get_ref_streets()]
        ref_street_objs = [util.Street(i) for i in ref_streets]
        osm_streets = self.get_osm_streets(sorted_result)
        osm_street_blacklist = self.get_config().get_osm_street_filters()

        only_in_osm = util.get_only_in_first(osm_streets, ref_street_objs)
        only_in_osm = [i for i in only_in_osm if i.get_osm_name() not in osm_street_blacklist]

        return only_in_osm

    def write_missing_streets(self) -> Tuple[int, int, str, List[str]]:
        """Calculate and write stat for the street coverage of a relation."""
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

    def write_additional_streets(self) -> List[util.Street]:
        """Calculate aand write stat for the unexpected street coverage of a relation."""
        additional_streets = self.get_additional_streets()

        # Write the count to a file, so the index page show it fast.
        with self.get_files().get_streets_additional_count_stream("w") as stream:
            stream.write(str(len(additional_streets)))

        return additional_streets

    def get_osm_housenumbers_query(self) -> str:
        """Produces a query which lists house numbers in relation."""
        datadir = config.get_abspath("data")
        with open(os.path.join(datadir, "street-housenumbers-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())


class Relations:
    """A relations object is a container of named relation objects."""
    def __init__(self, workdir: str) -> None:
        self.__workdir = workdir
        datadir = config.get_abspath("data")
        with open(os.path.join(datadir, "yamls.pickle"), "rb") as stream:
            self.__yaml_cache: Dict[str, Any] = pickle.load(stream)
        self.__dict = self.__yaml_cache["relations.yaml"]
        self.__relations: Dict[str, Relation] = {}
        self.__activate_all = False
        self.__refcounty_names = self.__yaml_cache["refcounty-names.yaml"]
        self.__refsettlement_names = self.__yaml_cache["refsettlement-names.yaml"]

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

    def limit_to_refcounty(self, refcounty: Optional[str]) -> None:
        """If refcounty is not None, forget about all relations outside that refcounty."""
        if not refcounty:
            return
        for relation_name in list(self.__dict.keys()):
            relation = self.get_relation(relation_name)
            if relation.get_config().get_refcounty() == refcounty:
                continue
            del self.__dict[relation_name]

    def limit_to_refsettlement(self, refsettlement: Optional[str]) -> None:
        """If refsettlement is not None, forget about all relations outside that refsettlement."""
        if not refsettlement:
            return
        for relation_name in list(self.__dict.keys()):
            relation = self.get_relation(relation_name)
            if relation.get_config().get_refsettlement() == refsettlement:
                continue
            del self.__dict[relation_name]

    def refcounty_get_name(self, refcounty: str) -> str:
        """Produces a UI name for a refcounty."""
        if refcounty in self.__refcounty_names:
            return cast(str, self.__refcounty_names[refcounty])

        return ""

    def refcounty_get_refsettlement_ids(self, refcounty_name: str) -> List[str]:
        """Produces refsettlement IDs of a refcounty."""
        if refcounty_name not in self.__refsettlement_names:
            return []

        refcounty = self.__refsettlement_names[refcounty_name]
        return list(refcounty.keys())

    def refsettlement_get_name(self, refcounty_name: str, refsettlement: str) -> str:
        """Produces a UI name for a refsettlement in refcounty."""
        if refcounty_name not in self.__refsettlement_names:
            return ""

        refcounty = self.__refsettlement_names[refcounty_name]
        if refsettlement not in refcounty:
            return ""

        return cast(str, refcounty[refsettlement])

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


def normalize_housenumber_letters(
        relation: Relation,
        house_numbers: str,
        suffix: str,
        comment: str
) -> List[util.HouseNumber]:
    """Handles the part of normalize() that deals with housenumber letters."""
    style = relation.get_config().get_letter_suffix_style()
    normalized = util.HouseNumber.normalize_letter_suffix(house_numbers, suffix, style)
    return [util.HouseNumber(normalized, normalized, comment)]


def normalize(relation: Relation, house_numbers: str, street_name: str,
              normalizers: Dict[str, ranges.Ranges]) -> List[util.HouseNumber]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    comment = ""
    if "\t" in house_numbers:
        house_numbers, comment = house_numbers.split("\t")
    if ';' in house_numbers:
        separator = ';'
    elif ',' in house_numbers:
        separator = ','
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
        return normalize_housenumber_letters(relation, house_numbers, suffix, comment)
    return [util.HouseNumber(str(number) + suffix, house_numbers, comment) for number in ret_numbers]


def make_turbo_query_for_streets(relation: Relation, streets: List[str]) -> str:
    """Creates an overpass query that shows all streets from a missing housenumbers table."""
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


def make_turbo_query_for_street_objs(relation: Relation, streets: List[util.Street]) -> str:
    """Creates an overpass query that shows all streets from a list."""
    header = """[out:json][timeout:425];
rel(@RELATION@)->.searchRelation;
area(@AREA@)->.searchArea;
("""
    query = util.process_template(header, relation.get_config().get_osmrelation())
    ids = []
    for street in streets:
        ids.append((street.get_osm_type(), str(street.get_osm_id())))
    for osm_type, osm_id in sorted(set(ids)):
        query += osm_type + "(" + osm_id + ");\n"
    query += """);
out body;
>;
out skel qt;"""
    return query

# vim:set shiftwidth=4 softtabstop=4 expandtab:
