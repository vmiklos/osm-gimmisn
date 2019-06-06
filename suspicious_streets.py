#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The suspicious_streets module tries to find streets which do have at least one house number, but
suspicious as lots of house numbers are probably missing."""

import os
import sys
import configparser
# pylint: disable=unused-import
from typing import Dict, List
import helpers


def get_suspicious_streets(datadir, workdir, relation_name):
    """Compares reference house numbers with OSM ones and shows the diff."""
    normalizers = {}  # type: Dict[str, helpers.Ranges]
    # OSM name -> ref name map
    ref_streets = {}  # type: Dict[str, str]

    normalizers, ref_streets = helpers.load_normalizers(datadir, relation_name)
    street_names = helpers.get_streets(workdir, relation_name)

    results = []
    both_results = []

    for street_name in street_names:

        ref_street = street_name
        # See if we need to map the OSM name to ref name.
        if street_name in ref_streets.keys():
            ref_street = ref_streets[street_name]

        reference_house_numbers = helpers.get_house_numbers_from_lst(workdir, relation_name, street_name,
                                                                     ref_street, normalizers)
        osm_house_numbers = helpers.get_house_numbers_from_csv(workdir, relation_name, street_name,
                                                               normalizers)
        only_in_reference = helpers.get_only_in_first(reference_house_numbers, osm_house_numbers)
        in_both = helpers.get_in_both(reference_house_numbers, osm_house_numbers)
        if only_in_reference:
            results.append((street_name, only_in_reference))
        if in_both:
            both_results.append((street_name, in_both))

    # Sort by length.
    results.sort(key=lambda result: len(result[1]), reverse=True)

    suspicious_streets = results
    done_streets = both_results

    return suspicious_streets, done_streets


def main():
    """Commandline interface to this module."""
    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)
    workdir = config.get('wsgi', 'workdir').strip()
    datadir = os.path.join(os.path.dirname(__file__), "data")

    if len(sys.argv) > 1:
        relation_name = sys.argv[1]

    suspicious_streets, _ = get_suspicious_streets(datadir, workdir, relation_name)

    for result in suspicious_streets:
        if result[1]:
            # House number, # of only_in_reference items.
            print("%s\t%s" % (result[0], len(result[1])))
            # only_in_reference items.
            print(result[1])


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
