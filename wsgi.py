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
from typing import Dict
from typing import Callable
from typing import Iterable
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
import wsgiref.simple_server

import pytz

import helpers
import overpass_query
import version

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
            output += "Frissítés sikeres."
        except urllib.error.HTTPError as http_error:
            output += "Overpass hiba: " + str(http_error)

    osmrelation = relation.get_config().get_osmrelation()
    date = get_streets_last_modified(relations.get_workdir(), relation_name)
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
            output += "Frissítés sikeres."
        except urllib.error.HTTPError as http_error:
            output += "Overpass hiba: " + str(http_error)

    osmrelation = relation.get_config().get_osmrelation()
    date = get_housenumbers_last_modified(relations.get_workdir(), relation_name)
    return get_header(relations, "street-housenumbers", relation_name, osmrelation) + output + get_footer(date)


def suspicious_streets_view_result(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    output = ""
    relation = relations.get_relation(relation_name)
    if not os.path.exists(os.path.join(relations.get_workdir(), "streets-" + relation_name + ".csv")):
        output += "Nincsenek meglévő utcák: "
        output += "<a href=\"/osm/streets/" + relation_name + "/update-result\">"
        output += "Létrehozás Overpass hívásával</a>"
    elif not os.path.exists(os.path.join(relations.get_workdir(), "street-housenumbers-" + relation_name + ".csv")):
        output += "Nincsenek meglévő házszámok: "
        output += "<a href=\"/osm/street-housenumbers/" + relation_name + "/update-result\">"
        output += "Létrehozás Overpass hívásával</a>"
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        output += "Nincsenek hiányzó házszámok: "
        output += "<a href=\"/osm/suspicious-streets/" + relation_name + "/update-result\">"
        output += "Létrehozás referenciából</a>"
    else:
        ret = relation.write_missing_housenumbers()
        todo_street_count, todo_count, done_count, percent, table = ret

        output += "<p>Elképzelhető, hogy az OpenStreetMap nem tartalmazza a lenti "
        output += str(todo_street_count) + " utcához tartozó "
        output += str(todo_count) + " házszámot."
        output += " (meglévő: " + str(done_count) + ", készültség: " + str(percent) + "%).<br>"
        output += "<a href=\"" + \
                  "https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu" + \
                  "#hib%C3%A1s-riaszt%C3%A1s-hozz%C3%A1ad%C3%A1sa\">" + \
                  "Téves információ jelentése</a>.</p>"

        output += helpers.html_table_from_list(table)
    return output


def missing_relations_view_result(relations: helpers.Relations, request_uri: str, workdir: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(os.path.join(workdir, "streets-" + relation_name + ".csv")):
        output += "Nincsenek meglévő utcák: "
        output += "<a href=\"/osm/streets/" + relation_name + "/update-result\">"
        output += "Létrehozás Overpass hívásával</a>"
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        output += "Nincsen utcalista: "
        output += "<a href=\"/osm/suspicious-relations/" + relation_name + "/update-result\">"
        output += "Létrehozás referenciából</a>"
    else:
        ret = relation.write_missing_streets()
        todo_count, done_count, percent, streets = ret
        streets.sort(key=locale.strxfrm)
        table = [["Utcanév"]]
        for street in streets:
            table.append([street])

        output += "<p>Elképzelhető, hogy az OpenStreetMap nem tartalmazza a lenti "
        output += str(todo_count) + " utcát."
        output += " (meglévő: " + str(done_count) + ", készültség: " + str(percent) + "%).<br>"

        output += helpers.html_table_from_list(table)
    return output


def suspicious_streets_view_txt(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(os.path.join(relations.get_workdir(), "streets-" + relation_name + ".csv")):
        output += "Nincsenek meglévő utcák"
    elif not os.path.exists(os.path.join(relations.get_workdir(), "street-housenumbers-" + relation_name + ".csv")):
        output += "Nincsenek meglévő házszámok"
    elif not os.path.exists(relations.get_relation(relation_name).get_files().get_ref_housenumbers_path()):
        output += "Nincsenek referencia házszámok"
    else:
        ongoing_streets, _ = relation.get_missing_housenumbers()

        table = []
        for result in ongoing_streets:
            if result[1]:
                # Street name, only_in_reference items.
                if not relation.get_config().get_street_is_even_odd(result[0]):
                    row = result[0] + "\t[" + ", ".join(result[1]) + "]"
                else:
                    elements = helpers.format_even_odd(result[1])
                    row = result[0] + "\t[" + "], [".join(elements) + "]"
                table.append(row)
        table.sort(key=locale.strxfrm)
        output += "\n".join(table)
    return output


def suspicious_relations_view_txt(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(os.path.join(relations.get_workdir(), "streets-" + relation_name + ".csv")):
        output += "Nincsenek meglévő utcák"
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        output += "Nincsenek referencia utcák"
    else:
        todo_streets, _ = relation.get_missing_streets()
        todo_streets.sort(key=locale.strxfrm)
        output += "\n".join(todo_streets)
    return output


def suspicious_streets_update(relations: helpers.Relations, relation_name: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/update-result."""
    reference = get_config().get('wsgi', 'reference_local').strip()
    relation = relations.get_relation(relation_name)
    relation.write_ref_housenumbers(reference)
    return "Frissítés sikeres."


def suspicious_relations_update(relations: helpers.Relations, relation_name: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/update-result."""
    reference = get_config().get('wsgi', 'reference_street').strip()
    relation = relations.get_relation(relation_name)
    relation.write_ref_streets(reference)
    return "Frissítés sikeres."


def handle_suspicious_streets(relations: helpers.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-streets/ormezo/view-[result|query]."""
    output = ""

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    action_noext, _, ext = action.partition('.')

    relation = relations.get_relation(relation_name)
    if action_noext == "view-result":
        if ext == "txt":
            return suspicious_streets_view_txt(relations, request_uri)

        output += suspicious_streets_view_result(relations, request_uri)
    elif action_noext == "view-query":
        output += "<pre>"
        with relation.get_files().get_ref_housenumbers_stream("r") as sock:
            output += sock.read()
        output += "</pre>"
    elif action_noext == "update-result":
        output += suspicious_streets_update(relations, relation_name)

    osmrelation = relation.get_config().get_osmrelation()
    date = ref_housenumbers_last_modified(relations, relation_name)
    return get_header(relations, "suspicious-streets", relation_name, osmrelation) + output + get_footer(date)


def handle_suspicious_relations(relations: helpers.Relations, request_uri: str, workdir: str) -> str:
    """Expected request_uri: e.g. /osm/suspicious-relations/ujbuda/view-[result|query]."""
    output = ""

    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    action_noext, _, ext = action.partition('.')
    relation = relations.get_relation(relation_name)

    if action_noext == "view-result":
        if ext == "txt":
            return suspicious_relations_view_txt(relations, request_uri)

        output += missing_relations_view_result(relations, request_uri, workdir)
    elif action_noext == "view-query":
        output += "<pre>"
        with relation.get_files().get_ref_streets_stream("r") as sock:
            output += sock.read()
        output += "</pre>"
    elif action_noext == "update-result":
        output += suspicious_relations_update(relations, relation_name)

    osmrelation = relation.get_config().get_osmrelation()
    date = ref_streets_last_modified(relation, workdir, relation_name)
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
    t_ref = get_timestamp(relations.get_relation(name).get_files().get_ref_housenumbers_path())
    t_housenumbers = get_timestamp(relations.get_workdir(), "street-housenumbers-" + name + ".csv")
    return format_timestamp(max(t_ref, t_housenumbers))


def ref_streets_last_modified(relation: helpers.Relation, workdir: str, name: str) -> str:
    """Gets the update date for missing streets."""
    t_ref = get_timestamp(relation.get_files().get_ref_streets_path())
    t_osm = get_timestamp(workdir, "streets-" + name + ".csv")
    return format_timestamp(max(t_ref, t_osm))


def get_housenumbers_last_modified(workdir: str, name: str) -> str:
    """Gets the update date of house numbers for a relation."""
    return get_last_modified(workdir, "street-housenumbers-" + name + ".csv")


def get_streets_last_modified(workdir: str, name: str) -> str:
    """Gets the update date of streets for a relation."""
    return get_last_modified(workdir, "streets-" + name + ".csv")


def handle_main_housenr_percent(relation: helpers.Relation) -> Tuple[str, str]:
    """Handles the house number percent part of the main page."""
    url = "\"/osm/suspicious-streets/" + relation.get_name() + "/view-result\""
    percent = "N/A"
    if os.path.exists(relation.get_files().get_housenumbers_percent_path()):
        percent = helpers.get_content(relation.get_files().get_housenumbers_percent_path())

    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_housenumbers_percent_path())
        cell = "<strong><a href=" + url + " title=\"frissítve " + date + "\">"
        cell += percent + "%"
        cell += "</a></strong>"
        return cell, percent

    cell = "<strong><a href=" + url + ">"
    cell += "hiányzó házszámok"
    cell += "</a></strong>"
    return cell, "0"


def handle_main_street_percent(workdir: str, relation_name: str) -> Tuple[str, str]:
    """Handles the street percent part of the main page."""
    percent_file = relation_name + "-streets.percent"
    url = "\"/osm/suspicious-relations/" + relation_name + "/view-result\""
    percent = "N/A"
    if os.path.exists(os.path.join(workdir, percent_file)):
        percent = helpers.get_content(workdir, percent_file)

    if percent != "N/A":
        date = get_last_modified(workdir, percent_file)
        cell = "<strong><a href=" + url + " title=\"frissítve " + date + "\">"
        cell += percent + "%"
        cell += "</a></strong>"
        return cell, percent

    cell = "<strong><a href=" + url + ">"
    cell += "hiányzó utcák"
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
    items.append('<a href="/osm/filter-for/incomplete">Kész területek elrejtése</a>')
    # Sorted set of refmegye values of all relations.
    for refmegye in sorted({relation.get_config().get_refmegye() for relation in relations.get_relations()}):
        name = helpers.refmegye_get_name(refmegye)
        if not name:
            continue

        items.append('<a href="/osm/filter-for/refmegye/' + refmegye + '">' + name + '</a>')
    return '<p>Szűrők: ' + " &brvbar; ".join(items) + '</p>'


def handle_main(request_uri: str, relations: helpers.Relations, workdir: str) -> str:
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

    output += "<h1>Hol térképezzek?</h1>"
    output += handle_main_filters(relations)
    table = []
    table.append(["Terület",
                  "Házszám lefedettség",
                  "Meglévő házszámok",
                  "Utca lefedettség",
                  "Meglévő utcák",
                  "Terület határa"])
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
            date = get_housenumbers_last_modified(workdir, relation_name)
            row.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/view-result\""
                       " title=\"frissítve " + date + "\" >meglévő házszámok</a>")
        else:
            row.append("")

        if streets != "no":
            cell, percent = handle_main_street_percent(workdir, relation_name)
            row.append(cell)
            if float(percent) < 100.0:
                complete = False
        else:
            row.append("")

        date = get_streets_last_modified(workdir, relation_name)
        row.append("<a href=\"/osm/streets/" + relation_name + "/view-result\""
                   " title=\"frissítve " + date + "\" >meglévő utcák</a>")

        row.append("<a href=\"https://www.openstreetmap.org/relation/"
                   + str(relation.get_config().get_osmrelation())
                   + "\">terület határa</a>")

        if filter_for(complete, relation.get_config().get_refmegye()):
            table.append(row)
    output += helpers.html_table_from_list(table)
    output += "<p><a href=\"" + \
              "https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu" + \
              "#%C3%BAj-rel%C3%A1ci%C3%B3-hozz%C3%A1ad%C3%A1sa\">" + \
              "Új terület hozzáadása</a></p>"

    return get_header(relations) + output + get_footer()


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

    items.append("<a href=\"/osm\">Területek listája</a>")
    if relation_name:
        if streets != "only":
            suspicious = '<a href="/osm/suspicious-streets/' + relation_name + '/view-result">Hiányzó házszámok</a>'
            suspicious += ' (<a href="/osm/suspicious-streets/' + relation_name + '/view-result.txt">txt</a>)'
            items.append(suspicious)
            items.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/view-result\">Meglévő házszámok</a>")
        if streets != "no":
            suspicious = '<a href="/osm/suspicious-relations/' + relation_name + '/view-result">Hiányzó utcák</a>'
            suspicious += ' (<a href="/osm/suspicious-relations/' + relation_name + '/view-result.txt">txt</a>)'
            items.append(suspicious)
        items.append("<a href=\"/osm/streets/" + relation_name + "/view-result\">Meglévő utcák</a>")

    if function == "suspicious-streets":
        title = " - " + relation_name + " hiányzó házszámok"
        items.append("<a href=\"/osm/suspicious-streets/" + relation_name + "/update-result\">"
                     + "Frissítés referenciából</a> (másodpercekig tarthat)")
    elif function == "suspicious-relations":
        title = " - " + relation_name + " hiányzó utcák"
        items.append("<a href=\"/osm/suspicious-relations/" + relation_name + "/update-result\">"
                     + "Frissítés referenciából</a>")
    elif function == "street-housenumbers":
        title = " - " + relation_name + " meglévő házszámok"
        items.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/update-result\">"
                     + "Frissítés Overpass hívásával</a> (másodpercekig tarthat)")
        items.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/view-query\">"
                     + "Lekérdezés megtekintése</a>")
    elif function == "streets":
        title = " - " + relation_name + " meglévő utcák"
        items.append("<a href=\"/osm/streets/" + relation_name + "/update-result\">"
                     + "Frissítés Overpass hívásával</a> (másodpercekig tarthat)")
        items.append("<a href=\"/osm/streets/" + relation_name + "/view-query\">Lekérdezés megtekintése</a>")

    if relation_osmid:
        items.append("<a href=\"https://www.openstreetmap.org/relation/" + str(relation_osmid) + "\">"
                     + "Terület határa</a>")
    items.append("<a href=\"https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu\">Dokumentáció</a>")

    config = get_config()
    if config.has_option("wsgi", "lang"):
        lang = config.get("wsgi", "lang")
    else:
        lang = "hu"
    output = '<!DOCTYPE html>\n<html lang="' + lang + '"><head><title>Hol térképezzek?' + title + '</title>'
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
    items.append("Verzió: " + helpers.git_link(version.VERSION, "https://github.com/vmiklos/osm-gimmisn/commit/"))
    items.append("OSM adatok © OpenStreetMap közreműködők.")
    if last_updated:
        items.append("Utolsó frissítés: " + last_updated)
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
    _, _, ext = request_uri.partition('.')

    config = get_config()
    workdir = helpers.get_workdir(config)

    relations = helpers.Relations(get_datadir(), workdir)

    content_type = "text/html"
    if ext == "txt":
        content_type = "text/plain"

    if request_uri.startswith("/osm/streets/"):
        output = handle_streets(relations, request_uri)
    elif request_uri.startswith("/osm/suspicious-relations/"):
        output = handle_suspicious_relations(relations, request_uri, workdir)
    elif request_uri.startswith("/osm/street-housenumbers/"):
        output = handle_street_housenumbers(relations, request_uri)
    elif request_uri.startswith("/osm/suspicious-streets/"):
        output = handle_suspicious_streets(relations, request_uri)
    elif request_uri.startswith("/osm/webhooks/github"):
        output = handle_github_webhook(environ)
    elif request_uri.startswith("/osm/static/"):
        output, content_type = handle_static(request_uri)
    else:
        output = handle_main(request_uri, relations, workdir)

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
    body = "<pre>Internal error when serving " + request_uri + "\n" + \
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
