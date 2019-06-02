#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The suspicious_streets module tries to find streets which do have at least one house number, but
suspicious as lots of house numbers are probably missing."""

import os
import re
import sys
# pylint: disable=unused-import
from typing import Dict, List
import configparser
import yaml
import helpers


class Finder:
    """Compares reference house numbers with OSM ones and shows the diff."""
    def __init__(self, datadir, workdir, relation_name):
        self.normalizers = {}  # type: Dict[str, helpers.Ranges]
        # OSM name -> ref name map
        self.ref_streets = {}  # type: Dict[str, str]

        self.normalizers, self.ref_streets, self.simplify = helpers.load_normalizers(datadir, relation_name)
        street_names = helpers.get_streets(workdir, relation_name)

        results = []
        both_results = []

        for street_name in street_names:

            ref_street = street_name
            # See if we need to map the OSM name to ref name.
            if street_name in self.ref_streets.keys():
                ref_street = self.ref_streets[street_name]

            reference_house_numbers = self.get_house_numbers_from_lst(workdir, relation_name, street_name, ref_street)
            osm_house_numbers = self.get_house_numbers_from_csv(workdir, relation_name, street_name)
            only_in_reference = helpers.get_only_in_first(reference_house_numbers, osm_house_numbers)
            in_both = helpers.get_in_both(reference_house_numbers, osm_house_numbers)
            if only_in_reference:
                results.append((street_name, only_in_reference))
            if in_both:
                both_results.append((street_name, in_both))

        # Sort by length.
        results.sort(key=lambda result: len(result[1]), reverse=True)

        self.suspicious_streets = results
        self.done_streets = both_results

    def normalize(self, house_numbers: str, street_name: str) -> List[str]:
        """Strips down string input to bare minimum that can be interpreted as an
        actual number. Think about a/b, a-b, and so on."""
        ret = []
        for house_number in house_numbers.split('-'):
            try:
                number = int(re.sub(r"([0-9]+).*", r"\1", house_number))
            except ValueError:
                continue

            street_simple = street_name
            if self.simplify:
                # Old code path
                street_simple = helpers.simplify(street_name)

            if street_simple in self.normalizers.keys():
                # Have a custom filter.
                normalizer = self.normalizers[street_simple]
            else:
                # Default sanity checks.
                default = [helpers.Range(1, 999), helpers.Range(2, 998)]
                normalizer = helpers.Ranges(default)
            if number not in normalizer:
                continue

            ret.append(str(number))
        return ret

    def get_house_numbers_from_csv(self, workdir, relation_name, street_name):
        """Gets house numbers from the overpass query."""
        house_numbers = []  # type: List[str]
        with open(os.path.join(workdir, "street-housenumbers-%s.csv" % relation_name)) as sock:
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
                house_numbers += self.normalize(tokens[2], street_name)
        return helpers.sort_numerically(set(house_numbers))

    def get_house_numbers_from_lst(self, workdir, relation_name, street_name, ref_street):
        """Gets house numbers from reference."""
        house_numbers = []  # type: List[str]
        lst_street_name = ref_street
        prefix = lst_street_name + " "
        sock = open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relation_name))
        for line in sock.readlines():
            line = line.strip()
            if line.startswith(prefix):
                house_numbers += self.normalize(line.replace(prefix, ''), street_name)
        sock.close()
        return helpers.sort_numerically(set(house_numbers))


def main():
    """Commandline interface to this module."""
    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)
    workdir = config.get('wsgi', 'workdir').strip()
    datadir = os.path.join(os.path.dirname(__file__), "data")

    if len(sys.argv) > 1:
        relation_name = sys.argv[1]

    finder = Finder(datadir, workdir, relation_name)

    for result in finder.suspicious_streets:
        if result[1]:
            # House number, # of only_in_reference items.
            print("%s\t%s" % (result[0], len(result[1])))
            # only_in_reference items.
            print(result[1])


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
