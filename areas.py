#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The areas module contains the Relations class and associated functionality."""

from typing import List

import rust
import util


RelationConfig = rust.PyRelationConfig
Relation = rust.PyRelation
Relations = rust.PyRelations


def make_turbo_query_for_streets(relation: Relation, streets: List[str]) -> str:
    """Creates an overpass query that shows all streets from a missing housenumbers table."""
    return rust.py_make_turbo_query_for_streets(relation, streets)


def make_turbo_query_for_street_objs(relation: Relation, streets: List[util.Street]) -> str:
    """Creates an overpass query that shows all streets from a list."""
    return rust.py_make_turbo_query_for_street_objs(relation, streets)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
