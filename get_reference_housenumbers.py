#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The get_reference_housenumbers module allows fetching reference house numbers for a relation."""

import sys

import areas
import config


def main() -> None:
    """Commandline interface to this module."""

    relation_name = sys.argv[1]

    references = config.Config.get_reference_housenumber_paths()
    workdir = config.Config.get_workdir()
    relations = areas.Relations(workdir)
    relation = relations.get_relation(relation_name)
    relation.write_ref_housenumbers(references)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
