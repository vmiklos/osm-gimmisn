#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi_additional module contains functionality for additional streets."""

import locale
import os
from typing import Tuple

import yattag

from i18n import translate as _
import areas
import config
import util
import webframe


def additional_streets_view_txt(relations: areas.Relations, request_uri: str, chkl: bool) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets")
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        output += _("No reference streets")
    else:
        streets = relation.get_additional_streets()
        streets.sort(key=lambda street: locale.strxfrm(street.get_osm_name()))
        for street in streets:
            if chkl:
                output += "[ ] {}\n".format(street.get_osm_name())
            else:
                output += "{}\n".format(street.get_osm_name())
    return output, relation_name


def additional_streets_view_result(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    doc = yattag.doc.Doc()
    prefix = config.Config.get_uri_prefix()
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        doc.asis(webframe.handle_no_osm_streets(prefix, relation_name).getvalue())
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        doc.asis(webframe.handle_no_ref_streets(prefix, relation_name).getvalue())
    else:
        # Get "only in OSM" streets.
        streets = relation.write_additional_streets()
        count = len(streets)
        streets.sort(key=lambda street: locale.strxfrm(street.get_osm_name()))
        table = [[util.html_escape(_("Identifier")),
                  util.html_escape(_("Type")),
                  util.html_escape(_("Source")),
                  util.html_escape(_("Street name"))]]
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
            doc.text(_("OpenStreetMap additionally has the below {0} streets.").format(str(count)))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/additional-streets/" + relation_name + "/view-result.txt"):
                doc.text(_("Plain text format"))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/additional-streets/" + relation_name + "/view-result.chkl"):
                doc.text(_("Checklist format"))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/additional-streets/{}/view-turbo".format(relation_name)):
                doc.text(_("Overpass turbo query for the below streets"))

        doc.asis(util.html_table_from_list(table).getvalue())
    return doc


def additional_streets_view_turbo(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.doc.Doc()
    relation = relations.get_relation(relation_name)
    streets = relation.get_additional_streets()
    query = areas.make_turbo_query_for_street_objs(relation, streets)

    with doc.tag("pre"):
        doc.text(query)
    return doc

# vim:set shiftwidth=4 softtabstop=4 expandtab:
