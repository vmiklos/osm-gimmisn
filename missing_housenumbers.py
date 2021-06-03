#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Compares reference house numbers with OSM ones and shows the diff."""

import sys

import areas
import config
import util


def main() -> None:
    """Commandline interface."""
    conf = config.make_config()
    workdir = conf.get_workdir()

    relation_name = sys.argv[1]

    relations = areas.Relations(workdir)
    relation = relations.get_relation(relation_name)
    ongoing_streets, _ = relation.get_missing_housenumbers()

    for result in ongoing_streets:
        # House number, # of only_in_reference items.
        range_list = util.get_housenumber_ranges(result[1])
        range_strings = [i.get_number() for i in range_list]
        range_strings = sorted(range_strings, key=util.split_house_number)
        print("%s\t%s" % (result[0].get_osm_name(), len(range_strings)))
        # only_in_reference items.
        print(range_strings)


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
