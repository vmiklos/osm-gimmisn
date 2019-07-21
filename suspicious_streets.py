#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Compares reference house numbers with OSM ones and shows the diff."""

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
    suspicious_streets, _ = helpers.get_suspicious_streets(datadir, relations, relation_name)

    for result in suspicious_streets:
        if result[1]:
            # House number, # of only_in_reference items.
            print("%s\t%s" % (result[0], len(result[1])))
            # only_in_reference items.
            print(result[1])


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
