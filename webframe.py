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
from typing import cast
import datetime
import json
import os
import time
import urllib
import xmlrpc.client

import yattag

from rust import py_translate as tr
import areas
import context
import rust
import util

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_footer(last_updated: str = "") -> yattag.Doc:
    """Produces the end of the page."""
    items: List[yattag.Doc] = []
    doc = yattag.Doc()
    doc.text(tr("Version: "))
    doc.append_value(util.git_link(rust.py_get_version(), "https://github.com/vmiklos/osm-gimmisn/commit/").get_value())
    items.append(doc)
    items.append(yattag.Doc.from_text(tr("OSM data © OpenStreetMap contributors.")))
    if last_updated:
        items.append(yattag.Doc.from_text(tr("Last update: ") + last_updated))
    doc = yattag.Doc()
    doc.stag("hr", [])
    with doc.tag("div", []):
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.append_value(item.get_value())
    return doc


def fill_header_function(ctx: context.Context, function: str, relation_name: str, items: List[yattag.Doc]) -> None:
    """Fills items with function-specific links in the header. Returns a title."""
    prefix = ctx.get_ini().get_uri_prefix()
    if function == "missing-housenumbers":
        # The OSM data source changes much more frequently than the ref one, so add a dedicated link
        # to update OSM house numbers first.
        doc = yattag.Doc()
        with doc.tag("span", [("id", "trigger-street-housenumbers-update")]):
            with doc.tag("a", [("href", prefix + "/street-housenumbers/" + relation_name + "/update-result")]):
                doc.text(tr("Update from OSM"))
        items.append(doc)

        doc = yattag.Doc()
        with doc.tag("span", [("id", "trigger-missing-housenumbers-update")]):
            with doc.tag("a", [("href", prefix + "/missing-housenumbers/" + relation_name + "/update-result")]):
                doc.text(tr("Update from reference"))
        items.append(doc)
    elif function in ("missing-streets", "additional-streets"):
        # The OSM data source changes much more frequently than the ref one, so add a dedicated link
        # to update OSM streets first.
        doc = yattag.Doc()
        with doc.tag("span", [("id", "trigger-streets-update")]):
            with doc.tag("a", [("href", prefix + "/streets/" + relation_name + "/update-result")]):
                doc.text(tr("Update from OSM"))
        items.append(doc)

        doc = yattag.Doc()
        with doc.tag("span", [("id", "trigger-missing-streets-update")]):
            with doc.tag("a", [("href", prefix + "/missing-streets/" + relation_name + "/update-result")]):
                doc.text(tr("Update from reference"))
        items.append(doc)
    elif function == "street-housenumbers":
        doc = yattag.Doc()
        with doc.tag("span", [("id", "trigger-street-housenumbers-update")]):
            with doc.tag("a", [("href", prefix + "/street-housenumbers/" + relation_name + "/update-result")]):
                doc.text(tr("Call Overpass to update"))
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", [("href", prefix + "/street-housenumbers/" + relation_name + "/view-query")]):
            doc.text(tr("View query"))
        items.append(doc)
    elif function == "streets":
        doc = yattag.Doc()
        with doc.tag("span", [("id", "trigger-streets-update")]):
            with doc.tag("a", [("href", prefix + "/streets/" + relation_name + "/update-result")]):
                doc.text(tr("Call Overpass to update"))
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", [("href", prefix + "/streets/" + relation_name + "/view-query")]):
            doc.text(tr("View query"))
        items.append(doc)


def fill_missing_header_items(
    ctx: context.Context,
    streets: str,
    additional_housenumbers: bool,
    relation_name: str,
    items: List[yattag.Doc]
) -> None:
    """Generates the 'missing house numbers/streets' part of the header."""
    prefix = ctx.get_ini().get_uri_prefix()
    if streets != "only":
        doc = yattag.Doc()
        with doc.tag("a", [("href", prefix + "/missing-housenumbers/" + relation_name + "/view-result")]):
            doc.text(tr("Missing house numbers"))
        items.append(doc)

        if additional_housenumbers:
            doc = yattag.Doc()
            with doc.tag("a", [("href", prefix + "/additional-housenumbers/" + relation_name + "/view-result")]):
                doc.text(tr("Additional house numbers"))
            items.append(doc)
    if streets != "no":
        doc = yattag.Doc()
        with doc.tag("a", [("href", prefix + "/missing-streets/" + relation_name + "/view-result")]):
            doc.text(tr("Missing streets"))
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", [("href", prefix + "/additional-streets/" + relation_name + "/view-result")]):
            doc.text(tr("Additional streets"))
        items.append(doc)


def fill_existing_header_items(
    ctx: context.Context,
    streets: str,
    relation_name: str,
    items: List[yattag.Doc]
) -> None:
    """Generates the 'existing house numbers/streets' part of the header."""
    prefix = ctx.get_ini().get_uri_prefix()
    if streets != "only":
        doc = yattag.Doc()
        with doc.tag("a", [("href", prefix + "/street-housenumbers/" + relation_name + "/view-result")]):
            doc.text(tr("Existing house numbers"))
        items.append(doc)

    doc = yattag.Doc()
    with doc.tag("a", [("href", prefix + "/streets/" + relation_name + "/view-result")]):
        doc.text(tr("Existing streets"))
    items.append(doc)


def get_toolbar(
        ctx: context.Context,
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
        additional_housenumbers = relation.get_config().should_check_additional_housenumbers()

    doc = yattag.Doc()
    with doc.tag("a", [("href", ctx.get_ini().get_uri_prefix() + "/")]):
        doc.text(tr("Area list"))
    items.append(doc)

    if relation_name:
        fill_missing_header_items(ctx, streets, additional_housenumbers, relation_name, items)

    fill_header_function(ctx, function, relation_name, items)

    if relation_name:
        fill_existing_header_items(ctx, streets, relation_name, items)

    doc = yattag.Doc()

    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-toolbar-overpass-wait", tr("Waiting for Overpass...")),
            ("str-toolbar-overpass-error", tr("Error from Overpass: ")),
            ("str-toolbar-reference-wait", tr("Creating from reference...")),
            ("str-toolbar-reference-error", tr("Error from reference: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass

    with doc.tag("a", [("href", "https://overpass-turbo.eu/")]):
        doc.text(tr("Overpass turbo"))
    items.append(doc)

    if relation_osmid:
        doc = yattag.Doc()
        with doc.tag("a", [("href", "https://www.openstreetmap.org/relation/" + str(relation_osmid))]):
            doc.text(tr("Area boundary"))
        items.append(doc)
    else:
        # These are on the main page only.
        doc = yattag.Doc()
        with doc.tag("a", [("href", ctx.get_ini().get_uri_prefix() + "/housenumber-stats/hungary/")]):
            doc.text(tr("Statistics"))
        items.append(doc)

        doc = yattag.Doc()
        with doc.tag("a", [("href", "https://github.com/vmiklos/osm-gimmisn/tree/master/doc")]):
            doc.text(tr("Documentation"))
        items.append(doc)

    doc = yattag.Doc()
    with doc.tag("div", [("id", "toolbar")]):
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.append_value(item.get_value())
    doc.stag("hr", [])
    return doc


def handle_static(ctx: context.Context, request_uri: str) -> Tuple[bytes, str, List[Tuple[str, str]]]:
    """Handles serving static content."""
    tokens = request_uri.split("/")
    path = tokens[-1]
    extra_headers: List[Tuple[str, str]] = []

    if request_uri.endswith(".js"):
        content_type = "application/x-javascript"
        content, extra_headers = util.get_content_with_meta(ctx.get_abspath("builddir/" + path))
        return content, content_type, extra_headers
    if request_uri.endswith(".css"):
        content_type = "text/css"
        content, extra_headers = util.get_content_with_meta(ctx.get_ini().get_workdir() + "/" + path)
        return content, content_type, extra_headers
    if request_uri.endswith(".json"):
        content_type = "application/json"
        content, extra_headers = util.get_content_with_meta(os.path.join(ctx.get_ini().get_workdir(), "stats/" + path))
        return content, content_type, extra_headers
    if request_uri.endswith(".ico"):
        content_type = "image/x-icon"
        content, extra_headers = util.get_content_with_meta(ctx.get_abspath(path))
        return content, content_type, extra_headers
    if request_uri.endswith(".svg"):
        content_type = "image/svg+xml"
        content, extra_headers = util.get_content_with_meta(ctx.get_abspath(path))
        return content, content_type, extra_headers

    return bytes(), "", extra_headers


class Response:
    """A HTTP response, to be sent by send_response()."""
    def __init__(self, content_type: str, status: str, output_bytes: bytes, headers: List[Tuple[str, str]]) -> None:
        self.__content_type: str = content_type
        self.__status: str = status
        self.__output_bytes = output_bytes
        self.__headers = headers

    def get_content_type(self) -> str:
        """Gets the Content-type value."""
        return self.__content_type

    def get_status(self) -> str:
        """Gets the HTTP status."""
        return self.__status

    def get_output_bytes(self) -> bytes:
        """Gets the encoded output."""
        return self.__output_bytes

    def get_headers(self) -> List[Tuple[str, str]]:
        """Gets the HTTP headers."""
        return self.__headers


def send_response(
        environ: Dict[str, Any],
        start_response: 'StartResponse',
        response: Response
) -> Iterable[bytes]:
    """Turns an output string into a byte array and sends it."""
    content_type = response.get_content_type()
    if content_type != "application/octet-stream":
        content_type += "; charset=utf-8"
    accept_encodings = environ.get("HTTP_ACCEPT_ENCODING", "").split(",")
    accepts_gzip = "gzip" in accept_encodings
    output_bytes = response.get_output_bytes()
    if accepts_gzip:
        output_bytes = xmlrpc.client.gzip_encode(output_bytes)
    content_length = len(output_bytes)
    headers = [('Content-type', content_type),
               ('Content-Length', str(content_length))]
    if accepts_gzip:
        headers.append(("Content-Encoding", "gzip"))
    headers += response.get_headers()
    status = response.get_status()
    start_response(status, headers)
    return [output_bytes]


def handle_exception(
        environ: Dict[str, Any],
        start_response: 'StartResponse',
        error: str
) -> Iterable[bytes]:
    """Displays an unhandled exception on the page."""
    status = '500 Internal Server Error'
    path_info = environ.get("PATH_INFO")
    request_uri = path_info
    doc = yattag.Doc()
    util.write_html_header(doc)
    with doc.tag("pre", []):
        doc.text(tr("Internal error when serving {0}").format(request_uri) + "\n")
        doc.text(error)
    response_properties = Response("text/html", status, util.to_bytes(doc.get_value()), [])
    return send_response(environ, start_response, response_properties)


def handle_404() -> yattag.Doc:
    """Displays a not-found page."""
    doc = yattag.Doc()
    util.write_html_header(doc)
    with doc.tag("html", []):
        with doc.tag("body", []):
            with doc.tag("h1", []):
                doc.text(tr("Not Found"))
            with doc.tag("p", []):
                doc.text(tr("The requested URL was not found on this server."))
    return doc


def format_timestamp(timestamp: float) -> str:
    """Formats timestamp as UI date-time."""
    date_time = datetime.datetime.fromtimestamp(timestamp)
    fmt = '%Y-%m-%d %H:%M'
    return date_time.strftime(fmt)


def handle_stats_cityprogress(ctx: context.Context, relations: areas.Relations) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/cityprogress."""
    doc = yattag.Doc()
    doc.append_value(get_toolbar(ctx, relations).get_value())

    ref_citycounts: Dict[str, int] = {}
    with open(ctx.get_ini().get_reference_citycounts_path(), "r") as stream:
        first = True
        for line in stream:
            if first:
                first = False
                continue
            cells = line.strip().split('\t')
            if len(cells) < 2:
                continue
            city = cells[0]
            count = int(cells[1])
            ref_citycounts[city] = count
    today = time.strftime("%Y-%m-%d", time.gmtime(ctx.get_time().now()))
    osm_citycounts: Dict[str, int] = {}
    with open(ctx.get_ini().get_workdir() + "/stats/" + today + ".citycount", "r") as stream:
        for line in stream:
            cells = line.strip().split('\t')
            if len(cells) < 2:
                continue
            city = cells[0]
            count = int(cells[1])
            osm_citycounts[city] = count
    ref_cities = [util.Street.from_string(i) for i in ref_citycounts]
    osm_cities = [util.Street.from_string(i) for i in osm_citycounts]
    cities = [i.get_osm_name() for i in util.get_in_both(ref_cities, osm_cities)]
    cities.sort(key=util.get_sort_key)
    table = []
    table.append([yattag.Doc.from_text(tr("City name")),
                  yattag.Doc.from_text(tr("House number coverage")),
                  yattag.Doc.from_text(tr("OSM count")),
                  yattag.Doc.from_text(tr("Reference count"))])
    for city in cities:
        percent = "100.00"
        if ref_citycounts[city] > 0 and osm_citycounts[city] < ref_citycounts[city]:
            percent = "%.2f" % (osm_citycounts[city] / ref_citycounts[city] * 100)
        table.append([yattag.Doc.from_text(city),
                      yattag.Doc.from_text(util.format_percent(percent)),
                      yattag.Doc.from_text(str(osm_citycounts[city])),
                      yattag.Doc.from_text(str(ref_citycounts[city]))])
    doc.append_value(util.html_table_from_list(table).get_value())

    with doc.tag("h2", []):
        doc.text(tr("Note"))
    with doc.tag("div", []):
        doc.text(tr("""These statistics are estimates, not taking house number filters into account.
Only cities with house numbers in OSM are considered."""))

    doc.append_value(get_footer().get_value())
    return doc


def handle_invalid_refstreets(ctx: context.Context, relations: areas.Relations) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/invalid-relations."""
    doc = yattag.Doc()
    doc.append_value(get_toolbar(ctx, relations).get_value())

    prefix = ctx.get_ini().get_uri_prefix()
    for relation in relations.get_relations():
        if not ctx.get_file_system().path_exists(relation.get_files().get_osm_streets_path()):
            continue
        invalid_refstreets = relation.get_invalid_refstreets()
        osm_invalids, ref_invalids = invalid_refstreets
        key_invalids = relation.get_invalid_filter_keys()
        if not osm_invalids and not ref_invalids and not key_invalids:
            continue
        with doc.tag("h1", []):
            relation_name = relation.get_name()
            with doc.tag("a", [("href", prefix + "/streets/" + relation_name + "/view-result")]):
                doc.text(relation_name)
        doc.append_value(util.invalid_refstreets_to_html(osm_invalids, ref_invalids).get_value())
        doc.append_value(util.invalid_filter_keys_to_html(key_invalids).get_value())

    doc.append_value(get_footer().get_value())
    return doc


def handle_stats(ctx: context.Context, relations: areas.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/."""
    if request_uri.endswith("/cityprogress"):
        return handle_stats_cityprogress(ctx, relations)

    if request_uri.endswith("/invalid-relations"):
        return handle_invalid_refstreets(ctx, relations)

    doc = yattag.Doc()
    doc.append_value(get_toolbar(ctx, relations).get_value())

    prefix = ctx.get_ini().get_uri_prefix()

    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-daily-title", tr("New house numbers, last 2 weeks, as of {}")),
            ("str-daily-x-axis", tr("During this day")),
            ("str-daily-y-axis", tr("New house numbers")),
            ("str-monthly-title", tr("New house numbers, last year, as of {}")),
            ("str-monthly-x-axis", tr("During this month")),
            ("str-monthly-y-axis", tr("New house numbers")),
            ("str-monthlytotal-title", tr("All house numbers, last year, as of {}")),
            ("str-monthlytotal-x-axis", tr("Latest for this month")),
            ("str-monthlytotal-y-axis", tr("All house numbers")),
            ("str-dailytotal-title", tr("All house numbers, last 2 weeks, as of {}")),
            ("str-dailytotal-x-axis", tr("At the start of this day")),
            ("str-dailytotal-y-axis", tr("All house numbers")),
            ("str-topusers-title", tr("Top house number editors, as of {}")),
            ("str-topusers-x-axis", tr("User name")),
            ("str-topusers-y-axis", tr("Number of house numbers last changed by this user")),
            ("str-topcities-title", tr("Top edited cities, as of {}")),
            ("str-topcities-x-axis", tr("City name")),
            ("str-topcities-y-axis", tr("Number of house numbers added in the past 30 days")),
            ("str-topcities-empty", tr("(empty)")),
            ("str-topcities-invalid", tr("(invalid)")),
            ("str-usertotal-title", tr("Number of house number editors, as of {}")),
            ("str-usertotal-x-axis", tr("All editors")),
            ("str-usertotal-y-axis", tr("Number of editors, at least one housenumber is last changed by these users")),
            ("str-progress-title", tr("Coverage is {1}%, as of {2}")),
            ("str-progress-x-axis", tr("Number of house numbers in database")),
            ("str-progress-y-axis", tr("Data source")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass

    title_ids = [
        (tr("New house numbers"), "daily"),
        (tr("All house numbers"), "dailytotal"),
        (tr("New house numbers, monthly"), "monthly"),
        (tr("All house numbers, monthly"), "monthlytotal"),
        (tr("Top house number editors"), "topusers"),
        (tr("Top edited cities"), "topcities"),
        (tr("All house number editors"), "usertotal"),
        (tr("Coverage"), "progress"),
        (tr("Per-city coverage"), "cityprogress"),
        (tr("Invalid relation settings"), "invalid-relations"),
    ]

    with doc.tag("ul", []):
        for title, identifier in title_ids:
            with doc.tag("li", []):
                if identifier == "cityprogress":
                    with doc.tag("a", [("href", prefix + "/housenumber-stats/hungary/cityprogress")]):
                        doc.text(title)
                    continue
                if identifier == "invalid-relations":
                    with doc.tag("a", [("href", prefix + "/housenumber-stats/hungary/invalid-relations")]):
                        doc.text(title)
                    continue
                with doc.tag("a", [("href", "#_" + identifier)]):
                    doc.text(title)

    for title, identifier in title_ids:
        if identifier in ("cityprogress", "invalid-relations"):
            continue
        with doc.tag("h2", [("id", "_" + identifier)]):
            doc.text(title)

        with doc.tag("div", [("class", "canvasblock js")]):
            with doc.tag("canvas", [("id", identifier)]):
                pass

    with doc.tag("h2", []):
        doc.text(tr("Note"))
    with doc.tag("div", []):
        doc.text(tr("""These statistics are provided purely for interested editors, and are not
intended to reflect quality of work done by any given editor in OSM. If you want to use
them to motivate yourself, that's fine, but keep in mind that a bit of useful work is
more meaningful than a lot of useless work."""))

    doc.append_value(get_footer().get_value())
    return doc


def get_request_uri(environ: Dict[str, Any], ctx: context.Context, relations: areas.Relations) -> str:
    """Finds out the request URI."""
    request_uri = cast(str, environ.get("PATH_INFO"))

    prefix = ctx.get_ini().get_uri_prefix()
    if request_uri:
        # Compatibility.
        if request_uri.startswith(prefix + "/suspicious-streets/"):
            request_uri = request_uri.replace('suspicious-streets', 'missing-housenumbers')
        elif request_uri.startswith(prefix + "/suspicious-relations/"):
            request_uri = request_uri.replace('suspicious-relations', 'missing-streets')

        # Performance: don't bother with relation aliases for non-relation requests.
        if not request_uri.startswith(prefix + "/streets/") \
                and not request_uri.startswith(prefix + "/missing-streets/") \
                and not request_uri.startswith(prefix + "/street-housenumbers/") \
                and not request_uri.startswith(prefix + "/missing-housenumbers/"):
            return request_uri

        # Relation aliases.
        aliases = relations.get_aliases()
        tokens = request_uri.split("/")
        relation_name = tokens[-2]
        if relation_name in aliases:
            request_uri = request_uri.replace(relation_name, aliases[relation_name])

    return request_uri


def check_existing_relation(ctx: context.Context, relations: areas.Relations, request_uri: str) -> yattag.Doc:
    """Prevents serving outdated data from a relation that has been renamed."""
    doc = yattag.Doc()
    prefix = ctx.get_ini().get_uri_prefix()
    if not request_uri.startswith(prefix + "/streets/") \
            and not request_uri.startswith(prefix + "/missing-streets/") \
            and not request_uri.startswith(prefix + "/additional-streets/") \
            and not request_uri.startswith(prefix + "/street-housenumbers/") \
            and not request_uri.startswith(prefix + "/missing-housenumbers/"):
        return doc

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    if relation_name in relations.get_names():
        return doc

    with doc.tag("div", [("id", "no-such-relation-error")]):
        doc.text(tr("No such relation: {0}").format(relation_name))
    return doc


def handle_no_osm_streets(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-osm-streets error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/streets/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-osm-streets")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No existing streets: call Overpass to create..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-overpass-wait", tr("No existing streets: waiting for Overpass...")),
            ("str-overpass-error", tr("Error from Overpass: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_no_osm_housenumbers(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/street-housenumbers/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-osm-housenumbers")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No existing house numbers: call Overpass to create..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-overpass-wait", tr("No existing house numbers: waiting for Overpass...")),
            ("str-overpass-error", tr("Error from Overpass: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_no_ref_housenumbers(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/missing-housenumbers/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-ref-housenumbers")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No reference house numbers: create from reference..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-reference-wait", tr("No reference house numbers: creating from reference...")),
            ("str-reference-error", tr("Error from reference: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_no_ref_streets(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-ref-streets error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/missing-streets/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-ref-streets")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No street list: create from reference..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-reference-wait", tr("No reference streets: creating from reference...")),
            ("str-reference-error", tr("Error from reference: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_github_webhook(environ: Dict[str, Any], ctx: context.Context) -> yattag.Doc:
    """Handles a GitHub style webhook."""

    body = urllib.parse.parse_qs(util.from_bytes(environ["wsgi.input"].read()))
    payload = body["payload"][0]
    root = json.loads(payload)
    if root["ref"] == "refs/heads/master":
        my_env: Dict[str, str] = {}
        my_env["PATH"] = "osm-gimmisn-env/bin:" + os.environ["PATH"]
        ctx.get_subprocess().run(["make", "-C", ctx.get_abspath(""), "deploy"], env=my_env)

    return yattag.Doc.from_text("")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
