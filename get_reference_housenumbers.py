#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The get_reference_housenumbers module allows fetching referene house numbers for a relation."""

import configparser
import os
import sys
# pylint: disable=unused-import
from typing import List
import helpers


def house_numbers_of_street(datadir, reference, relation_name, street):
    """Gets house numbers for a street locally."""
    refmegye, reftelepules_list, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
    street = street_name + " " + street_type
    ret = []  # type: List[str]
    for reftelepules in reftelepules_list:
        if street in reference[refmegye][reftelepules].keys():
            house_numbers = reference[refmegye][reftelepules][street]
            ret += [street + " " + i for i in house_numbers]

    return ret


def get_reference_housenumbers(config, relation_name):
    """Gets known house numbers (not their coordinates) from a reference site, based on street names
    from OSM."""
    reference = config.get('wsgi', 'reference_local').strip()
    memory_cache = helpers.build_reference_cache(reference)

    datadir = os.path.join(os.path.dirname(__file__), "data")
    workdir = config.get('wsgi', 'workdir').strip()
    streets = helpers.get_streets(workdir, relation_name)

    lst = []  # type: List[str]
    for street in streets:
        lst += house_numbers_of_street(datadir, memory_cache, relation_name, street)

    lst = sorted(set(lst))
    sock = open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relation_name), "w")
    for line in lst:
        sock.write(line + "\n")
    sock.close()


def main():
    """Commandline interface to this module."""

    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)

    if len(sys.argv) > 1:
        relation_name = sys.argv[1]

    get_reference_housenumbers(config, relation_name)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
