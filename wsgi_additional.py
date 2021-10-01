#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi_additional module contains functionality for additional streets."""

from typing import Tuple

import yattag

import areas
import rust


def additional_streets_view_txt(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str,
    chkl: bool
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt."""
    return rust.py_additional_streets_view_txt(ctx, relations, request_uri, chkl)


def additional_streets_view_result(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str
) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result."""
    return rust.py_additional_streets_view_result(ctx, relations, request_uri)


def additional_housenumbers_view_result(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str
) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/budapest_11/view-result."""
    return rust.py_additional_housenumbers_view_result(ctx, relations, request_uri)


def additional_streets_view_turbo(relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.Doc()
    relation = relations.get_relation(relation_name)
    streets = relation.get_additional_streets(sorted_result=False)
    query = areas.make_turbo_query_for_street_objs(relation, streets)

    with doc.tag("pre", []):
        doc.text(query)
    return doc

# vim:set shiftwidth=4 softtabstop=4 expandtab:
