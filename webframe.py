#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The webframe module provides the header, toolbar and footer code."""

from typing import Any
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
import configparser
import datetime
import traceback

import pytz
import yattag

from i18n import translate as _
import areas
import util
import version

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_footer(last_updated: str = "") -> yattag.Doc:
    """Produces the end of the page."""
    items: List[yattag.Doc] = []
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
        doc.text(", ")
        with doc.tag("a", href="/osm/missing-housenumbers/" + relation_name + "/view-result.chkl"):
            doc.text("chkl")
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
    items: List[yattag.Doc] = []

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


def handle_static(request_uri: str) -> Tuple[str, str]:
    """Handles serving static content."""
    tokens = request_uri.split("/")
    path = tokens[-1]

    if request_uri.endswith(".js"):
        content_type = "application/x-javascript"
    elif request_uri.endswith(".css"):
        content_type = "text/css"

    if path.endswith(".js") or path.endswith(".css"):
        return util.get_content(util.get_abspath("static"), path), content_type

    return "", ""


def send_response(start_response: 'StartResponse', content_type: str, status: str, output: str) -> Iterable[bytes]:
    """Turns an output string into a byte array and sends it."""
    output_bytes = output.encode('utf-8')
    response_headers = [('Content-type', content_type + '; charset=utf-8'),
                        ('Content-Length', str(len(output_bytes)))]
    start_response(status, response_headers)
    return [output_bytes]


def handle_exception(
        environ: Dict[str, Any],
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """Displays an unhandled exception on the page."""
    status = '500 Internal Server Error'
    path_info = environ.get("PATH_INFO")
    request_uri = path_info
    doc = yattag.Doc()
    util.write_html_header(doc)
    with doc.tag("pre"):
        doc.text(_("Internal error when serving {0}").format(request_uri) + "\n")
        doc.text(traceback.format_exc())
    return send_response(start_response, "text/html", status, doc.getvalue())


def get_config() -> configparser.ConfigParser:
    """Gets access to information which are specific to this installation."""
    config = configparser.ConfigParser()
    config_path = util.get_abspath("wsgi.ini")
    config.read(config_path)
    return config


def local_to_ui_tz(local_dt: datetime.datetime) -> datetime.datetime:
    """Converts from local date-time to UI date-time, based on config."""
    config = get_config()
    if config.has_option("wsgi", "timezone"):
        ui_tz = pytz.timezone(config.get("wsgi", "timezone"))
    else:
        ui_tz = pytz.timezone("Europe/Budapest")

    return local_dt.astimezone(ui_tz)


def format_timestamp(timestamp: float) -> str:
    """Formats timestamp as UI date-time."""
    local_dt = datetime.datetime.fromtimestamp(timestamp)
    ui_dt = local_to_ui_tz(local_dt)
    fmt = '%Y-%m-%d %H:%M'
    return ui_dt.strftime(fmt)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
