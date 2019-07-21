#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Compares reference streets with OSM ones and shows the diff."""

import os
import sys
import configparser
import helpers


def main() -> None:
    """Commandline interface."""
    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)
    workdir = config.get('wsgi', 'workdir').strip()
    datadir = os.path.join(os.path.dirname(__file__), "data")

    if len(sys.argv) > 1:
        relation_name = sys.argv[1]

    relations = helpers.Relations(datadir, workdir)
    only_in_reference, _ = helpers.get_suspicious_relations(relations, relation_name)

    for street in only_in_reference:
        print(street)


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
