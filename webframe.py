#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The webframe module provides the header, toolbar and footer code."""

from typing import List
from typing import Optional

import yattag  # type: ignore

from i18n import translate as _
import areas
import util
import version


def get_footer(last_updated: str = "") -> yattag.Doc:
    """Produces the end of the page."""
    items = []  # type: List[yattag.Doc]
    doc = yattag.Doc()
    doc.text(_("Version: "))
    doc.asis(util.git_link(version.VERSION, "https://github.com/vmiklos/osm-gimmisn/commit/").getvalue())
    items.append(doc)
    items.append(util.html_escape(_("OSM data © OpenStreetMap contributors.")))
    if last_updated:
        items.append(util.html_escape(_("Last update: ") + last_updated))
    doc = yattag.Doc()
    doc.stag("hr")
    with doc.tag("div"):
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.asis(item.getvalue())
    return doc


def fill_header_function(function: str, relation_name: str, items: List[yattag.Doc]) -> None:
    """Fills items with function-specific links in the header. Returns a title."""
    if function == "missing-housenumbers":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/missing-housenumbers/" + relation_name + "/update-result"):
            doc.text(_("Update from reference"))
        doc.text(" " + _("(may take seconds)"))
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", href="https://overpass-turbo.eu/"):
            doc.text(_("Overpass turbo"))
        items.append(doc)
    elif function == "missing-streets":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/missing-streets/" + relation_name + "/update-result"):
            doc.text(_("Update from reference"))
        items.append(doc)
    elif function == "street-housenumbers":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/street-housenumbers/" + relation_name + "/update-result"):
            doc.text(_("Call Overpass to update"))
        doc.text(" " + _("(may take seconds)"))
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/street-housenumbers/" + relation_name + "/view-query"):
            doc.text(_("View query"))
        items.append(doc)
    elif function == "streets":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/streets/" + relation_name + "/update-result"):
            doc.text(_("Call Overpass to update"))
        doc.text(" " + _("(may take seconds)"))
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/streets/" + relation_name + "/view-query"):
            doc.text(_("View query"))
        items.append(doc)


def fill_missing_header_items(streets: str, relation_name: str, items: List[yattag.Doc]) -> None:
    """Generates the 'missing house numbers/streets' part of the header."""
    if streets != "only":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/missing-housenumbers/" + relation_name + "/view-result"):
            doc.text(_("Missing house numbers"))
        doc.text(" (")
        with doc.tag("a", href="/osm/missing-housenumbers/" + relation_name + "/view-result.txt"):
            doc.text("txt")
        doc.text(")")
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/street-housenumbers/" + relation_name + "/view-result"):
            doc.text(_("Existing house numbers"))
        items.append(doc)
    if streets != "no":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/missing-streets/" + relation_name + "/view-result"):
            doc.text(_("Missing streets"))
        doc.text(" (")
        with doc.tag("a", href="/osm/missing-streets/" + relation_name + "/view-result.txt"):
            doc.text("txt")
        doc.text(")")
        items.append(doc)


def get_toolbar(
        relations: Optional[areas.Relations] = None,
        function: str = "",
        relation_name: str = "",
        relation_osmid: int = 0
) -> yattag.Doc:
    """Produces the start of the page. Note that the content depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    items = []  # type: List[yattag.Doc]

    if relations and relation_name:
        relation = relations.get_relation(relation_name)
        streets = relation.get_config().should_check_missing_streets()

    doc = yattag.Doc()
    with doc.tag("a", href="/osm"):
        doc.text(_("Area list"))
    items.append(doc)
    if relation_name:
        fill_missing_header_items(streets, relation_name, items)
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/streets/" + relation_name + "/view-result"):
            doc.text(_("Existing streets"))
        items.append(doc)

    fill_header_function(function, relation_name, items)

    if relation_osmid:
        doc = yattag.Doc()
        with doc.tag("a", href="https://www.openstreetmap.org/relation/" + str(relation_osmid)):
            doc.text(_("Area boundary"))
        items.append(doc)
    doc = yattag.Doc()
    with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
        doc.text(_("Documentation"))
    items.append(doc)

    doc = yattag.Doc()
    with doc.tag("div", id="toolbar"):
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.asis(item.getvalue())
    doc.stag("hr")
    return doc


# vim:set shiftwidth=4 softtabstop=4 expandtab:
