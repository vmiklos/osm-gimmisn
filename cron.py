#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

from typing import List
from typing import BinaryIO

import rust


def update_osm_streets(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the OSM street list of all relations."""
    rust.py_update_osm_streets(ctx, relations, update)


def update_osm_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the OSM housenumber list of all relations."""
    rust.py_update_osm_housenumbers(ctx, relations, update)


def update_stats_count(ctx: rust.PyContext, today: str) -> None:
    """Counts the # of all house numbers as of today."""
    rust.py_update_stats_count(ctx, today)


def update_stats_topusers(ctx: rust.PyContext, today: str) -> None:
    """Counts the top housenumber editors as of today."""
    rust.py_update_stats_topusers(ctx, today)


def update_stats(ctx: rust.PyContext, overpass: bool) -> None:
    """Performs the update of country-level stats."""
    rust.py_update_stats(ctx, overpass)


def our_main(ctx: rust.PyContext, relations: rust.PyRelations, mode: str, update: bool, overpass: bool) -> None:
    """Performs the actual nightly task."""
    return rust.py_our_main(ctx, relations, mode, update, overpass)


def main(argv: List[str], stdout: BinaryIO, ctx: rust.PyContext) -> None:
    """Commandline interface to this module."""
    rust.py_cron_main(argv, stdout, ctx)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
