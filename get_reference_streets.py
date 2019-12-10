#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The get_reference_streets module allows fetching reference streets for a relation."""

import configparser
import sys
import areas
import util


def main() -> None:
    """Commandline interface to this module."""

    config = configparser.ConfigParser()
    config_path = util.get_abspath("wsgi.ini")
    config.read(config_path)

    relation_name = sys.argv[1]

    reference = util.get_abspath(config.get('wsgi', 'reference_street').strip())
    datadir = util.get_abspath("data")
    workdir = util.get_abspath(config.get('wsgi', 'workdir').strip())
    relations = areas.Relations(datadir, workdir)
    relation = relations.get_relation(relation_name)
    relation.write_ref_streets(reference)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
