#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

import configparser
import datetime
import json
import locale
import os
import subprocess
import sys
import traceback
import urllib.parse
from typing import Any
from typing import Callable
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import wsgiref.simple_server

import pytz
import yattag  # type: ignore

import accept_language
import helpers
import i18n
from i18n import translate as _
import overpass_query
import version
import util

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_config() -> configparser.ConfigParser:
    """Gets access to information which are specific to this installation."""
    config = configparser.ConfigParser()
    config_path = helpers.get_abspath("wsgi.ini")
    config.read(config_path)
    return config


def get_datadir() -> str:
    """Gets the directory which is tracked (in version control) data."""
    return helpers.get_abspath("data")


def get_staticdir() -> str:
    """Gets the directory which is static data."""
    return os.path.join(os.path.dirname(__file__), "static")


def handle_streets(relations: helpers.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/streets/ormezo/view-query."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.Doc()
    doc.asis(get_toolbar(relations, "streets", relation_name, osmrelation).getvalue())

    if action == "view-query":
        with doc.tag("pre"):
            doc.text(relation.get_osm_streets_query())
    elif action == "view-result":
        with relation.get_files().get_osm_streets_stream("r") as sock:
            table = helpers.tsv_to_list(sock)
            doc.asis(helpers.html_table_from_list(table).getvalue())
    elif action == "update-result":
        query = relation.get_osm_streets_query()
        try:
            relation.get_files().write_osm_streets(overpass_query.overpass_query(query))
            streets = relation.get_config().should_check_missing_streets()
            if streets != "only":
                doc.text(_("Update successful: "))
                link = "/osm/suspicious-streets/" + relation_name + "/view-result"
                doc.asis(gen_link(link, _("View missing house numbers")).getvalue())
            else:
                doc.text(_("Update successful."))
        except urllib.error.HTTPError as http_error:
            doc.asis(util.handle_overpass_error(http_error).getvalue())

    date = get_streets_last_modified(relation)
    doc.asis(get_footer(date).getvalue())
    return doc


def handle_street_housenumbers(relations: helpers.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.Doc()
    doc.asis(get_toolbar(relations, "street-housenumbers", relation_name, osmrelation).getvalue())

    if action == "view-query":
        with doc.tag("pre"):
            doc.text(relation.get_osm_housenumbers_query())
    elif action == "view-result":
        with relation.get_files().get_osm_housenumbers_stream(mode="r") as sock:
            table = helpers.tsv_to_list(sock)
            doc.asis(helpers.html_table_from_list(table).getvalue())
    elif action == "update-result":
        query = relation.get_osm_housenumbers_query()
        try:
            relation.get_files().write_osm_housenumbers(overpass_query.overpass_query(query))
            doc.text(_("Update successful: "))
            link = "/osm/suspicious-streets/" + relation_name + "/view-result"
            doc.asis(gen_link(link, _("View missing house numbers")).getvalue())
        except urllib.error.HTTPError as http_error:
            doc.asis(util.handle_overpass_error(http_error).getvalue())

    date = get_housenumbers_last_modified(relation)
    doc.asis(get_footer(date).getvalue())
    return doc


def gen_link(url: str, label: str) -> yattag.Doc:
    """Generates a link to a URL with a given label."""
    doc = yattag.Doc()
    with doc.tag("a", href=url):
        doc.text(label + "...")

    # Always auto-visit the link for now.
    with doc.tag("script", type="text/javascript"):
        doc.text("window.location.href = \"%s\";" % url)

    return doc


def missing_housenumbers_view_res(relations: helpers.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.Doc()
    relation = relations.get_relation(relation_name)
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        doc.text(_("No existing streets: "))
        link = "/osm/streets/" + relation_name + "/update-result"
        doc.asis(gen_link(link, _("Call Overpass to create")).getvalue())
    elif not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
        doc.text(_("No existing house numbers: "))
        link = "/osm/street-housenumbers/" + relation_name + "/update-result"
        doc.asis(gen_link(link, _("Call Overpass to create")).getvalue())
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        doc.text(_("No missing house numbers: "))
        link = "/osm/suspicious-streets/" + relation_name + "/update-result"
        doc.asis(gen_link(link, _("Create from reference")).getvalue())
    else:
        ret = relation.write_missing_housenumbers()
        todo_street_count, todo_count, done_count, percent, table = ret

        with doc.tag("p"):
            doc.text(_("OpenStreetMap is possibly missing the below {0} house numbers for {1} streets.")
                     .format(str(todo_count), str(todo_street_count)))
            doc.text(_(" (existing: {0}, ready: {1}%).").format(str(done_count), str(percent)))
            doc.stag("br")
            with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
                doc.text(_("Filter incorrect information"))
            doc.text(".")

        doc.asis(helpers.html_table_from_list(table).getvalue())
    return doc


def missing_relations_view_result(relations: helpers.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/suspicious-relations/budapest_11/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    doc = yattag.Doc()
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        doc.text(_("No existing streets: "))
        with doc.tag("a", href="/osm/streets/" + relation_name + "/update-result"):
            doc.text(_("Call Overpass to create"))
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        doc.text(_("No street list: "))
        with doc.tag("a", href="/osm/suspicious-relations/" + relation_name + "/update-result"):
            doc.text(_("Create from reference"))
    else:
        ret = relation.write_missing_streets()
        todo_count, done_count, percent, streets = ret
        streets.sort(key=locale.strxfrm)
        table = [[util.html_escape(_("Street name"))]]
        for street in streets:
            table.append([util.html_escape(street)])

        with doc.tag("p"):
            doc.text(_("OpenStreetMap is possibly missing the below {0} streets.").format(str(todo_count)))
            doc.text(_(" (existing: {0}, ready: {1}%).").format(str(done_count), str(percent)))

        doc.asis(helpers.html_table_from_list(table).getvalue())
    return doc


def missing_housenumbers_view_txt(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets")
    elif not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
        output += _("No existing house numbers")
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        output += _("No reference house numbers")
    else:
        ongoing_streets, _ignore = relation.get_missing_housenumbers()

        table = []
        for result in ongoing_streets:
            if result[1]:
                # Street name, only_in_reference items.
                if not relation.get_config().get_street_is_even_odd(result[0]):
                    row = result[0] + "\t[" + ", ".join(result[1]) + "]"
                else:
                    elements = util.format_even_odd(result[1], doc=None)
                    row = result[0] + "\t[" + "], [".join(elements) + "]"
                table.append(row)
        table.sort(key=locale.strxfrm)
        output += "\n".join(table)
    return output


def missing_streets_view_txt(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets")
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        output += _("No reference streets")
    else:
        todo_streets, _ignore = relation.get_missing_streets()
        todo_streets.sort(key=locale.strxfrm)
        output += "\n".join(todo_streets)
    return output


def missing_housenumbers_update(relations: helpers.Relations, relation_name: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/update-result."""
    reference = get_config().get('wsgi', 'reference_housenumbers').strip().split(' ')
    reference = [helpers.get_abspath(i) for i in reference]
    relation = relations.get_relation(relation_name)
    relation.write_ref_housenumbers(reference)
    doc = yattag.Doc()
    doc.text(_("Update successful: "))
    link = "/osm/suspicious-streets/" + relation_name + "/view-result"
    doc.asis(gen_link(link, _("View missing house numbers")).getvalue())
    return doc


def missing_streets_update(relations: helpers.Relations, relation_name: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/update-result."""
    reference = helpers.get_abspath(get_config().get('wsgi', 'reference_street').strip())
    relation = relations.get_relation(relation_name)
    relation.write_ref_streets(reference)
    return util.html_escape(_("Update successful."))


def handle_missing_housenumbers(relations: helpers.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    date = None

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()
    doc = yattag.Doc()
    doc.asis(get_toolbar(relations, "suspicious-streets", relation_name, osmrelation).getvalue())

    if action == "view-result":
        doc.asis(missing_housenumbers_view_res(relations, request_uri).getvalue())
    elif action == "view-query":
        with doc.tag("pre"):
            with relation.get_files().get_ref_housenumbers_stream("r") as sock:
                doc.text(sock.read())
        date = get_last_modified(relation.get_files().get_ref_housenumbers_path())
    elif action == "update-result":
        doc.asis(missing_housenumbers_update(relations, relation_name).getvalue())

    if not date:
        date = ref_housenumbers_last_modified(relations, relation_name)
    doc.asis(get_footer(date).getvalue())
    return doc


def handle_missing_streets(relations: helpers.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.Doc()
    doc.asis(get_toolbar(relations, "suspicious-relations", relation_name, osmrelation).getvalue())

    if action == "view-result":
        doc.asis(missing_relations_view_result(relations, request_uri).getvalue())
    elif action == "view-query":
        with doc.tag("pre"):
            with relation.get_files().get_ref_streets_stream("r") as sock:
                doc.text(sock.read())
    elif action == "update-result":
        doc.asis(missing_streets_update(relations, relation_name).getvalue())

    date = ref_streets_last_modified(relation)
    doc.asis(get_footer(date).getvalue())
    return doc


def local_to_ui_tz(local_dt: datetime.datetime) -> datetime.datetime:
    """Converts from local date-time to UI date-time, based on config."""
    config = get_config()
    if config.has_option("wsgi", "timezone"):
        ui_tz = pytz.timezone(config.get("wsgi", "timezone"))
    else:
        ui_tz = pytz.timezone("Europe/Budapest")

    return local_dt.astimezone(ui_tz)


def get_last_modified(workdir: str, path: str = "") -> str:
    """Gets the update date of a file in workdir."""
    if path:
        path = os.path.join(workdir, path)
    else:
        path = workdir
    return format_timestamp(get_timestamp(path))


def get_timestamp(workdir: str, path: str = "") -> float:
    """Gets the timestamp of a file in workdir."""
    if path:
        path = os.path.join(workdir, path)
    else:
        path = workdir
    try:
        return os.path.getmtime(path)
    except FileNotFoundError:
        return 0


def format_timestamp(timestamp: float) -> str:
    """Formats timestamp as UI date-time."""
    local_dt = datetime.datetime.fromtimestamp(timestamp)
    ui_dt = local_to_ui_tz(local_dt)
    fmt = '%Y-%m-%d %H:%M'
    return ui_dt.strftime(fmt)


def ref_housenumbers_last_modified(relations: helpers.Relations, name: str) -> str:
    """Gets the update date for suspicious streets."""
    relation = relations.get_relation(name)
    t_ref = get_timestamp(relation.get_files().get_ref_housenumbers_path())
    t_housenumbers = get_timestamp(relation.get_files().get_osm_housenumbers_path())
    return format_timestamp(max(t_ref, t_housenumbers))


def ref_streets_last_modified(relation: helpers.Relation) -> str:
    """Gets the update date for missing streets."""
    t_ref = get_timestamp(relation.get_files().get_ref_streets_path())
    t_osm = get_timestamp(relation.get_files().get_osm_streets_path())
    return format_timestamp(max(t_ref, t_osm))


def get_housenumbers_last_modified(relation: helpers.Relation) -> str:
    """Gets the update date of house numbers for a relation."""
    return get_last_modified(relation.get_files().get_osm_housenumbers_path())


def get_streets_last_modified(relation: helpers.Relation) -> str:
    """Gets the update date of streets for a relation."""
    return get_last_modified(relation.get_files().get_osm_streets_path())


def handle_main_housenr_percent(relation: helpers.Relation) -> Tuple[yattag.Doc, str]:
    """Handles the house number percent part of the main page."""
    url = "/osm/suspicious-streets/" + relation.get_name() + "/view-result"
    percent = "N/A"
    if os.path.exists(relation.get_files().get_housenumbers_percent_path()):
        percent = helpers.get_content(relation.get_files().get_housenumbers_percent_path())

    doc = yattag.Doc()
    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_housenumbers_percent_path())
        with doc.tag("strong"):
            with doc.tag("a", href=url, title=_("updated") + " " + date):
                doc.text(percent + "%")
        return doc, percent

    with doc.tag("strong"):
        with doc.tag("a", href=url):
            doc.text(_("missing house numbers"))
    return doc, "0"


def handle_main_street_percent(relation: helpers.Relation) -> Tuple[yattag.Doc, str]:
    """Handles the street percent part of the main page."""
    url = "/osm/suspicious-relations/" + relation.get_name() + "/view-result"
    percent = "N/A"
    if os.path.exists(relation.get_files().get_streets_percent_path()):
        percent = helpers.get_content(relation.get_files().get_streets_percent_path())

    doc = yattag.Doc()
    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_streets_percent_path())
        with doc.tag("strong"):
            with doc.tag("a", href=url, title=_("updated") + " " + date):
                doc.text(percent + "%")
        return doc, percent

    with doc.tag("strong"):
        with doc.tag("a", href=url):
            doc.text(_("missing streets"))
    return doc, "0"


def filter_for_everything(_complete: bool, _relation: helpers.Relation) -> bool:
    """Does not filter out anything."""
    return True


def filter_for_incomplete(complete: bool, _relation: helpers.Relation) -> bool:
    """Filters out complete items."""
    return not complete


def create_filter_for_refmegye(refmegye_filter: str) -> Callable[[bool, helpers.Relation], bool]:
    """Creates a function that filters for a single refmegye."""
    return lambda _complete, relation: relation.get_config().get_refmegye() == refmegye_filter


def create_filter_for_refmegye_reftelepules(
        refmegye_filter: str,
        reftelepules_filter: str
) -> Callable[[bool, helpers.Relation], bool]:
    """Creates a function that filters for a single reftelepules in a refmegye."""
    def filter_for(_complete: bool, relation: helpers.Relation) -> bool:
        config = relation.get_config()
        return config.get_refmegye() == refmegye_filter and config.get_reftelepules() == reftelepules_filter
    return filter_for


def handle_main_filters_refmegye(relations: helpers.Relations, refmegye_id: str, refmegye: str) -> yattag.Doc:
    """Handles one refmegye in the filter part of the main wsgi page."""
    doc = yattag.Doc()
    name = relations.refmegye_get_name(refmegye)
    if not name:
        return doc

    with doc.tag("a", href="/osm/filter-for/refmegye/" + refmegye):
        doc.text(name)
    if refmegye_id and refmegye == refmegye_id:
        reftelepules_ids = relations.refmegye_get_reftelepules_ids(refmegye_id)
        if reftelepules_ids:
            names = []  # type: List[yattag.Doc]
            for reftelepules_id in reftelepules_ids:
                name = relations.reftelepules_get_name(refmegye_id, reftelepules_id)
                if name:
                    name_doc = yattag.Doc()
                    href_format = "/osm/filter-for/refmegye/{}/reftelepules/{}"
                    with name_doc.tag("a", href=href_format.format(refmegye, reftelepules_id)):
                        name_doc.text(name)
                    names.append(name_doc)
            if names:
                doc.text(" (")
                for index, item in enumerate(names):
                    if index:
                        doc.text(", ")
                    doc.asis(item.getvalue())
                doc.text(")")
    return doc


def handle_main_filters(relations: helpers.Relations, refmegye_id: str) -> yattag.Doc:
    """Handlers the filter part of the main wsgi page."""
    items = []  # type: List[yattag.Doc]
    doc = yattag.Doc()
    with doc.tag("a", href="/osm/filter-for/incomplete"):
        doc.text(_("Hide complete areas"))
    items.append(doc)
    # Sorted set of refmegye values of all relations.
    for refmegye in sorted({relation.get_config().get_refmegye() for relation in relations.get_relations()}):
        items.append(handle_main_filters_refmegye(relations, refmegye_id, refmegye))
    doc = yattag.Doc()
    with doc.tag("h1"):
        doc.text(_("Where to map?"))
    with doc.tag("p"):
        doc.text(_("Filters:") + " ")
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.asis(item.getvalue())
    return doc


def setup_main_filter_for(request_uri: str) -> Tuple[Callable[[bool, helpers.Relation], bool], str]:
    """Sets up a filter-for function from request uri: only certain areas are shown then."""
    tokens = request_uri.split("/")
    filter_for = filter_for_everything  # type: Callable[[bool, helpers.Relation], bool]
    filters = util.parse_filters(tokens)
    refmegye = ""
    if "incomplete" in filters:
        # /osm/filter-for/incomplete
        filter_for = filter_for_incomplete
    elif "refmegye" in filters and "reftelepules" in filters:
        # /osm/filter-for/refmegye/<value>/reftelepules/<value>.
        refmegye = filters["refmegye"]
        filter_for = create_filter_for_refmegye_reftelepules(filters["refmegye"], filters["reftelepules"])
    elif "refmegye" in filters:
        # /osm/filter-for/refmegye/<value>.
        refmegye = filters["refmegye"]
        filter_for = create_filter_for_refmegye(refmegye)
    return filter_for, refmegye


def handle_main_relation(
        relations: helpers.Relations,
        filter_for: Callable[[bool, helpers.Relation], bool],
        relation_name: str
) -> List[yattag.Doc]:
    """Handles one relation (one table row) on the main page."""
    relation = relations.get_relation(relation_name)
    complete = True

    streets = relation.get_config().should_check_missing_streets()

    row = []  # List[yattag.Doc]
    row.append(util.html_escape(relation_name))

    if streets != "only":
        cell, percent = handle_main_housenr_percent(relation)
        doc = yattag.Doc()
        doc.asis(cell.getvalue())
        row.append(doc)
        if float(percent) < 100.0:
            complete = False

        date = get_housenumbers_last_modified(relation)
        doc = yattag.Doc()
        href = "/osm/street-housenumbers/" + relation_name + "/view-result"
        with doc.tag("a", href=href, title=_("updated") + " " + date):
            doc.text(_("existing house numbers"))
        row.append(doc)
    else:
        row.append(yattag.Doc())

        row.append(yattag.Doc())

    if streets != "no":
        cell, percent = handle_main_street_percent(relation)
        row.append(cell)
        if float(percent) < 100.0:
            complete = False
    else:
        row.append(yattag.Doc())

    date = get_streets_last_modified(relation)
    doc = yattag.Doc()
    with doc.tag("a", href="/osm/streets/" + relation_name + "/view-result", title=_("updated") + " " + date):
        doc.text(_("existing streets"))
    row.append(doc)

    doc = yattag.Doc()
    with doc.tag("a", href="https://www.openstreetmap.org/relation/" + str(relation.get_config().get_osmrelation())):
        doc.text(_("area boundary"))
    row.append(doc)

    if not filter_for(complete, relation):
        row.clear()

    return row


def handle_main(request_uri: str, relations: helpers.Relations) -> yattag.Doc:
    """Handles the main wsgi page.

    Also handles /osm/filter-for/* which filters for a condition."""
    filter_for, refmegye = setup_main_filter_for(request_uri)

    doc = yattag.Doc()
    doc.asis(get_toolbar(relations).getvalue())

    doc.asis(handle_main_filters(relations, refmegye).getvalue())
    table = []
    table.append([util.html_escape(_("Area")),
                  util.html_escape(_("House number coverage")),
                  util.html_escape(_("Existing house numbers")),
                  util.html_escape(_("Street coverage")),
                  util.html_escape(_("Existing streets")),
                  util.html_escape(_("Area boundary"))])
    for relation_name in relations.get_names():
        row = handle_main_relation(relations, filter_for, relation_name)
        if row:
            table.append(row)
    doc.asis(helpers.html_table_from_list(table).getvalue())
    with doc.tag("p"):
        with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
            doc.text(_("Add new area"))

    doc.asis(get_footer().getvalue())
    return doc


def fill_missing_header_items(streets: str, relation_name: str, items: List[yattag.Doc]) -> None:
    """Generates the 'missing house numbers/streets' part of the header."""
    if streets != "only":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/suspicious-streets/" + relation_name + "/view-result"):
            doc.text(_("Missing house numbers"))
        doc.text(" (")
        with doc.tag("a", href="/osm/suspicious-streets/" + relation_name + "/view-result.txt"):
            doc.text("txt")
        doc.text(")")
        items.append(doc)
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/street-housenumbers/" + relation_name + "/view-result"):
            doc.text(_("Existing house numbers"))
        items.append(doc)
    if streets != "no":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/suspicious-relations/" + relation_name + "/view-result"):
            doc.text(_("Missing streets"))
        doc.text(" (")
        with doc.tag("a", href="/osm/suspicious-relations/" + relation_name + "/view-result.txt"):
            doc.text("txt")
        doc.text(")")
        items.append(doc)


def get_html_title(request_uri: str) -> str:
    """Determines the HTML title for a given function and relation name."""
    tokens = request_uri.split("/")
    function = ""
    relation_name = ""
    if len(tokens) > 3:
        function = tokens[2]
        relation_name = tokens[3]
    title = ""
    if function == "suspicious-streets":
        title = " - " + _("{0} missing house numbers").format(relation_name)
    elif function == "suspicious-relations":
        title = " - " + relation_name + " " + _("missing streets")
    elif function == "street-housenumbers":
        title = " - " + relation_name + " " + _("existing house numbers")
    elif function == "streets":
        title = " - " + relation_name + " " + _("existing streets")
    return title


def fill_header_function(function: str, relation_name: str, items: List[yattag.Doc]) -> None:
    """Fills items with function-specific links in the header. Returns a title."""
    if function == "suspicious-streets":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/suspicious-streets/" + relation_name + "/update-result"):
            doc.text(_("Update from reference"))
        doc.text(" " + _("(may take seconds)"))
        items.append(doc)
    elif function == "suspicious-relations":
        doc = yattag.Doc()
        with doc.tag("a", href="/osm/suspicious-relations/" + relation_name + "/update-result"):
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


def write_html_header(doc: yattag.Doc) -> None:
    """Produces the verify first line of a HTML output."""
    doc.asis("<!DOCTYPE html>\n")


def write_html_head(doc: yattag.Doc, title: str) -> None:
    """Produces the <head> tag and its contents."""
    with doc.tag("head"):
        with doc.tag("title"):
            doc.text(_("Where to map?") + title)
        doc.stag("meta", charset="UTF-8")
        doc.stag("link", rel="stylesheet", type="text/css", href="/osm/static/osm.css")
        with doc.tag("script", src="/osm/static/sorttable.js"):
            pass
        doc.stag("meta", name="viewport", content="width=device-width, initial-scale=1")


def get_toolbar(
        relations: Optional[helpers.Relations] = None,
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


def get_footer(last_updated: str = "") -> yattag.Doc:
    """Produces the end of the page."""
    items = []  # type: List[yattag.Doc]
    doc = yattag.Doc()
    doc.text(_("Version: "))
    doc.asis(helpers.git_link(version.VERSION, "https://github.com/vmiklos/osm-gimmisn/commit/").getvalue())
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


def handle_github_webhook(environ: Dict[str, Any]) -> yattag.Doc:
    """Handles a GitHub style webhook."""

    body = urllib.parse.parse_qs(environ["wsgi.input"].read().decode('utf-8'))
    payload = body["payload"][0]
    root = json.loads(payload)
    if root["ref"] == "refs/heads/master":
        subprocess.run(["make", "-C", helpers.get_abspath(""), "deploy-pythonanywhere"], check=True)

    return util.html_escape("")


def handle_static(request_uri: str) -> Tuple[str, str]:
    """Handles serving static content."""
    tokens = request_uri.split("/")
    path = tokens[-1]

    if request_uri.endswith(".js"):
        content_type = "application/x-javascript"
    elif request_uri.endswith(".css"):
        content_type = "text/css"

    if path.endswith(".js") or path.endswith(".css"):
        return helpers.get_content(get_staticdir(), path), content_type

    return "", ""


def setup_localization(environ: Dict[str, Any]) -> str:
    """Provides localized strings for this thread."""
    # Set up localization.
    languages = environ.get("HTTP_ACCEPT_LANGUAGE")
    if languages:
        parsed = accept_language.parse_accept_language(languages)
        if parsed:
            language = parsed[0].language
            i18n.set_language(language)
            return cast(str, language)
    return ""


def send_response(start_response: 'StartResponse', content_type: str, status: str, output: str) -> Iterable[bytes]:
    """Turns an output string into a byte array and sends it."""
    output_bytes = output.encode('utf-8')
    response_headers = [('Content-type', content_type + '; charset=utf-8'),
                        ('Content-Length', str(len(output_bytes)))]
    start_response(status, response_headers)
    return [output_bytes]


def our_application_txt(
        start_response: 'StartResponse',
        relations: helpers.Relations,
        request_uri: str
) -> Iterable[bytes]:
    """Dispatches plain text requests based on their URIs."""
    content_type = "text/plain"
    if request_uri.startswith("/osm/suspicious-relations/"):
        output = missing_streets_view_txt(relations, request_uri)
    elif request_uri.startswith("/osm/suspicious-streets/"):
        output = missing_housenumbers_view_txt(relations, request_uri)
    return send_response(start_response, content_type, "200 OK", output)


def get_request_uri(environ: Dict[str, Any]) -> str:
    """Finds out the request URI."""
    request_uri = ""
    path_info = environ.get("PATH_INFO")
    if path_info:
        request_uri = path_info
    return request_uri


def our_application(
        environ: Dict[str, Any],
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """Dispatches the request based on its URI."""
    config = get_config()
    if config.has_option("wsgi", "locale"):
        ui_locale = config.get("wsgi", "locale")
    else:
        ui_locale = "hu_HU.UTF-8"
    try:
        locale.setlocale(locale.LC_ALL, ui_locale)
    except locale.Error:
        # Ignore, this happens only on the cut-down CI environment.
        pass

    language = setup_localization(environ)
    if not language:
        language = "hu"

    request_uri = get_request_uri(environ)
    _ignore, _ignore, ext = request_uri.partition('.')

    relations = helpers.Relations(get_datadir(), helpers.get_workdir(config))

    if ext == "txt":
        return our_application_txt(start_response, relations, request_uri)

    if request_uri.startswith("/osm/static/"):
        output, content_type = handle_static(request_uri)
        return send_response(start_response, content_type, "200 OK", output)

    doc = yattag.Doc()
    write_html_header(doc)
    with doc.tag("html", lang=language):
        write_html_head(doc, get_html_title(request_uri))

        with doc.tag("body"):
            if request_uri.startswith("/osm/streets/"):
                doc.asis(handle_streets(relations, request_uri).getvalue())
            elif request_uri.startswith("/osm/suspicious-relations/"):
                doc.asis(handle_missing_streets(relations, request_uri).getvalue())
            elif request_uri.startswith("/osm/street-housenumbers/"):
                doc.asis(handle_street_housenumbers(relations, request_uri).getvalue())
            elif request_uri.startswith("/osm/suspicious-streets/"):
                doc.asis(handle_missing_housenumbers(relations, request_uri).getvalue())
            elif request_uri.startswith("/osm/webhooks/github"):
                doc.asis(handle_github_webhook(environ).getvalue())
            else:
                doc.asis(handle_main(request_uri, relations).getvalue())

    return send_response(start_response, "text/html", "200 OK", doc.getvalue())


def handle_exception(
        environ: Dict[str, Any],
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """Displays an unhandled exception on the page."""
    status = '500 Internal Server Error'
    path_info = environ.get("PATH_INFO")
    if path_info:
        request_uri = path_info
    doc = yattag.Doc()
    write_html_header(doc)
    with doc.tag("pre"):
        doc.text(_("Internal error when serving {0}").format(request_uri) + "\n")
        doc.text(traceback.format_exc())
    return send_response(start_response, "text/html", status, doc.getvalue())


def application(
        environ: Dict[str, Any],
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """The entry point of this WSGI app."""
    try:
        return our_application(environ, start_response)

    # pylint: disable=broad-except
    except Exception:
        return handle_exception(environ, start_response)


def main() -> None:
    """Commandline interface to this module."""
    if sys.platform.startswith("win"):
        import _locale
        # pylint: disable=protected-access
        _locale._getdefaultlocale = (lambda *args: ['en_US', 'utf8'])

    httpd = wsgiref.simple_server.make_server('', 8000, application)
    print("Open <http://localhost:8000/osm> in your browser.")
    httpd.serve_forever()


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
