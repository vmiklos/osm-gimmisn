#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The areas module contains the Relations class and associated functionality."""

from typing import Dict
from typing import List

import rust
import util


RelationConfig = rust.PyRelationConfig
Relation = rust.PyRelation
Relations = rust.PyRelations


def normalize(relation: rust.PyRelation, house_numbers: str, street_name: str,
              street_is_even_odd: bool,
              normalizers: Dict[str, rust.PyRanges]) -> List[rust.PyHouseNumber]:
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    return rust.py_normalize(relation, house_numbers, street_name, street_is_even_odd, normalizers)


def make_turbo_query_for_streets(relation: Relation, streets: List[str]) -> str:
    """Creates an overpass query that shows all streets from a missing housenumbers table."""
    header = """[out:json][timeout:425];
rel(@RELATION@)->.searchRelation;
area(@AREA@)->.searchArea;
(rel(@RELATION@);
"""
    query = util.process_template(header, relation.get_config().get_osmrelation())
    for street in streets:
        query += 'way["name"="' + street + '"](r.searchRelation);\n'
        query += 'way["name"="' + street + '"](area.searchArea);\n'
    query += """);
out body;
>;
out skel qt;
{{style:
relation{width:3}
way{color:blue; width:4;}
}}"""
    return query


def make_turbo_query_for_street_objs(relation: Relation, streets: List[util.Street]) -> str:
    """Creates an overpass query that shows all streets from a list."""
    header = """[out:json][timeout:425];
rel(@RELATION@)->.searchRelation;
area(@AREA@)->.searchArea;
("""
    query = util.process_template(header, relation.get_config().get_osmrelation())
    ids = []
    for street in streets:
        ids.append((street.get_osm_type(), str(street.get_osm_id())))
    for osm_type, osm_id in sorted(set(ids)):
        query += osm_type + "(" + osm_id + ");\n"
    query += """);
out body;
>;
out skel qt;"""
    return query

# vim:set shiftwidth=4 softtabstop=4 expandtab:
