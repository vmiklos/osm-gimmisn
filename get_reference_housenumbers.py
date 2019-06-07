#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The get_reference_housenumbers module allows fetching referene house numbers for a relation."""

import configparser
import os
import pickle
import sys
# pylint: disable=unused-import
from typing import Dict
from typing import List
import helpers

VERBOSE = False
MEMORY_CACHE = {}  # type: Dict[str, Dict[str, Dict[str, List[str]]]]


def build_memory_cache(local):
    """Builds an in-memory cache from the reference on-disk TSV."""
    global MEMORY_CACHE

    disk_cache = local + ".pickle"
    if os.path.exists(disk_cache):
        MEMORY_CACHE = pickle.load(open(disk_cache, "rb"))
        return

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
            if refmegye not in MEMORY_CACHE.keys():
                MEMORY_CACHE[refmegye] = {}
            if reftelepules not in MEMORY_CACHE[refmegye].keys():
                MEMORY_CACHE[refmegye][reftelepules] = {}
            if street not in MEMORY_CACHE[refmegye][reftelepules].keys():
                MEMORY_CACHE[refmegye][reftelepules][street] = []
            MEMORY_CACHE[refmegye][reftelepules][street].append(num)
    pickle.dump(MEMORY_CACHE, open(disk_cache, "wb"))


def house_numbers_of_street(datadir, reference, relation_name, street):
    """Gets house numbers for a street locally."""
    if VERBOSE:
        print("searching '" + street + "'")
    refmegye, reftelepules, street_name, street_type = helpers.get_street_details(datadir, street, relation_name)
    street = street_name + " " + street_type
    if street in reference[refmegye][reftelepules].keys():
        house_numbers = reference[refmegye][reftelepules][street]
        return [street + " " + i for i in house_numbers]

    return []


def get_reference_housenumbers(config, relation_name):
    """Gets known house numbers (not their coordinates) from a reference site, based on street names
    from OSM."""
    reference = config.get('wsgi', 'reference_local').strip()
    if not MEMORY_CACHE:
        if VERBOSE:
            print("building in-memory cache")
        build_memory_cache(reference)

    datadir = os.path.join(os.path.dirname(__file__), "data")
    workdir = config.get('wsgi', 'workdir').strip()
    streets = helpers.get_streets(workdir, relation_name)

    lst = []  # type: List[str]
    for street in streets:
        lst += house_numbers_of_street(datadir, MEMORY_CACHE, relation_name, street)

    lst = sorted(set(lst))
    sock = open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relation_name), "w")
    for line in lst:
        sock.write(line + "\n")
    sock.close()


def main():
    """Commandline interface to this module."""
    global VERBOSE

    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)

    if len(sys.argv) > 1:
        relation_name = sys.argv[1]

    VERBOSE = True
    get_reference_housenumbers(config, relation_name)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
