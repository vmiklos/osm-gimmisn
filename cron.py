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


def overpass_sleep(ctx: rust.PyContext) -> None:
    """Sleeps to respect overpass rate limit."""
    rust.py_overpass_sleep(ctx)


def update_osm_streets(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the OSM street list of all relations."""
    rust.py_update_osm_streets(ctx, relations, update)


def update_osm_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the OSM housenumber list of all relations."""
    rust.py_update_osm_housenumbers(ctx, relations, update)


def update_ref_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the reference housenumber list of all relations."""
    rust.py_update_ref_housenumbers(ctx, relations, update)


def update_ref_streets(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the reference street list of all relations."""
    rust.py_update_ref_streets(ctx, relations, update)


def update_missing_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the relation's house number coverage stats."""
    rust.py_update_missing_housenumbers(ctx, relations, update)


def update_missing_streets(relations: rust.PyRelations, update: bool) -> None:
    """Update the relation's street coverage stats."""
    rust.py_update_missing_streets(relations, update)


def update_additional_streets(relations: rust.PyRelations, update: bool) -> None:
    """Update the relation's "additional streets" stats."""
    rust.py_update_additional_streets(relations, update)


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
