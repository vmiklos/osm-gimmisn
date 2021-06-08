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
import locale
import os
import time
import traceback
import xmlrpc.client

import pytz
import yattag

from i18n import translate as _
import areas
import config
import util
import version

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_footer(last_updated: str = "") -> yattag.doc.Doc:
    """Produces the end of the page."""
    items: List[yattag.doc.Doc] = []
    doc = yattag.doc.Doc()
    doc.text(_("Version: "))
    doc.asis(util.git_link(version.VERSION, "https://github.com/vmiklos/osm-gimmisn/commit/").getvalue())
    items.append(doc)
    items.append(util.html_escape(_("OSM data © OpenStreetMap contributors.")))
    if last_updated:
        items.append(util.html_escape(_("Last update: ") + last_updated))
    doc = yattag.doc.Doc()
    doc.stag("hr")
    with doc.tag("div"):
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.asis(item.getvalue())
    return doc


def fill_header_function(conf: config.Config, function: str, relation_name: str, items: List[yattag.doc.Doc]) -> None:
    """Fills items with function-specific links in the header. Returns a title."""
    prefix = conf.get_uri_prefix()
    if function == "missing-housenumbers":
        # The OSM data source changes much more frequently than the ref one, so add a dedicated link
        # to update OSM house numbers first.
        doc = yattag.doc.Doc()
        with doc.tag("span", id="trigger-street-housenumbers-update"):
            with doc.tag("a", href=prefix + "/street-housenumbers/" + relation_name + "/update-result"):
                doc.text(_("Update from OSM"))
        items.append(doc)

        doc = yattag.doc.Doc()
        with doc.tag("span", id="trigger-missing-housenumbers-update"):
            with doc.tag("a", href=prefix + "/missing-housenumbers/" + relation_name + "/update-result"):
                doc.text(_("Update from reference"))
        items.append(doc)
    elif function in ("missing-streets", "additional-streets"):
        # The OSM data source changes much more frequently than the ref one, so add a dedicated link
        # to update OSM streets first.
        doc = yattag.doc.Doc()
        with doc.tag("span", id="trigger-streets-update"):
            with doc.tag("a", href=prefix + "/streets/" + relation_name + "/update-result"):
                doc.text(_("Update from OSM"))
        items.append(doc)

        doc = yattag.doc.Doc()
        with doc.tag("span", id="trigger-missing-streets-update"):
            with doc.tag("a", href=prefix + "/missing-streets/" + relation_name + "/update-result"):
                doc.text(_("Update from reference"))
        items.append(doc)
    elif function == "street-housenumbers":
        doc = yattag.doc.Doc()
        with doc.tag("span", id="trigger-street-housenumbers-update"):
            with doc.tag("a", href=prefix + "/street-housenumbers/" + relation_name + "/update-result"):
                doc.text(_("Call Overpass to update"))
        items.append(doc)
        doc = yattag.doc.Doc()
        with doc.tag("a", href=prefix + "/street-housenumbers/" + relation_name + "/view-query"):
            doc.text(_("View query"))
        items.append(doc)
    elif function == "streets":
        doc = yattag.doc.Doc()
        with doc.tag("span", id="trigger-streets-update"):
            with doc.tag("a", href=prefix + "/streets/" + relation_name + "/update-result"):
                doc.text(_("Call Overpass to update"))
        items.append(doc)
        doc = yattag.doc.Doc()
        with doc.tag("a", href=prefix + "/streets/" + relation_name + "/view-query"):
            doc.text(_("View query"))
        items.append(doc)


def fill_missing_header_items(
    conf: config.Config,
    streets: str,
    additional_housenumbers: bool,
    relation_name: str,
    items: List[yattag.doc.Doc]
) -> None:
    """Generates the 'missing house numbers/streets' part of the header."""
    prefix = conf.get_uri_prefix()
    if streets != "only":
        doc = yattag.doc.Doc()
        with doc.tag("a", href=prefix + "/missing-housenumbers/" + relation_name + "/view-result"):
            doc.text(_("Missing house numbers"))
        items.append(doc)

        if additional_housenumbers:
            doc = yattag.doc.Doc()
            with doc.tag("a", href=prefix + "/additional-housenumbers/" + relation_name + "/view-result"):
                doc.text(_("Additional house numbers"))
            items.append(doc)
    if streets != "no":
        doc = yattag.doc.Doc()
        with doc.tag("a", href=prefix + "/missing-streets/" + relation_name + "/view-result"):
            doc.text(_("Missing streets"))
        items.append(doc)
        doc = yattag.doc.Doc()
        with doc.tag("a", href=prefix + "/additional-streets/" + relation_name + "/view-result"):
            doc.text(_("Additional streets"))
        items.append(doc)


def fill_existing_header_items(
    conf: config.Config,
    streets: str,
    relation_name: str,
    items: List[yattag.doc.Doc]
) -> None:
    """Generates the 'existing house numbers/streets' part of the header."""
    prefix = conf.get_uri_prefix()
    if streets != "only":
        doc = yattag.doc.Doc()
        with doc.tag("a", href=prefix + "/street-housenumbers/" + relation_name + "/view-result"):
            doc.text(_("Existing house numbers"))
        items.append(doc)

    doc = yattag.doc.Doc()
    with doc.tag("a", href=prefix + "/streets/" + relation_name + "/view-result"):
        doc.text(_("Existing streets"))
    items.append(doc)


def get_toolbar(
        conf: config.Config,
        relations: Optional[areas.Relations] = None,
        function: str = "",
        relation_name: str = "",
        relation_osmid: int = 0
) -> yattag.doc.Doc:
    """Produces the start of the page. Note that the content depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    items: List[yattag.doc.Doc] = []

    if relations and relation_name:
        relation = relations.get_relation(relation_name)
        streets = relation.get_config().should_check_missing_streets()
        additional_housenumbers = relation.get_config().should_check_additional_housenumbers()

    doc = yattag.doc.Doc()
    with doc.tag("a", href=conf.get_uri_prefix() + "/"):
        doc.text(_("Area list"))
    items.append(doc)

    if relation_name:
        fill_missing_header_items(conf, streets, additional_housenumbers, relation_name, items)

    fill_header_function(conf, function, relation_name, items)

    if relation_name:
        fill_existing_header_items(conf, streets, relation_name, items)

    doc = yattag.doc.Doc()

    # Emit localized strings for JS purposes.
    with doc.tag("div", style="display: none;"):
        string_pairs = [
            ("str-toolbar-overpass-wait", _("Waiting for Overpass...")),
            ("str-toolbar-overpass-error", _("Error from Overpass: ")),
            ("str-toolbar-reference-wait", _("Creating from reference...")),
            ("str-toolbar-reference-error", _("Error from reference: ")),
        ]
        for key, value in string_pairs:
            kwargs: Dict[str, str] = {}
            kwargs["id"] = key
            kwargs["data-value"] = value
            with doc.tag("div", **kwargs):
                pass

    with doc.tag("a", href="https://overpass-turbo.eu/"):
        doc.text(_("Overpass turbo"))
    items.append(doc)

    if relation_osmid:
        doc = yattag.doc.Doc()
        with doc.tag("a", href="https://www.openstreetmap.org/relation/" + str(relation_osmid)):
            doc.text(_("Area boundary"))
        items.append(doc)
    else:
        # These are on the main page only.
        doc = yattag.doc.Doc()
        with doc.tag("a", href=conf.get_uri_prefix() + "/housenumber-stats/hungary/"):
            doc.text(_("Statistics"))
        items.append(doc)

        doc = yattag.doc.Doc()
        with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
            doc.text(_("Documentation"))
        items.append(doc)

    doc = yattag.doc.Doc()
    with doc.tag("div", id="toolbar"):
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.asis(item.getvalue())
    doc.stag("hr")
    return doc


def handle_static(conf: config.Config, request_uri: str) -> Tuple[bytes, str, List[Tuple[str, str]]]:
    """Handles serving static content."""
    tokens = request_uri.split("/")
    path = tokens[-1]
    extra_headers: List[Tuple[str, str]] = []

    if request_uri.endswith(".js"):
        content_type = "application/x-javascript"
        content = util.get_content(config.get_abspath("builddir"), path, extra_headers)
        return content, content_type, extra_headers
    if request_uri.endswith(".css"):
        content_type = "text/css"
        content = util.get_content(conf.get_workdir(), path, extra_headers)
        return content, content_type, extra_headers
    if request_uri.endswith(".json"):
        content_type = "application/json"
        content = util.get_content(os.path.join(conf.get_workdir(), "stats"), path, extra_headers)
        return content, content_type, extra_headers
    if request_uri.endswith(".ico"):
        content_type = "image/x-icon"
        content = util.get_content(config.get_abspath(""), path, extra_headers)
        return content, content_type, extra_headers
    if request_uri.endswith(".svg"):
        content_type = "image/svg+xml"
        content = util.get_content(config.get_abspath(""), path, extra_headers)
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
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """Displays an unhandled exception on the page."""
    status = '500 Internal Server Error'
    path_info = environ.get("PATH_INFO")
    request_uri = path_info
    doc = yattag.doc.Doc()
    util.write_html_header(doc)
    with doc.tag("pre"):
        doc.text(_("Internal error when serving {0}").format(request_uri) + "\n")
        doc.text(traceback.format_exc())
    response_properties = Response("text/html", status, doc.getvalue().encode("utf-8"), [])
    return send_response(environ, start_response, response_properties)


def handle_404() -> yattag.doc.Doc:
    """Displays a not-found page."""
    doc = yattag.doc.Doc()
    util.write_html_header(doc)
    with doc.tag("html"):
        with doc.tag("body"):
            with doc.tag("h1"):
                doc.text(_("Not Found"))
            with doc.tag("p"):
                doc.text(_("The requested URL was not found on this server."))
    return doc


def local_to_ui_tz(conf: config.Config, local_dt: datetime.datetime) -> datetime.datetime:
    """Converts from local date-time to UI date-time, based on config."""
    if conf.has_value("timezone"):
        ui_tz = pytz.timezone(conf.get_timezone())
    else:
        ui_tz = pytz.timezone("Europe/Budapest")

    return local_dt.astimezone(ui_tz)


def format_timestamp(conf: config.Config, timestamp: float) -> str:
    """Formats timestamp as UI date-time."""
    local_dt = datetime.datetime.fromtimestamp(timestamp)
    ui_dt = local_to_ui_tz(conf, local_dt)
    fmt = '%Y-%m-%d %H:%M'
    return ui_dt.strftime(fmt)


def handle_stats_cityprogress(conf: config.Config, relations: areas.Relations) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/cityprogress."""
    doc = yattag.doc.Doc()
    doc.asis(get_toolbar(conf, relations).getvalue())

    ref_citycounts: Dict[str, int] = {}
    with open(conf.get_reference_citycounts_path(), "r") as stream:
        first = True
        for line in stream.readlines():
            if first:
                first = False
                continue
            cells = line.strip().split('\t')
            if len(cells) < 2:
                continue
            city = cells[0]
            count = int(cells[1])
            ref_citycounts[city] = count
    today = time.strftime("%Y-%m-%d")
    osm_citycounts: Dict[str, int] = {}
    with open(conf.get_workdir() + "/stats/" + today + ".citycount", "r") as stream:
        for line in stream.readlines():
            cells = line.strip().split('\t')
            if len(cells) < 2:
                continue
            city = cells[0]
            count = int(cells[1])
            osm_citycounts[city] = count
    cities = util.get_in_both(list(ref_citycounts.keys()), list(osm_citycounts.keys()))
    cities.sort(key=locale.strxfrm)
    table = []
    table.append([util.html_escape(_("City name")),
                  util.html_escape(_("House number coverage")),
                  util.html_escape(_("OSM count")),
                  util.html_escape(_("Reference count"))])
    for city in cities:
        percent = "100.00"
        if ref_citycounts[city] > 0 and osm_citycounts[city] < ref_citycounts[city]:
            percent = "%.2f" % (osm_citycounts[city] / ref_citycounts[city] * 100)
        table.append([util.html_escape(city),
                      util.html_escape(util.format_percent(percent)),
                      util.html_escape(str(osm_citycounts[city])),
                      util.html_escape(str(ref_citycounts[city]))])
    doc.asis(util.html_table_from_list(table).getvalue())

    with doc.tag("h2"):
        doc.text(_("Note"))
    with doc.tag("div"):
        doc.text(_("""These statistics are estimates, not taking house number filters into account.
Only cities with house numbers in OSM are considered."""))

    doc.asis(get_footer().getvalue())
    return doc


def handle_invalid_refstreets(conf: config.Config, relations: areas.Relations) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/invalid-relations."""
    doc = yattag.doc.Doc()
    doc.asis(get_toolbar(conf, relations).getvalue())

    prefix = conf.get_uri_prefix()
    for relation in relations.get_relations():
        if not os.path.exists(relation.get_files().get_osm_streets_path()):
            continue
        invalid_refstreets = relation.get_invalid_refstreets()
        osm_invalids, ref_invalids = invalid_refstreets
        key_invalids = relation.get_invalid_filter_keys()
        if not osm_invalids and not ref_invalids and not key_invalids:
            continue
        with doc.tag("h1"):
            relation_name = relation.get_name()
            with doc.tag("a", href=prefix + "/streets/" + relation_name + "/view-result"):
                doc.text(relation_name)
        doc.asis(util.invalid_refstreets_to_html(invalid_refstreets).getvalue())
        doc.asis(util.invalid_filter_keys_to_html(key_invalids).getvalue())

    doc.asis(get_footer().getvalue())
    return doc


def handle_stats(conf: config.Config, relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/."""
    if request_uri.endswith("/cityprogress"):
        return handle_stats_cityprogress(conf, relations)

    if request_uri.endswith("/invalid-relations"):
        return handle_invalid_refstreets(conf, relations)

    doc = yattag.doc.Doc()
    doc.asis(get_toolbar(conf, relations).getvalue())

    prefix = conf.get_uri_prefix()

    # Emit localized strings for JS purposes.
    with doc.tag("div", style="display: none;"):
        string_pairs = [
            ("str-daily-title", _("New house numbers, last 2 weeks, as of {}")),
            ("str-daily-x-axis", _("During this day")),
            ("str-daily-y-axis", _("New house numbers")),
            ("str-monthly-title", _("New house numbers, last year, as of {}")),
            ("str-monthly-x-axis", _("During this month")),
            ("str-monthly-y-axis", _("New house numbers")),
            ("str-monthlytotal-title", _("All house numbers, last year, as of {}")),
            ("str-monthlytotal-x-axis", _("Latest for this month")),
            ("str-monthlytotal-y-axis", _("All house numbers")),
            ("str-dailytotal-title", _("All house numbers, last 2 weeks, as of {}")),
            ("str-dailytotal-x-axis", _("At the start of this day")),
            ("str-dailytotal-y-axis", _("All house numbers")),
            ("str-topusers-title", _("Top house number editors, as of {}")),
            ("str-topusers-x-axis", _("User name")),
            ("str-topusers-y-axis", _("Number of house numbers last changed by this user")),
            ("str-topcities-title", _("Top edited cities, as of {}")),
            ("str-topcities-x-axis", _("City name")),
            ("str-topcities-y-axis", _("Number of house numbers added in the past 30 days")),
            ("str-topcities-empty", _("(empty)")),
            ("str-topcities-invalid", _("(invalid)")),
            ("str-usertotal-title", _("Number of house number editors, as of {}")),
            ("str-usertotal-x-axis", _("All editors")),
            ("str-usertotal-y-axis", _("Number of editors, at least one housenumber is last changed by these users")),
            ("str-progress-title", _("Coverage is {1}%, as of {2}")),
            ("str-progress-x-axis", _("Number of house numbers in database")),
            ("str-progress-y-axis", _("Data source")),
        ]
        for key, value in string_pairs:
            kwargs: Dict[str, str] = {}
            kwargs["id"] = key
            kwargs["data-value"] = value
            with doc.tag("div", **kwargs):
                pass

    title_ids = [
        (_("New house numbers"), "daily"),
        (_("All house numbers"), "dailytotal"),
        (_("New house numbers, monthly"), "monthly"),
        (_("All house numbers, monthly"), "monthlytotal"),
        (_("Top house number editors"), "topusers"),
        (_("Top edited cities"), "topcities"),
        (_("All house number editors"), "usertotal"),
        (_("Coverage"), "progress"),
        (_("Per-city coverage"), "cityprogress"),
        (_("Invalid relation settings"), "invalid-relations"),
    ]

    with doc.tag("ul"):
        for title, identifier in title_ids:
            with doc.tag("li"):
                if identifier == "cityprogress":
                    with doc.tag("a", href=prefix + "/housenumber-stats/hungary/cityprogress"):
                        doc.text(title)
                    continue
                if identifier == "invalid-relations":
                    with doc.tag("a", href=prefix + "/housenumber-stats/hungary/invalid-relations"):
                        doc.text(title)
                    continue
                with doc.tag("a", href="#_" + identifier):
                    doc.text(title)

    for title, identifier in title_ids:
        if identifier in ("cityprogress", "invalid-relations"):
            continue
        with doc.tag("h2", id="_" + identifier):
            doc.text(title)

        with doc.tag("div", klass="canvasblock js"):
            with doc.tag("canvas", id=identifier):
                pass

    with doc.tag("h2"):
        doc.text(_("Note"))
    with doc.tag("div"):
        doc.text(_("""These statistics are provided purely for interested editors, and are not
intended to reflect quality of work done by any given editor in OSM. If you want to use
them to motivate yourself, that's fine, but keep in mind that a bit of useful work is
more meaningful than a lot of useless work."""))

    doc.asis(get_footer().getvalue())
    return doc


def get_request_uri(environ: Dict[str, Any], conf: config.Config, relations: areas.Relations) -> str:
    """Finds out the request URI."""
    request_uri = cast(str, environ.get("PATH_INFO"))

    prefix = conf.get_uri_prefix()
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


def check_existing_relation(conf: config.Config, relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Prevents serving outdated data from a relation that has been renamed."""
    doc = yattag.doc.Doc()
    prefix = conf.get_uri_prefix()
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

    with doc.tag("div", id="no-such-relation-error"):
        doc.text(_("No such relation: {0}").format(relation_name))
    return doc


def handle_no_osm_streets(prefix: str, relation_name: str) -> yattag.doc.Doc:
    """Handles the no-osm-streets error on a page using JS."""
    doc = yattag.doc.Doc()
    link = prefix + "/streets/" + relation_name + "/update-result"
    with doc.tag("div", id="no-osm-streets"):
        with doc.tag("a", href=link):
            doc.text(_("No existing streets: call Overpass to create..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", style="display: none;"):
        string_pairs = [
            ("str-overpass-wait", _("No existing streets: waiting for Overpass...")),
            ("str-overpass-error", _("Error from Overpass: ")),
        ]
        for key, value in string_pairs:
            kwargs: Dict[str, str] = {}
            kwargs["id"] = key
            kwargs["data-value"] = value
            with doc.tag("div", **kwargs):
                pass
    return doc


def handle_no_osm_housenumbers(prefix: str, relation_name: str) -> yattag.doc.Doc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    doc = yattag.doc.Doc()
    link = prefix + "/street-housenumbers/" + relation_name + "/update-result"
    with doc.tag("div", id="no-osm-housenumbers"):
        with doc.tag("a", href=link):
            doc.text(_("No existing house numbers: call Overpass to create..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", style="display: none;"):
        string_pairs = [
            ("str-overpass-wait", _("No existing house numbers: waiting for Overpass...")),
            ("str-overpass-error", _("Error from Overpass: ")),
        ]
        for key, value in string_pairs:
            kwargs: Dict[str, str] = {}
            kwargs["id"] = key
            kwargs["data-value"] = value
            with doc.tag("div", **kwargs):
                pass
    return doc


def handle_no_ref_housenumbers(prefix: str, relation_name: str) -> yattag.doc.Doc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    doc = yattag.doc.Doc()
    link = prefix + "/missing-housenumbers/" + relation_name + "/update-result"
    with doc.tag("div", id="no-ref-housenumbers"):
        with doc.tag("a", href=link):
            doc.text(_("No reference house numbers: create from reference..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", style="display: none;"):
        string_pairs = [
            ("str-reference-wait", _("No reference house numbers: creating from reference...")),
            ("str-reference-error", _("Error from reference: ")),
        ]
        for key, value in string_pairs:
            kwargs: Dict[str, str] = {}
            kwargs["id"] = key
            kwargs["data-value"] = value
            with doc.tag("div", **kwargs):
                pass
    return doc


def handle_no_ref_streets(prefix: str, relation_name: str) -> yattag.doc.Doc:
    """Handles the no-ref-streets error on a page using JS."""
    doc = yattag.doc.Doc()
    link = prefix + "/missing-streets/" + relation_name + "/update-result"
    with doc.tag("div", id="no-ref-streets"):
        with doc.tag("a", href=link):
            doc.text(_("No street list: create from reference..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", style="display: none;"):
        string_pairs = [
            ("str-reference-wait", _("No reference streets: creating from reference...")),
            ("str-reference-error", _("Error from reference: ")),
        ]
        for key, value in string_pairs:
            kwargs: Dict[str, str] = {}
            kwargs["id"] = key
            kwargs["data-value"] = value
            with doc.tag("div", **kwargs):
                pass
    return doc

# vim:set shiftwidth=4 softtabstop=4 expandtab:
