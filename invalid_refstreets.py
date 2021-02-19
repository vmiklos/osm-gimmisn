#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Prints invalid refstreets for all realtions."""

import areas
import config


def main() -> None:
    """Commandline interface."""
    workdir = config.Config.get_workdir()
    relations = areas.Relations(workdir)
    for relation in relations.get_relations():
        invalid_refstreets = areas.get_invalid_refstreets(relation)
        osm_invalids, ref_invalids = invalid_refstreets
        if not osm_invalids and not ref_invalids:
            continue

        print(relation.get_name() + ": " + str(invalid_refstreets))


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
