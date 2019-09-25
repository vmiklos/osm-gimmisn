#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

import configparser
import datetime
import locale
import os
import traceback
import urllib.parse
import json
import subprocess
import sys
from typing import Any
from typing import Callable
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
import wsgiref.simple_server

import pytz

import helpers
import overpass_query
import version
from i18n import translate as _

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_config() -> configparser.ConfigParser:
    """Gets access to information which are specific to this installation."""
    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)
    if not config.has_option("wsgi", "workdir"):
        workdir = os.path.join(os.path.dirname(__file__), "workdir")
        if not os.path.exists(workdir):
            os.makedirs(workdir)
        config.set("wsgi", "workdir", workdir)
    return config


def get_datadir() -> str:
    """Gets the directory which is tracked (in version control) data."""
    return os.path.join(os.path.dirname(__file__), "data")


def get_staticdir() -> str:
    """Gets the directory which is static data."""
    return os.path.join(os.path.dirname(__file__), "static")


def handle_streets(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/streets/ormezo/view-query."""
    output = ""

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    if action == "view-query":
        output += "<pre>"
        output += relation.get_osm_streets_query()
        output += "</pre>"
    elif action == "view-result":
        with relation.get_files().get_osm_streets_stream("r") as sock:
            table = helpers.tsv_to_list(sock)
            output += helpers.html_table_from_list(table)
    elif action == "update-result":
        query = relation.get_osm_streets_query()
        try:
            relation.get_files().write_osm_streets(overpass_query.overpass_query(query))
            streets = relation.get_config().should_check_missing_streets()
            if streets != "only":
                output += _("Update successful: ")
                output += gen_link("/osm/suspicious-streets/" + relation_name + "/view-result",
                                   _("View missing house numbers"))
            else:
                output += _("Update successful.")
        except urllib.error.HTTPError as http_error:
            output += _("Overpass error: {0}").format(str(http_error))

    osmrelation = relation.get_config().get_osmrelation()
    date = get_streets_last_modified(relation)
    return get_header(relations, "streets", relation_name, osmrelation) + output + get_footer(date)


def handle_street_housenumbers(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    output = ""

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    relation = relations.get_relation(relation_name)

    if action == "view-query":
        output += "<pre>"
        output += relation.get_osm_housenumbers_query()
        output += "</pre>"
    elif action == "view-result":
        with relation.get_files().get_osm_housenumbers_stream(mode="r") as sock:
            table = helpers.tsv_to_list(sock)
            output += helpers.html_table_from_list(table)
    elif action == "update-result":
        query = relation.get_osm_housenumbers_query()
        try:
            relation.get_files().write_osm_housenumbers(overpass_query.overpass_query(query))
            output += _("Update successful: ")
            output += gen_link("/osm/suspicious-streets/" + relation_name + "/view-result",
                               _("View missing house numbers"))
        except urllib.error.HTTPError as http_error:
            output += _("Overpass error: {0}").format(str(http_error))

    osmrelation = relation.get_config().get_osmrelation()
    date = get_housenumbers_last_modified(relation)
    return get_header(relations, "street-housenumbers", relation_name, osmrelation) + output + get_footer(date)


def gen_link(url: str, label: str) -> str:
    """Generates a link to a URL with a given label."""
    ret = '<a href="%s">' % url
    ret += label + "..."
    ret += "</a>"

    # Always auto-visit the link for now.
    ret += '<script type="text/javascript">window.location.href = "%s";</script>' % url

    return ret


def missing_housenumbers_view_res(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    output = ""
    relation = relations.get_relation(relation_name)
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets: ")
        output += gen_link("/osm/streets/" + relation_name + "/update-result",
                           _("Call Overpass to create"))
    elif not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
        output += _("No existing house numbers: ")
        output += gen_link("/osm/street-housenumbers/" + relation_name + "/update-result",
                           _("Call Overpass to create"))
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        output += _("No missing house numbers: ")
        output += gen_link("/osm/suspicious-streets/" + relation_name + "/update-result",
                           _("Create from reference"))
    else:
        ret = relation.write_missing_housenumbers()
        todo_street_count, todo_count, done_count, percent, table = ret

        output += "<p>"
        output += _("OpenStreetMap is possibly missing the below {0} house numbers for {1} streets.") \
            .format(str(todo_count), str(todo_street_count))
        output += _(" (existing: {0}, ready: {1}%).").format(str(done_count), str(percent)) + "<br>"
        output += "<a href=\"https://github.com/vmiklos/osm-gimmisn/tree/master/doc\">"
        output += _("Filter incorrect information")
        output += "</a>.</p>"

        output += helpers.html_table_from_list(table)
    return output


def missing_relations_view_result(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets: ")
        output += "<a href=\"/osm/streets/" + relation_name + "/update-result\">"
        output += _("Call Overpass to create")
        output += "</a>"
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        output += _("No street list: ")
        output += "<a href=\"/osm/suspicious-relations/" + relation_name + "/update-result\">"
        output += _("Create from reference")
        output += "</a>"
    else:
        ret = relation.write_missing_streets()
        todo_count, done_count, percent, streets = ret
        streets.sort(key=locale.strxfrm)
        table = [[_("Street name")]]
        for street in streets:
            table.append([street])

        output += "<p>"
        output += _("OpenStreetMap is possibly missing the below {0} streets.").format(str(todo_count))
        output += _(" (existing: {0}, ready: {1}%).").format(str(done_count), str(percent))
        output += "</p>"

        output += helpers.html_table_from_list(table)
    return output


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
                    elements = helpers.format_even_odd(result[1], html=False)
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


def missing_housenumbers_update(relations: helpers.Relations, relation_name: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/update-result."""
    reference = get_config().get('wsgi', 'reference_housenumbers').strip().split(' ')
    relation = relations.get_relation(relation_name)
    relation.write_ref_housenumbers(reference)
    output = _("Update successful: ")
    output += gen_link("/osm/suspicious-streets/" + relation_name + "/view-result",
                       _("View missing house numbers"))
    return output


def missing_streets_update(relations: helpers.Relations, relation_name: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/update-result."""
    reference = get_config().get('wsgi', 'reference_street').strip()
    relation = relations.get_relation(relation_name)
    relation.write_ref_streets(reference)
    return _("Update successful.")


def handle_missing_housenumbers(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-[result|query]."""
    output = ""

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    action_noext, _, ext = action.partition('.')
    date = None

    relation = relations.get_relation(relation_name)
    if action_noext == "view-result":
        if ext == "txt":
            return missing_housenumbers_view_txt(relations, request_uri)

        output += missing_housenumbers_view_res(relations, request_uri)
    elif action_noext == "view-query":
        output += "<pre>"
        with relation.get_files().get_ref_housenumbers_stream("r") as sock:
            output += sock.read()
        output += "</pre>"
        date = get_last_modified(relation.get_files().get_ref_housenumbers_path())
    elif action_noext == "update-result":
        output += missing_housenumbers_update(relations, relation_name)

    osmrelation = relation.get_config().get_osmrelation()
    if not date:
        date = ref_housenumbers_last_modified(relations, relation_name)
    return get_header(relations, "suspicious-streets", relation_name, osmrelation) + output + get_footer(date)


def handle_missing_streets(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-[result|query]."""
    output = ""

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    action_noext, _, ext = action.partition('.')
    relation = relations.get_relation(relation_name)

    if action_noext == "view-result":
        if ext == "txt":
            return missing_streets_view_txt(relations, request_uri)

        output += missing_relations_view_result(relations, request_uri)
    elif action_noext == "view-query":
        output += "<pre>"
        with relation.get_files().get_ref_streets_stream("r") as sock:
            output += sock.read()
        output += "</pre>"
    elif action_noext == "update-result":
        output += missing_streets_update(relations, relation_name)

    osmrelation = relation.get_config().get_osmrelation()
    date = ref_streets_last_modified(relation)
    return get_header(relations, "suspicious-relations", relation_name, osmrelation) + output + get_footer(date)


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


def handle_main_housenr_percent(relation: helpers.Relation) -> Tuple[str, str]:
    """Handles the house number percent part of the main page."""
    url = "\"/osm/suspicious-streets/" + relation.get_name() + "/view-result\""
    percent = "N/A"
    if os.path.exists(relation.get_files().get_housenumbers_percent_path()):
        percent = helpers.get_content(relation.get_files().get_housenumbers_percent_path())

    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_housenumbers_percent_path())
        cell = "<strong><a href=" + url + " title=\"" + _("updated") + " " + date + "\">"
        cell += percent + "%"
        cell += "</a></strong>"
        return cell, percent

    cell = "<strong><a href=" + url + ">"
    cell += _("missing house numbers")
    cell += "</a></strong>"
    return cell, "0"


def handle_main_street_percent(relation: helpers.Relation) -> Tuple[str, str]:
    """Handles the street percent part of the main page."""
    url = "\"/osm/suspicious-relations/" + relation.get_name() + "/view-result\""
    percent = "N/A"
    if os.path.exists(relation.get_files().get_streets_percent_path()):
        percent = helpers.get_content(relation.get_files().get_streets_percent_path())

    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_streets_percent_path())
        cell = "<strong><a href=" + url + " title=\"" + _("updated") + " " + date + "\">"
        cell += percent + "%"
        cell += "</a></strong>"
        return cell, percent

    cell = "<strong><a href=" + url + ">"
    cell += _("missing streets")
    cell += "</a></strong>"
    return cell, "0"


def filter_for_everything(_complete: bool, _refmegye: str) -> bool:
    """Does not filter out anything."""
    return True


def filter_for_incomplete(complete: bool, _refmegye: str) -> bool:
    """Filters out complete items."""
    return not complete


def create_filter_for_refmegye(refmegye_filter: str) -> Callable[[bool, str], bool]:
    """Creates a function that filters for a single refmegye."""
    return lambda _complete, refmegye: refmegye == refmegye_filter


def handle_main_filters(relations: helpers.Relations) -> str:
    """Handlers the filter part of the main wsgi page."""
    items = []
    items.append('<a href="/osm/filter-for/incomplete">' + _("Hide complete areas") + '</a>')
    # Sorted set of refmegye values of all relations.
    for refmegye in sorted({relation.get_config().get_refmegye() for relation in relations.get_relations()}):
        name = helpers.refmegye_get_name(refmegye)
        if not name:
            continue

        items.append('<a href="/osm/filter-for/refmegye/' + refmegye + '">' + name + '</a>')
    return '<p>' + _("Filters:") + " &brvbar; ".join(items) + '</p>'


def handle_main(request_uri: str, relations: helpers.Relations) -> str:
    """Handles the main wsgi page.

    Also handles /osm/filter-for/* which filters for a condition."""
    tokens = request_uri.split("/")
    filter_for = filter_for_everything  # type: Callable[[bool, str], bool]
    if len(tokens) >= 2 and tokens[-2] == "filter-for" and tokens[-1] == "incomplete":
        # /osm/filter-for/incomplete
        filter_for = filter_for_incomplete
    elif len(tokens) >= 3 and tokens[-3] == "filter-for" and tokens[-2] == "refmegye":
        # /osm/filter-for/refmegye/<value>.
        filter_for = create_filter_for_refmegye(tokens[-1])

    output = ""

    output += "<h1>" + _("Where to map?") + "</h1>"
    output += handle_main_filters(relations)
    table = []
    table.append([_("Area"),
                  _("House number coverage"),
                  _("Existing house numbers"),
                  _("Street coverage"),
                  _("Existing streets"),
                  _("Area boundary")])
    for relation_name in relations.get_names():
        relation = relations.get_relation(relation_name)
        complete = True

        streets = relation.get_config().should_check_missing_streets()

        row = []
        row.append(relation_name)

        if streets != "only":
            cell, percent = handle_main_housenr_percent(relation)
            row.append(cell)
            if float(percent) < 100.0:
                complete = False
        else:
            row.append("")

        if streets != "only":
            date = get_housenumbers_last_modified(relation)
            row.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/view-result\""
                       " title=\"" + _("updated") + " " + date + "\" >" + _("existing house numbers") + "</a>")
        else:
            row.append("")

        if streets != "no":
            cell, percent = handle_main_street_percent(relation)
            row.append(cell)
            if float(percent) < 100.0:
                complete = False
        else:
            row.append("")

        date = get_streets_last_modified(relation)
        row.append("<a href=\"/osm/streets/" + relation_name + "/view-result\""
                   " title=\"" + _("updated") + " " + date + "\" >" + _("existing streets") + "</a>")

        row.append("<a href=\"https://www.openstreetmap.org/relation/"
                   + str(relation.get_config().get_osmrelation())
                   + "\">" + _("area boundary") + "</a>")

        if filter_for(complete, relation.get_config().get_refmegye()):
            table.append(row)
    output += helpers.html_table_from_list(table)
    output += "<p><a href=\"https://github.com/vmiklos/osm-gimmisn/tree/master/doc\">"
    output += _("Add new area")
    output += "</a></p>"

    return get_header(relations) + output + get_footer()


def fill_missing_header_items(streets: str, relation_name: str, items: List[str]) -> None:
    """Generates the 'missing house numbers/streets' part of the header."""
    if streets != "only":
        suspicious = '<a href="/osm/suspicious-streets/' + relation_name + '/view-result">'
        suspicious += _("Missing house numbers") + '</a>'
        suspicious += ' (<a href="/osm/suspicious-streets/' + relation_name + '/view-result.txt">txt</a>)'
        items.append(suspicious)
        existing = "<a href=\"/osm/street-housenumbers/" + relation_name + "/view-result\">"
        existing += _("Existing house numbers") + "</a>"
        items.append(existing)
    if streets != "no":
        suspicious = '<a href="/osm/suspicious-relations/' + relation_name + '/view-result">'
        suspicious += _("Missing streets") + '</a>'
        suspicious += ' (<a href="/osm/suspicious-relations/' + relation_name + '/view-result.txt">txt</a>)'
        items.append(suspicious)


def get_header(
        relations: Optional[helpers.Relations] = None,
        function: str = "",
        relation_name: str = "",
        relation_osmid: int = 0
) -> str:
    """Produces the start of the page. Note that the content depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    title = ""
    items = []

    if relations and relation_name:
        relation = relations.get_relation(relation_name)
        streets = relation.get_config().should_check_missing_streets()

    items.append("<a href=\"/osm\">" + _("Area list") + "</a>")
    if relation_name:
        fill_missing_header_items(streets, relation_name, items)
        items.append("<a href=\"/osm/streets/" + relation_name + "/view-result\">" + _("Existing streets") + "</a>")

    if function == "suspicious-streets":
        title = " - " + _("{0} missing house numbers").format(relation_name)
        items.append("<a href=\"/osm/suspicious-streets/" + relation_name + "/update-result\">"
                     + _("Update from reference") + "</a> " + _("(may take seconds)"))
    elif function == "suspicious-relations":
        title = " - " + relation_name + " " + _("missing streets")
        items.append("<a href=\"/osm/suspicious-relations/" + relation_name + "/update-result\">"
                     + _("Update from reference") + "</a>")
    elif function == "street-housenumbers":
        title = " - " + relation_name + " " + _("existing house numbers")
        items.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/update-result\">"
                     + _("Call Overpass to update") + "</a> " + _("(may take seconds)"))
        items.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/view-query\">"
                     + _("View query") + "</a>")
    elif function == "streets":
        title = " - " + relation_name + " " + _("existing streets")
        items.append("<a href=\"/osm/streets/" + relation_name + "/update-result\">"
                     + _("Call Overpass to update") + "</a> " + _("(may take seconds)"))
        items.append("<a href=\"/osm/streets/" + relation_name + "/view-query\">" + _("View query") + "</a>")

    if relation_osmid:
        items.append("<a href=\"https://www.openstreetmap.org/relation/" + str(relation_osmid) + "\">"
                     + _("Area boundary") + "</a>")
    items.append("<a href=\"https://github.com/vmiklos/osm-gimmisn/tree/master/doc\">" + _("Documentation") + "</a>")

    config = get_config()
    if config.has_option("wsgi", "lang"):
        lang = config.get("wsgi", "lang")
    else:
        lang = "hu"
    output = '<!DOCTYPE html>\n<html lang="' + lang + '"><head><title>' + _("Where to map?") + title + '</title>'
    output += '<meta charset="UTF-8">'
    output += '<link rel="stylesheet" type="text/css" href="/osm/static/osm.css">'
    output += '<script src="/osm/static/sorttable.js"></script>'
    output += "</head><body><div>"
    output += " &brvbar; ".join(items)
    output += "</div><hr/>"
    return output


def get_footer(last_updated: str = "") -> str:
    """Produces the end of the page."""
    items = []
    items.append(_("Version: ") + helpers.git_link(version.VERSION, "https://github.com/vmiklos/osm-gimmisn/commit/"))
    items.append(_("OSM data © OpenStreetMap contributors."))
    if last_updated:
        items.append(_("Last update: ") + last_updated)
    output = "<hr/><div>"
    output += " &brvbar; ".join(items)
    output += "</div>"
    output += "</body></html>"
    return output


def handle_github_webhook(environ: Dict[str, Any]) -> str:
    """Handles a GitHub style webhook."""

    body = urllib.parse.parse_qs(environ["wsgi.input"].read().decode('utf-8'))
    payload = body["payload"][0]
    root = json.loads(payload)
    if root["ref"] == "refs/heads/master":
        subprocess.run(["make", "-C", version.GIT_DIR, "deploy-pythonanywhere"], check=True)

    return ""


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
    locale.setlocale(locale.LC_ALL, ui_locale)

    status = '200 OK'

    path_info = environ.get("PATH_INFO")
    if path_info:
        request_uri = path_info  # type: str
    _ignore, _ignore, ext = request_uri.partition('.')

    config = get_config()
    workdir = helpers.get_workdir(config)

    relations = helpers.Relations(get_datadir(), workdir)

    content_type = "text/html"
    if ext == "txt":
        content_type = "text/plain"

    if request_uri.startswith("/osm/streets/"):
        output = handle_streets(relations, request_uri)
    elif request_uri.startswith("/osm/suspicious-relations/"):
        output = handle_missing_streets(relations, request_uri)
    elif request_uri.startswith("/osm/street-housenumbers/"):
        output = handle_street_housenumbers(relations, request_uri)
    elif request_uri.startswith("/osm/suspicious-streets/"):
        output = handle_missing_housenumbers(relations, request_uri)
    elif request_uri.startswith("/osm/webhooks/github"):
        output = handle_github_webhook(environ)
    elif request_uri.startswith("/osm/static/"):
        output, content_type = handle_static(request_uri)
    else:
        output = handle_main(request_uri, relations)

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
    if path_info:
        request_uri = path_info
    body = "<pre>" + _("Internal error when serving {0}").format(request_uri) + "\n" + \
           traceback.format_exc() + "</pre>"
    output = get_header() + body + get_footer()
    output_bytes = output.encode('utf-8')
    response_headers = [('Content-type', 'text/html; charset=utf-8'),
                        ('Content-Length', str(len(output_bytes)))]
    start_response(status, response_headers)
    return [output_bytes]


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
