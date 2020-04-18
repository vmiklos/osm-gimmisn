#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The get_reference_streets module allows fetching reference streets for a relation."""

import sys
import areas

import config


def main() -> None:
    """Commandline interface to this module."""

    relation_name = sys.argv[1]

    reference = config.Config.get_reference_street_path()
    workdir = config.Config.get_workdir()
    relations = areas.Relations(workdir)
    relation = relations.get_relation(relation_name)
    relation.write_ref_streets(reference)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
