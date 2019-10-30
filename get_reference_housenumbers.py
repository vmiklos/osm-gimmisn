#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The get_reference_housenumbers module allows fetching reference house numbers for a relation."""

import configparser
import os
import sys
import helpers


def main() -> None:
    """Commandline interface to this module."""

    config = configparser.ConfigParser()
    config_path = os.path.join(os.getcwd(), "wsgi.ini")
    config.read(config_path)

    relation_name = sys.argv[1]

    reference = config.get('wsgi', 'reference_housenumbers').strip().split(' ')
    datadir = os.path.join(os.getcwd(), "data")
    workdir = config.get('wsgi', 'workdir').strip()
    relations = helpers.Relations(datadir, workdir)
    relation = relations.get_relation(relation_name)
    relation.write_ref_housenumbers(reference)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
