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
from typing import Tuple
from typing import cast
import json
import yattag

from i18n import translate as tr
import area_files
import context
import ranges
import util

class RelationConfigBase:
    """A relation configuration comes directly from static data, not a result of some external query."""
    def __init__(self, parent_config: Dict[str, Any], my_config: Dict[str, Any]) -> None:
        self.__parent = parent_config
        self.__dict = my_config

    def get_property(self, key: str) -> Any:
        """Gets the untyped value of a property transparently."""
        if key in self.__dict.keys():
            return self.__dict[key]

        if key in self.__parent.keys():
            return self.__parent[key]

        return None

    def set_property(self, key: str, value: Any) -> None:
        """Sets an untyped value."""
        self.__dict[key] = value

    def set_active(self, active: bool) -> None:
        """Sets if the relation is active."""
        self.__dict["inactive"] = not active

    def is_active(self) -> bool:
        """Gets if the relation is active."""
        return not cast(bool, self.get_property("inactive"))

    def get_osmrelation(self) -> int:
        """Gets the OSM relation object's ID."""
        return cast(int, self.get_property("osmrelation"))

    def get_refcounty(self) -> str:
        """Gets the relation's refcounty identifier from reference."""
        return cast(str, self.get_property("refcounty"))

    def get_refsettlement(self) -> str:
        """Gets the relation's refsettlement identifier from reference."""
        return cast(str, self.get_property("refsettlement"))

    def get_alias(self) -> List[str]:
        """Gets the alias(es) of the relation: alternative names which are also accepted."""
        return cast(List[str], self.get_property("alias"))


class RelationConfig(RelationConfigBase):
    """A relation config extends RelationConfigBase with additional typed values."""
    def __init__(self, parent_config: Dict[str, Any], my_config: Dict[str, Any]) -> None:
        RelationConfigBase.__init__(self, parent_config, my_config)

    def should_check_missing_streets(self) -> str:
        """Return value can be 'yes', 'no' and 'only'."""
        if self.get_property("missing-streets"):
            return cast(str, self.get_property("missing-streets"))

        return "yes"

    def should_check_housenumber_letters(self) -> bool:
        """Do we care if 42/B is missing when 42/A is provided?."""
        if self.get_property("housenumber-letters"):
            return cast(bool, self.get_property("housenumber-letters"))

        return False

    def should_check_additional_housenumbers(self) -> bool:
        """Do we care if 42 is in OSM when it's not in the ref?."""
        if self.get_property("additional-housenumbers"):
            return cast(bool, self.get_property("additional-housenumbers"))

        return False

    def set_housenumber_letters(self, housenumber_letters: bool) -> None:
        """Sets the housenumber_letters property from code."""
        self.set_property("housenumber-letters", housenumber_letters)

    def set_letter_suffix_style(self, letter_suffix_style: util.LetterSuffixStyle) -> None:
        """Sets the letter suffix style."""
        self.set_property("letter-suffix-style", letter_suffix_style)

    def get_letter_suffix_style(self) -> util.LetterSuffixStyle:
        """Gets the letter suffix style."""
        if self.get_property("letter-suffix-style"):
            return cast(util.LetterSuffixStyle, self.get_property("letter-suffix-style"))
        return util.LetterSuffixStyle.UPPER

    def get_refstreets(self) -> Dict[str, str]:
        """Returns an OSM name -> ref name map."""
        if self.get_property("refstreets"):
            return cast(Dict[str, str], self.get_property("refstreets"))
        return {}

    def set_filters(self, filters: Dict[str, Any]) -> None:
        """Sets the 'filters' key from code."""
        self.set_property("filters", filters)

    def get_filters(self) -> Dict[str, Any]:
        """Returns a street name -> properties map."""
        if self.get_property("filters"):
            return cast(Dict[str, Any], self.get_property("filters"))
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
        ret = [self.get_property("refsettlement")]
        if not self.get_property("filters"):
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
        if self.get_property("street-filters"):
            return cast(List[str], self.get_property("street-filters"))
        return []

    def get_osm_street_filters(self) -> List[str]:
        """Gets list of streets which are only in OSM, but have to be filtered out."""
        if self.get_property("osm-street-filters"):
            return cast(List[str], self.get_property("osm-street-filters"))
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


class RelationBase:
    """A relation is a closed polygon on the map."""
    def __init__(
            self,
            ctx: context.Context,
            name: str,
            parent_config: Dict[str, Any],
            yaml_cache: Dict[str, Any]
    ) -> None:
        self.__ctx = ctx
        self.__name = name
        my_config: Dict[str, Any] = {}
        self.__file = area_files.RelationFiles(ctx.get_ini().get_workdir(), name)
        relation_path = "relation-%s.yaml" % name
        # Intentionally don't require this cache to be present, it's fine to omit it for simple
        # relations.
        if relation_path in yaml_cache:
            my_config = yaml_cache[relation_path]
        self.__config = RelationConfig(parent_config, my_config)
        # osm street name -> house number list map, so we don't have to read the on-disk list of the
        # relation again and again for each street.
        self.__osm_housenumbers: Dict[str, List[util.HouseNumber]] = {}
        self.ctx = ctx

    def get_name(self) -> str:
        """Gets the name of the relation."""
        return self.__name

    def get_files(self) -> area_files.RelationFiles:
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
        with self.get_files().get_osm_streets_csv_stream(self.__ctx) as sock:
            first = True
            for row in sock.get_rows():
                if first:
                    first = False
                    continue
                # 0: @id, 1: name, 6: @type
                street = util.Street(osm_id=int(row[0]), osm_name=row[1])
                if len(row) > 6:
                    street.set_osm_type(row[6])
                street.set_source(tr("street"))
                ret.append(street)
        if os.path.exists(self.get_files().get_osm_housenumbers_path()):
            with self.get_files().get_osm_housenumbers_csv_stream(self.__ctx) as sock:
                ret += util.get_street_from_housenumber(sock)
        if sorted_result:
            return sorted(set(ret))
        return ret

    def get_osm_streets_query(self) -> str:
        """Produces a query which lists streets in relation."""
        with open(os.path.join(self.__ctx.get_abspath("data"), "streets-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())

    def get_osm_housenumbers(self, street_name: str) -> List[util.HouseNumber]:
        """Gets the OSM house number list of a street."""
        if not self.__osm_housenumbers:
            # This function gets called for each & every street, make sure we read the file only
            # once.
            street_ranges = self.get_street_ranges()
            house_numbers: Dict[str, List[util.HouseNumber]] = {}
            with self.get_files().get_osm_housenumbers_csv_stream(self.__ctx) as sock:
                first = True
                columns: Dict[str, int] = {}
                for row in sock.get_rows():
                    if first:
                        first = False
                        for index, label in enumerate(row):
                            columns[label] = index
                        continue
                    if not row:
                        continue
                    street = row[columns["addr:street"]]
                    if not street and "addr:place" in columns:
                        street = row[columns["addr:place"]]
                    for house_number in row[columns["addr:housenumber"]].replace(',', ';').split(';'):
                        if street not in house_numbers:
                            house_numbers[street] = []
                        house_numbers[street] += normalize(self, house_number, street, street_ranges)
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
        with self.get_files().get_ref_streets_write_stream(self.__ctx) as sock:
            for line in lst:
                sock.write(util.to_bytes(line + "\n"))

    def get_ref_streets(self) -> List[str]:
        """Gets streets from reference."""
        streets: List[str] = []
        with self.get_files().get_ref_streets_read_stream(self.__ctx) as sock:
            for line in sock.readlines():
                line = line.strip()
                streets.append(util.from_bytes(line))
        return sorted(set(streets))

    def build_ref_housenumbers(
            self,
            reference: Dict[str, Dict[str, Dict[str, List[util.HouseNumberWithComment]]]],
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
                # i[0] is number, i[1] is comment
                ret += [street + "\t" + i[0] + suffix + "\t" + i[1] for i in house_numbers]

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
        used by get_ref_housenumbers().
        """
        memory_caches = util.build_reference_caches(references, self.get_config().get_refcounty())

        streets = [i.get_osm_name() for i in self.get_osm_streets()]

        lst: List[str] = []
        for street in streets:
            for index, memory_cache in enumerate(memory_caches):
                suffix = RelationBase.__get_ref_suffix(index)
                lst += self.build_ref_housenumbers(memory_cache, street, suffix)

        lst = sorted(set(lst))
        with self.get_files().get_ref_housenumbers_write_stream(self.__ctx) as sock:
            for line in lst:
                sock.write(util.to_bytes(line + "\n"))

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

    def get_ref_housenumbers(self) -> Dict[str, List[util.HouseNumber]]:
        """Gets house numbers from reference, produced by write_ref_housenumbers()."""
        ret: Dict[str, List[util.HouseNumber]] = {}
        lines: Dict[str, List[str]] = {}
        with self.get_files().get_ref_housenumbers_read_stream(self.ctx) as sock:
            for line_bytes in sock.readlines():
                line = util.from_bytes(line_bytes)
                line = line.strip()
                key, _, value = line.partition("\t")
                if key not in lines:
                    lines[key] = []
                lines[key].append(value)
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

            if ref_street_name in lines.keys():
                for line in lines[ref_street_name]:
                    house_number = line.replace(prefix, '')
                    normalized = normalize(self, house_number, osm_street_name, street_ranges)
                    normalized = \
                        [i for i in normalized if not util.HouseNumber.is_invalid(i.get_number(), street_invalid)]
                    house_numbers += normalized
            ret[osm_street_name] = util.sort_numerically(set(house_numbers))
        return ret

    def get_missing_housenumbers(self) -> Tuple[util.NumberedStreets, util.NumberedStreets]:
        """
        Compares ref and osm house numbers, prints the ones which are in ref, but not in osm.
        Return value is a pair of ongoing and done streets.
        Each of of these is a pair of a street name and a house number list.
        """
        ongoing_streets = []
        done_streets = []

        osm_street_names = self.get_osm_streets()
        all_ref_house_numbers = self.get_ref_housenumbers()
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

    def get_missing_streets(self) -> Tuple[List[str], List[str]]:
        """Tries to find missing streets in a relation."""
        reference_streets = [util.Street(i) for i in self.get_ref_streets()]
        street_blacklist = self.get_config().get_street_filters()
        osm_streets = [util.Street(get_ref_street_from_osm_street(self.get_config(), street.get_osm_name()))
                       for street in self.get_osm_streets()]

        only_in_reference = util.get_only_in_first(reference_streets, osm_streets)
        only_in_ref_names = [i.get_osm_name() for i in only_in_reference if i.get_osm_name() not in street_blacklist]
        in_both = [i.get_osm_name() for i in util.get_in_both(reference_streets, osm_streets)]

        return only_in_ref_names, in_both

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
        with self.get_files().get_streets_percent_write_stream(self.__ctx) as stream:
            stream.write(util.to_bytes(percent))

        return todo_count, done_count, percent, streets

    def write_additional_streets(self) -> List[util.Street]:
        """Calculate and write stat for the unexpected street coverage of a relation."""
        additional_streets = self.get_additional_streets()

        # Write the count to a file, so the index page show it fast.
        with self.get_files().get_streets_additional_count_write_stream(self.__ctx) as stream:
            stream.write(util.to_bytes(str(len(additional_streets))))

        return additional_streets


class Relation(RelationBase):
    """A relation extends RelationBase with additional functionality, like reverse diffing."""
    def __init__(
            self,
            ctx: context.Context,
            name: str,
            parent_config: Dict[str, Any],
            yaml_cache: Dict[str, Any]
    ) -> None:
        RelationBase.__init__(self, ctx, name, parent_config, yaml_cache)
        self.__ctx = ctx

    def get_street_valid(self) -> Dict[str, List[str]]:
        """Gets a street name -> valid map, which allows silencing individual false positives."""
        valid_dict: Dict[str, List[str]] = {}

        filters = self.get_config().get_filters()
        for street in filters.keys():
            if "valid" not in filters[street]:
                continue
            valid_dict[street] = filters[street]["valid"]

        return valid_dict

    def numbered_streets_to_table(
        self,
        numbered_streets: util.NumberedStreets
    ) -> Tuple[List[List[yattag.doc.Doc]], int]:
        """Turns a list of numbered streets into a HTML table."""
        todo_count = 0
        table = []
        table.append([util.html_escape(tr("Street name")),
                      util.html_escape(tr("Missing count")),
                      util.html_escape(tr("House numbers"))])
        rows = []
        for result in numbered_streets:
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
        return table, todo_count

    def write_missing_housenumbers(self) -> Tuple[int, int, int, str, List[List[yattag.doc.Doc]]]:
        """
        Calculate a write stat for the house number coverage of a relation.
        Returns a tuple of: todo street count, todo count, done count, percent and table.
        """
        ongoing_streets, done_streets = self.get_missing_housenumbers()

        table, todo_count = self.numbered_streets_to_table(ongoing_streets)

        done_count = 0
        for result in done_streets:
            number_ranges = util.get_housenumber_ranges(result[1])
            done_count += len(number_ranges)
        if done_count > 0 or todo_count > 0:
            percent = "%.2f" % (done_count / (done_count + todo_count) * 100)
        else:
            percent = "100.00"

        # Write the bottom line to a file, so the index page show it fast.
        with self.get_files().get_housenumbers_percent_write_stream(self.__ctx) as stream:
            stream.write(util.to_bytes(percent))

        return len(ongoing_streets), todo_count, done_count, percent, table

    def write_additional_housenumbers(self) -> Tuple[int, int, List[List[yattag.doc.Doc]]]:
        """
        Calculate and write stat for the unexpected house number coverage of a relation.
        Returns a tuple of: todo street count, todo count and table.
        """
        ongoing_streets = self.get_additional_housenumbers()

        table, todo_count = self.numbered_streets_to_table(ongoing_streets)

        # Write the street count to a file, so the index page show it fast.
        with self.get_files().get_housenumbers_additional_count_write_stream(self.ctx) as stream:
            stream.write(util.to_bytes(str(todo_count)))

        return len(ongoing_streets), todo_count, table

    def get_osm_housenumbers_query(self) -> str:
        """Produces a query which lists house numbers in relation."""
        with open(os.path.join(self.__ctx.get_abspath("data"), "street-housenumbers-template.txt")) as stream:
            return util.process_template(stream.read(), self.get_config().get_osmrelation())

    def get_invalid_refstreets(self) -> Tuple[List[str], List[str]]:
        """Returns invalid osm names and ref names."""
        osm_invalids: List[str] = []
        ref_invalids: List[str] = []
        refstreets = self.get_config().get_refstreets()
        osm_streets = [i.get_osm_name() for i in self.get_osm_streets()]
        for osm_name, ref_name in refstreets.items():
            if osm_name not in osm_streets:
                osm_invalids.append(osm_name)
            if ref_name in osm_streets:
                ref_invalids.append(ref_name)
        return osm_invalids, ref_invalids

    def get_invalid_filter_keys(self) -> List[str]:
        """Returns invalid filter key names (street not in OSM)."""
        invalids: List[str] = []
        keys = [key for key, value in self.get_config().get_filters().items()]
        osm_streets = [i.get_osm_name() for i in self.get_osm_streets()]
        for key in keys:
            if key not in osm_streets:
                invalids.append(key)
        return invalids

    def get_additional_housenumbers(self) -> util.NumberedStreets:
        """
        Compares ref and osm house numbers, prints the ones which are in osm, but not in ref.
        Return value is a list of streets.
        Each of of these is a pair of a street name and a house number list.
        """
        additional = []

        osm_street_names = self.get_osm_streets()
        all_ref_house_numbers = self.get_ref_housenumbers()
        streets_valid = self.get_street_valid()
        for osm_street in osm_street_names:
            osm_street_name = osm_street.get_osm_name()
            ref_house_numbers = all_ref_house_numbers[osm_street_name]
            osm_house_numbers = self.get_osm_housenumbers(osm_street_name)

            if osm_street_name in streets_valid.keys():
                street_valid = streets_valid[osm_street_name]
                osm_house_numbers = \
                    [i for i in osm_house_numbers if not util.HouseNumber.is_invalid(i.get_number(), street_valid)]

            only_in_osm = util.get_only_in_first(osm_house_numbers, ref_house_numbers)
            ref_street_name = get_ref_street_from_osm_street(self.get_config(), osm_street_name)
            street = util.Street(osm_street_name, ref_street_name, self.should_show_ref_street(osm_street_name))
            if only_in_osm:
                additional.append((street, only_in_osm))
        # Sort by length.
        additional.sort(key=lambda result: len(result[1]), reverse=True)

        return additional


class Relations:
    """A relations object is a container of named relation objects."""
    def __init__(self, ctx: context.Context) -> None:
        self.__workdir = ctx.get_ini().get_workdir()
        self.__ctx = ctx
        with ctx.get_file_system().open_read(os.path.join(ctx.get_abspath("data"), "yamls.cache")) as stream:
            self.__yaml_cache: Dict[str, Any] = json.load(stream)
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
            self.__relations[name] = Relation(self.__ctx,
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
        relation: RelationBase,
        house_numbers: str,
        suffix: str,
        comment: str
) -> List[util.HouseNumber]:
    """Handles the part of normalize() that deals with housenumber letters."""
    style = relation.get_config().get_letter_suffix_style()
    normalized = util.HouseNumber.normalize_letter_suffix(house_numbers, suffix, style)
    return [util.HouseNumber(normalized, normalized, comment)]


def normalize(relation: RelationBase, house_numbers: str, street_name: str,
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
(rel(@RELATION@);
"""
    query = util.process_template(header, relation.get_config().get_osmrelation())
    for street in streets:
        query += 'way["name"="' + street + '"](r.searchRelation);\n'
        query += 'way["name"="' + street + '"](area.searchArea);\n'
    query += """);
out body;
>;
out skel qt;
{{style:
relation{width:3}
way{color:blue; width:4;}
}}"""
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
