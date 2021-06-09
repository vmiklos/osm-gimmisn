#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Compares OSM streets with reference ones and shows the diff."""

import sys

import areas
import config


def main(conf: config.Config) -> None:
    """Commandline interface."""
    relation_name = sys.argv[1]

    relations = areas.Relations(conf)
    relation = relations.get_relation(relation_name)
    only_in_osm = relation.get_additional_streets()

    for street in only_in_osm:
        print(street)


if __name__ == '__main__':
    main(config.Config(""))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
