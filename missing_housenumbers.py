#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Compares reference house numbers with OSM ones and shows the diff."""

from typing import List
from typing import TextIO
import sys

import areas
import rust


def main(argv: List[str], stdout: TextIO, ctx: rust.PyContext) -> None:
    """Commandline interface."""

    relation_name = argv[1]

    relations = areas.Relations(ctx)
    relation = relations.get_relation(relation_name)
    ongoing_streets, _ = relation.get_missing_housenumbers()

    for result in ongoing_streets:
        # House number, # of only_in_reference items.
        range_list = rust.py_get_housenumber_ranges(result[1])
        range_strings = [i.get_number() for i in range_list]
        range_strings = sorted(range_strings, key=rust.py_split_house_number)
        stdout.write("%s\t%s\n" % (result[0].get_osm_name(), len(range_strings)))
        # only_in_reference items.
        stdout.write(str(range_strings) + "\n")


if __name__ == '__main__':
    main(sys.argv, sys.stdout, rust.PyContext(""))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
