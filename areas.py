#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The areas module contains the Relations class and associated functionality."""

import os
from typing import Any
from typing import Dict
from typing import List
from typing import Optional
from typing import cast
import json

import context
import rust
import util


RelationConfig = rust.PyRelationConfig
Relation = rust.PyRelation


class Relations:
    """A relations object is a container of named relation objects."""
    def __init__(self, ctx: context.Context) -> None:
        with ctx.get_file_system().open_read(os.path.join(ctx.get_abspath("data"), "yamls.cache")) as stream:
            self.__yaml_cache: Dict[str, Any] = json.load(stream)
        self.__dict = self.__yaml_cache["relations.yaml"]
        self.__refcounty_names = self.__yaml_cache["refcounty-names.yaml"]
        self.__refsettlement_names = self.__yaml_cache["refsettlement-names.yaml"]
        self.rust = rust.PyRelations(ctx)

    def get_workdir(self) -> str:
        """Gets the workdir directory path."""
        return self.rust.get_workdir()

    def get_relation(self, name: str) -> Relation:
        """Gets the relation that has the specified name."""
        return self.rust.get_relation(name)

    def set_relation(self, name: str, relation: rust.PyRelation) -> None:
        """Sets a relation for testing."""
        self.rust.set_relation(name, relation)

    def get_names(self) -> List[str]:
        """Gets a sorted list of relation names."""
        return self.rust.get_names()

    def get_active_names(self) -> List[str]:
        """Gets a sorted list of active relation names."""
        return self.rust.get_active_names()

    def get_relations(self) -> List[Relation]:
        """Gets a list of relations."""
        return self.rust.get_relations()

    def activate_all(self, flag: bool) -> None:
        """Sets if inactive=true is ignored or not."""
        self.rust.activate_all(flag)

    def limit_to_refcounty(self, refcounty: Optional[str]) -> None:
        """If refcounty is not None, forget about all relations outside that refcounty."""
        if not refcounty:
            return
        for relation_name in list(self.__dict.keys()):
            relation = self.get_relation(relation_name)
            if relation.get_config().get_refcounty() == refcounty:
                continue
            del self.__dict[relation_name]
            self.rust.delete_relation(relation_name)

    def limit_to_refsettlement(self, refsettlement: Optional[str]) -> None:
        """If refsettlement is not None, forget about all relations outside that refsettlement."""
        if not refsettlement:
            return
        for relation_name in list(self.__dict.keys()):
            relation = self.get_relation(relation_name)
            if relation.get_config().get_refsettlement() == refsettlement:
                continue
            del self.__dict[relation_name]
            self.rust.delete_relation(relation_name)

    def refcounty_get_name(self, refcounty: str) -> str:
        """Produces a UI name for a refcounty."""
        if refcounty in self.__refcounty_names:
            return cast(str, self.__refcounty_names[refcounty])

        return ""

    def refcounty_get_refsettlement_ids(self, refcounty_name: str) -> List[str]:
        """Produces refsettlement IDs of a refcounty."""
        if refcounty_name not in self.__refsettlement_names:
            return []

        refcounty = self.__refsettlement_names[refcounty_name]
        return list(refcounty.keys())

    def refsettlement_get_name(self, refcounty_name: str, refsettlement: str) -> str:
        """Produces a UI name for a refsettlement in refcounty."""
        if refcounty_name not in self.__refsettlement_names:
            return ""

        refcounty = self.__refsettlement_names[refcounty_name]
        if refsettlement not in refcounty:
            return ""

        return cast(str, refcounty[refsettlement])

    def get_aliases(self) -> Dict[str, str]:
        """Provide an alias -> real name map of relations."""
        ret: Dict[str, str] = {}
        for relation in self.get_relations():
            aliases = relation.get_config().get_alias()
            if aliases:
                name = relation.get_name()
                for alias in aliases:
                    ret[alias] = name
        return ret


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
