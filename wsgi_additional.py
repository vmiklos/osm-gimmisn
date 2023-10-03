#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi_additional module contains functionality for additional streets."""

from typing import Tuple

import yattag

from i18n import translate as tr
import areas
import cache
import context
import util
import webframe


def additional_streets_view_txt(
    ctx: context.Context,
    relations: areas.Relations,
    request_uri: str,
    chkl: bool
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not ctx.get_file_system().path_exists(relation.get_files().get_osm_streets_path()):
        output += tr("No existing streets")
    elif not ctx.get_file_system().path_exists(relation.get_files().get_ref_streets_path()):
        output += tr("No reference streets")
    else:
        streets = relation.get_additional_streets()
        lexical_sort_key = util.get_lexical_sort_key()
        streets.sort(key=lambda street: lexical_sort_key(street.get_osm_name()))
        for street in streets:
            if chkl:
                output += "[ ] {}\n".format(street.get_osm_name())
            else:
                output += "{}\n".format(street.get_osm_name())
    return output, relation_name


def additional_streets_view_result(
    ctx: context.Context,
    relations: areas.Relations,
    request_uri: str
) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    doc = yattag.doc.Doc()
    prefix = ctx.get_ini().get_uri_prefix()
    prefix2 = "https://osm-gimmisn.vmiklos.hu/osm"

    if not ctx.get_file_system().path_exists(relation.get_files().get_osm_streets_path()):
        doc.asis(webframe.handle_no_osm_streets(prefix, relation_name).getvalue())
    elif not ctx.get_file_system().path_exists(relation.get_files().get_ref_streets_path()):
        doc.asis(webframe.handle_no_ref_streets(prefix, relation_name).getvalue())
    else:
        # Get "only in OSM" streets.
        streets = relation.write_additional_streets()
        count = len(streets)
        lexical_sort_key = util.get_lexical_sort_key()
        streets.sort(key=lambda street: lexical_sort_key(street.get_osm_name()))
        table = [[util.html_escape(tr("Identifier")),
                  util.html_escape(tr("Type")),
                  util.html_escape(tr("Source")),
                  util.html_escape(tr("Street name"))]]
        for street in streets:
            cell = yattag.doc.Doc()
            href = "https://www.openstreetmap.org/{}/{}".format(street.get_osm_type(), street.get_osm_id())
            with cell.tag("a", href=href, target="_blank"):
                cell.text(str(street.get_osm_id()))
            cells = [
                cell,
                util.html_escape(street.get_osm_type()),
                util.html_escape(street.get_source()),
                util.html_escape(street.get_osm_name()),
            ]
            table.append(cells)

        with doc.tag("p"):
            doc.text(tr("OpenStreetMap additionally has the below {0} streets.").format(str(count)))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/additional-streets/" + relation_name + "/view-result.txt"):
                doc.text(tr("Plain text format"))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/additional-streets/" + relation_name + "/view-result.chkl"):
                doc.text(tr("Checklist format"))
            doc.stag("br")
            with doc.tag("a", href=prefix2 + "/additional-streets/" + relation_name + "/view-result.gpx", target="_blank"):
                doc.text(tr("GPX format (vmiklos.hu)"))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/additional-streets/{}/view-turbo".format(relation_name)):
                doc.text(tr("Overpass turbo query for the below streets"))

        doc.asis(util.html_table_from_list(table).getvalue())
        doc.asis(util.invalid_refstreets_to_html(relation.get_invalid_refstreets()).getvalue())
    return doc


def additional_housenumbers_view_result(
    ctx: context.Context,
    relations: areas.Relations,
    request_uri: str
) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/budapest_11/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    doc = yattag.doc.Doc()
    relation = relations.get_relation(relation_name)
    prefix = ctx.get_ini().get_uri_prefix()
    if not ctx.get_file_system().path_exists(relation.get_files().get_osm_streets_path()):
        doc.asis(webframe.handle_no_osm_streets(prefix, relation_name).getvalue())
    elif not ctx.get_file_system().path_exists(relation.get_files().get_osm_housenumbers_path()):
        doc.asis(webframe.handle_no_osm_housenumbers(prefix, relation_name).getvalue())
    elif not ctx.get_file_system().path_exists(relation.get_files().get_ref_housenumbers_path()):
        doc.asis(webframe.handle_no_ref_housenumbers(prefix, relation_name).getvalue())
    else:
        doc = cache.get_additional_housenumbers_html(ctx, relation)
    return doc


def additional_streets_view_turbo(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.doc.Doc()
    relation = relations.get_relation(relation_name)
    streets = relation.get_additional_streets(sorted_result=False)
    query = areas.make_turbo_query_for_street_objs(relation, streets)

    with doc.tag("pre"):
        doc.text(query)
    return doc

# vim:set shiftwidth=4 softtabstop=4 expandtab:
