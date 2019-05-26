#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

import configparser
import datetime
import os
import traceback
import yaml
import pytz
import helpers
import overpass_query
import suspicious_streets
import get_reference_housenumbers
import version


def getConfig():
    """Gets access to information which are specific to this installation."""
    config = configparser.ConfigParser()
    configPath = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(configPath)
    return config


def getDatadir():
    """Gets the directory which is tracked (in version control) data."""
    return os.path.join(os.path.dirname(__file__), "data")


def getStreetsQuery(relations, relation):
    """Produces a query which lists streets in relation."""
    with open(os.path.join(getDatadir(), "streets-template.txt")) as sock:
        return helpers.process_template(sock.read(), relations[relation]["osmrelation"])


def getStreetHousenumbersQuery(relations, relation):
    """Produces a query which lists house numbers in relation."""
    with open(os.path.join(getDatadir(), "street-housenumbers-template.txt")) as sock:
        return helpers.process_template(sock.read(), relations[relation]["osmrelation"])


def getRelations():
    """Returns a name -> properties dictionary."""
    return yaml.load(open(os.path.join(getDatadir(), "relations.yaml")))


def handleStreets(requestUri, workdir, relations):
    """Expected requestUri: e.g. /osm/streets/ormezo/view-query."""
    output = ""

    tokens = requestUri.split("/")
    relation = tokens[-2]
    action = tokens[-1]

    if action == "view-query":
        output += "<pre>"
        output += getStreetsQuery(relations, relation)
        output += "</pre>"
    elif action == "view-result":
        output += "<pre>"
        with open(os.path.join(workdir, "streets-%s.csv" % relation)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "update-result":
        query = getStreetsQuery(relations, relation)
        result = helpers.sort_streets_csv(overpass_query.overpassQuery(query))
        with open(os.path.join(workdir, "streets-%s.csv" % relation), mode="w") as sock:
            sock.write(result)
            output += "Frissítés sikeres."

    osmrelation = relations[relation]["osmrelation"]
    date = get_streets_last_modified(workdir, relation)
    return getHeader("streets", relation, osmrelation) + output + getFooter(date)


def handleStreetHousenumbers(requestUri, workdir, relations):
    """Expected requestUri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    output = ""

    tokens = requestUri.split("/")
    relation = tokens[-2]
    action = tokens[-1]

    if action == "view-query":
        output += "<pre>"
        output += getStreetHousenumbersQuery(relations, relation)
        output += "</pre>"
    elif action == "view-result":
        output += "<pre>"
        with open(os.path.join(workdir, "street-housenumbers-%s.csv" % relation)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "update-result":
        query = getStreetHousenumbersQuery(relations, relation)
        result = helpers.sort_housenumbers_csv(overpass_query.overpassQuery(query))
        with open(os.path.join(workdir, "street-housenumbers-%s.csv" % relation), mode="w") as sock:
            sock.write(result)
            output += "Frissítés sikeres."

    osmrelation = relations[relation]["osmrelation"]
    date = get_housenumbers_last_modified(workdir, relation)
    return getHeader("street-housenumbers", relation, osmrelation) + output + getFooter(date)


def handleSuspiciousStreets(requestUri, workdir, relations):
    """Expected requestUri: e.g. /osm/suspicious-streets/ormezo/view-[result|query]."""
    output = ""

    tokens = requestUri.split("/")
    relation = tokens[-2]
    action = tokens[-1]

    if action == "view-result":
        output += "<pre>"
        finder = suspicious_streets.Finder(getDatadir(), workdir, relation)
        houseNrCount = 0
        for result in finder.suspiciousStreets:
            if result[1]:
                # House number, # of onlyInReference items.
                output += "%s\t%s\n" % (result[0], len(result[1]))
                # onlyInReference items.
                output += str(result[1]) + "\n"
                houseNrCount += len(result[1])
        doneNrCount = 0
        for result in finder.doneStreets:
            doneNrCount += len(result[1])
        output += "</pre>"
        output += "Elképzelhető, hogy az OpenStreetMap nem tartalmazza a fenti "
        output += str(len(finder.suspiciousStreets)) + " utcához tartozó "
        output += str(houseNrCount) + " házszámot."
        if doneNrCount > 0 or houseNrCount > 0:
            percent = "%.2f" % (doneNrCount / (doneNrCount + houseNrCount) * 100)
        else:
            percent = "N/A"
        output += " (meglévő: " + str(doneNrCount) + ", készültség: " + str(percent) + "%).<br>"
        output += "<a href=\"" + \
                  "https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu" + \
                  "#hib%C3%A1s-riaszt%C3%A1s-hozz%C3%A1ad%C3%A1sa\">" + \
                  "Téves információ jelentése</a>."

        # Write the bottom line to a file, so the index page show it fast.
        with open(os.path.join(workdir, relation + ".percent"), "w") as sock:
            sock.write(percent)
    elif action == "view-query":
        output += "<pre>"
        path = "street-housenumbers-reference-%s.lst" % relation
        with open(os.path.join(workdir, path)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "update-result":
        get_reference_housenumbers.getReferenceHousenumbers(getConfig(), relation)
        output += "Frissítés sikeres."

    osmrelation = relations[relation]["osmrelation"]
    date = get_ref_housenumbers_last_modified(workdir, relation)
    return getHeader("suspicious-streets", relation, osmrelation) + output + getFooter(date)


def local_to_ui_tz(localDt):
    """Converts from local date-time to UI date-time, based on config."""
    config = getConfig()
    if config.has_option("wsgi", "timezone"):
        uiTz = pytz.timezone(config.get("wsgi", "timezone"))
    else:
        uiTz = pytz.timezone("Europe/Budapest")

    return localDt.astimezone(uiTz)


def getLastModified(workdir, path):
    """Gets the update date of a file in workdir."""
    return format_timestamp(get_timestamp(workdir, path))


def get_timestamp(workdir, path):
    """Gets the timestamp of a file in workdir."""
    return os.path.getmtime(os.path.join(workdir, path))


def format_timestamp(t):
    """Formats timestamp as UI date-time."""
    localDt = datetime.datetime.fromtimestamp(t)
    uiDt = local_to_ui_tz(localDt)
    fmt = '%Y-%m-%d %H:%M'
    return uiDt.strftime(fmt)


def get_ref_housenumbers_last_modified(workdir, name):
    """Gets the update date for suspicious streets."""
    tRef = get_timestamp(workdir, "street-housenumbers-reference-" + name + ".lst")
    tHousenumbers = get_timestamp(workdir, "street-housenumbers-" + name + ".csv")
    return format_timestamp(max(tRef, tHousenumbers))


def get_housenumbers_last_modified(workdir, name):
    """Gets the update date of house numbers for a relation."""
    return getLastModified(workdir, "street-housenumbers-" + name + ".csv")


def get_streets_last_modified(workdir, name):
    """Gets the update date of streets for a relation."""
    return getLastModified(workdir, "streets-" + name + ".csv")


def handleMain(relations, workdir):
    """Handles the main wsgi page."""
    output = ""

    output += "<h1>Hol térképezzek?</h1>"
    output += "<table>"
    for k in sorted(relations):
        v = relations[k]
        output += "<tr>"
        output += "<td>" + k + "</td>"
        percentFile = k + ".percent"
        url = "\"/osm/suspicious-streets/" + k + "/view-result\""
        percent = "N/A"
        if os.path.exists(os.path.join(workdir, percentFile)):
            percent = helpers.get_content(workdir, percentFile)

        if percent != "N/A":
            date = getLastModified(workdir, percentFile)
            output += "<td><strong><a href=" + url + " title=\"frissítve " + date + "\">"
            output += percent + "% kész"
            output += "</a></strong></td>"
        else:
            output += "<td><strong><a href=" + url + ">"
            output += "hiányzó házszámok"
            output += "</a></strong></td>"

        date = get_housenumbers_last_modified(workdir, k)
        output += "<td><a href=\"/osm/street-housenumbers/" + k + "/view-result\"" \
                  "title=\"frissítve " + date + "\" >meglévő házszámok</a></td>"

        date = get_streets_last_modified(workdir, k)
        output += "<td><a href=\"/osm/streets/" + k + "/view-result\"" \
                  "title=\"frissítve " + date + "\" >meglévő utcák</a></td>"

        output += "<td><a href=\"https://www.openstreetmap.org/relation/" + str(v["osmrelation"]) + \
                  "\">terület határa</a></td>"

        output += "</tr>"
    output += "</table>"
    output += "<a href=\"" + \
              "https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu" + \
              "#%C3%BAj-rel%C3%A1ci%C3%B3-hozz%C3%A1ad%C3%A1sa\">" + \
              "Új terület hozzáadása</a>."

    return getHeader() + output + getFooter()


def getHeader(function=None, relation_name=None, relation_osmid=None):
    """Produces the start of the page. Note that the contnt depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    title = ""
    items = []

    items.append("<a href=\"/osm\">Területek listája</a>")
    if relation_name:
        items.append("<a href=\"/osm/suspicious-streets/" + relation_name + "/view-result\">Hiányzó házszámok</a>")
        items.append("<a href=\"/osm/street-housenumbers/" + relation_name + "/view-result\">Meglévő házszámok</a>")
        items.append("<a href=\"/osm/streets/" + relation_name + "/view-result\">Meglévő utcák</a>")

    if function == "suspicious-streets":
        title = " - " + relation_name + " hiányzó házszámok"
        items.append("<a href=\"/osm/suspicious-streets/" + relation_name + "/update-result\">"
                     + "Frissítés referenciából</a> (másodpercekig tarthat)")
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

    output = "<html><head><title>Hol térképezzek?" + title + "</title></head><body><div>"
    output += " &brvbar; ".join(items)
    output += "</div><hr/>"
    return output


def getFooter(last_updated=None):
    """Produces the end of the page."""
    items = []
    items.append("Verzió: " + helpers.git_link(version.version, "https://github.com/vmiklos/osm-gimmisn/commit/"))
    items.append("OSM adatok © OpenStreetMap közreműködők.")
    if last_updated:
        items.append("Utolsó frissítés: " + last_updated)
    output = "<hr/><div>"
    output += " &brvbar; ".join(items)
    output += "</div>"
    output += "</body></html>"
    return output


def our_application(environ, start_response):
    """Dispatches the request based on its URI."""
    status = '200 OK'

    requestUri = environ.get("REQUEST_URI")

    config = getConfig()

    workdir = helpers.get_workdir(config)

    relations = getRelations()

    if requestUri.startswith("/osm/streets/"):
        output = handleStreets(requestUri, workdir, relations)
    elif requestUri.startswith("/osm/street-housenumbers/"):
        output = handleStreetHousenumbers(requestUri, workdir, relations)
    elif requestUri.startswith("/osm/suspicious-streets/"):
        output = handleSuspiciousStreets(requestUri, workdir, relations)
    else:
        output = handleMain(relations, workdir)

    outputBytes = output.encode('utf-8')
    response_headers = [('Content-type', 'text/html; charset=utf-8'),
                        ('Content-Length', str(len(outputBytes)))]
    start_response(status, response_headers)
    return [outputBytes]


def handle_exception(environ, start_response):
    """Displays an unhandled exception on the page."""
    status = '500 Internal Server Error'
    requestUri = environ.get("REQUEST_URI")
    body = "<pre>Internal error when serving " + requestUri + "\n" + \
           traceback.format_exc() + "</pre>"
    output = getHeader() + body + getFooter()
    outputBytes = output.encode('utf-8')
    response_headers = [('Content-type', 'text/html; charset=utf-8'),
                        ('Content-Length', str(len(outputBytes)))]
    start_response(status, response_headers)
    return [outputBytes]


def application(environ, start_response):
    """The entry point of this WSGI app."""
    try:
        return our_application(environ, start_response)

    # pylint: disable=broad-except
    except Exception:
        return handle_exception(environ, start_response)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
