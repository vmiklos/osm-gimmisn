#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

import configparser
import datetime
import os
import traceback
import yaml
import helpers
import overpass_query
import suspicious_streets
import get_reference_housenumbers


def getConfig():
    config = configparser.ConfigParser()
    configPath = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(configPath)
    return config


def getWorkdir():
    return getConfig().get('wsgi', 'workdir').strip()


def getReference():
    return getConfig().get('wsgi', 'reference').strip()


def getDatadir():
    return os.path.join(os.path.dirname(__file__), "data")


def getStreetsQuery(relations, relation):
    with open(os.path.join(getDatadir(), "streets-template.txt")) as sock:
        return sock.read().replace("@RELATION@", str(relations[relation]["osmrelation"]))


def getStreetHousenumbersQuery(relations, relation):
    with open(os.path.join(getDatadir(), "street-housenumbers-template.txt")) as sock:
        return sock.read().replace("@RELATION@", str(relations[relation]["osmrelation"]))


# Returns a name -> properties dictionary.
def getRelations():
    return yaml.load(open(os.path.join(getDatadir(), "relations.yaml")))


# Expected requestUri: e.g. /osm/streets/ormezo/view-query
def handleStreets(requestUri, workdir, relations):
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

    date = get_streets_last_modified(workdir, relation)
    title = " - " + relation + " meglévő utcák"
    links = "Frissítve " + date + "&brvbar; " \
            "<a href=\"/osm/streets/" + relation + "/view-result\">" + \
            "Meglévő utcák</a> &brvbar; " + \
            "<a href=\"/osm/streets/" + relation + "/update-result\">" + \
            "Frissítés Overpass hívásával</a> (másodpercekig tarthat) &brvbar; " + \
            "<a href=\"/osm/streets/" + relation + "/view-query\">" + \
            "Lekérdezés megtekintése</a> &brvbar; "
    return getHeader(add_title=title, add_links=links) + output + getFooter()


# Expected requestUri: e.g. /osm/street-housenumbers/ormezo/view-query
def handleStreetHousenumbers(requestUri, workdir, relations):
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

    date = get_housenumbers_last_modified(workdir, relation)
    title = " - " + relation + " meglévő házszámok"
    links = "Frissítve " + date + "&brvbar; " + \
            "<a href=\"/osm/street-housenumbers/" + relation + "/view-result\">" + \
            "Meglévő házszámok</a> &brvbar; " + \
            "<a href=\"/osm/street-housenumbers/" + relation + "/update-result\">" + \
            "Frissítés Overpass hívásával</a> (másodpercekig tarthat) &brvbar; " + \
            "<a href=\"/osm/street-housenumbers/" + relation + "/view-query\">" + \
            "Lekérdezés megtekintése</a> &brvbar; "
    return getHeader(add_title=title, add_links=links) + output + getFooter()


# Expected requestUri: e.g. /osm/suspicious-streets/ormezo/view-[result|query].
def handleSuspiciousStreets(requestUri, workdir, relations):
    output = ""

    tokens = requestUri.split("/")
    relation = tokens[-2]
    action = tokens[-1]

    if action == "view-result":
        output += "<pre>"
        # TODO this API is far from nice
        cwd = os.getcwd()
        suspicious_streets.suffix = "-%s" % relation
        suspicious_streets.loadNormalizers()
        os.chdir(workdir)
        finder = suspicious_streets.Finder()
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
        os.chdir(cwd)
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
        date = getLastModified(workdir, path)
        output += "<div>updated on " + date + "</div>"
    elif action == "update-result":
        get_reference_housenumbers.getReferenceHousenumbers(getConfig(), relation)
        output += "Frissítés sikeres."

    date = get_ref_housenumbers_last_modified(workdir, relation)
    output += "<p>Frissítve " + date + "</p>"

    title = " - " + relation + " hiányzó házszámok"
    osmurl = "https://www.openstreetmap.org/relation/" + str(relations[relation]["osmrelation"])
    links = "<a href=\"/osm/street-housenumbers/" + relation + "/view-result\">" + \
            "Meglévő házszámok a környéken</a> &brvbar; " + \
            "<a href=\"/osm/streets/" + relation + "/view-result\">" + \
            "Meglévő utcák a környéken</a> &brvbar; " + \
            "<a href=\"/osm/suspicious-streets/" + relation + "/update-result\">" + \
            "Frissítés referenciából</a> (másodpercekig tarthat) &brvbar; " + \
            "<a href=\"" + osmurl + "\">terület határa</a> &brvbar; "
    return getHeader(add_title=title, add_links=links) + output + getFooter()


def getLastModified(workdir, path):
    t = os.path.getmtime(os.path.join(workdir, path))
    return datetime.datetime.fromtimestamp(t).isoformat()


def get_ref_housenumbers_last_modified(workdir, name):
    return getLastModified(workdir, "street-housenumbers-reference-" + name + ".lst")


def get_housenumbers_last_modified(workdir, name):
    return getLastModified(workdir, "street-housenumbers-" + name + ".csv")


def get_streets_last_modified(workdir, name):
    return getLastModified(workdir, "streets-" + name + ".csv")


def getContent(workdir, path):
    ret = ""
    with open(os.path.join(workdir, path)) as sock:
        ret = sock.read()
    return ret


def handleMain(relations, workdir):
    output = ""

    output += "<h1>Hol térképezzek?</h1>"
    output += "<table>"
    for k, v in relations.items():
        output += "<tr>"
        output += "<td>" + k + "</td>"
        percentFile = k + ".percent"
        url = "\"/osm/suspicious-streets/" + k + "/view-result\""
        percent = "N/A"
        if os.path.exists(os.path.join(workdir, percentFile)):
            percent = getContent(workdir, percentFile)

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
              "https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu#az-osm-gimmisn-haszn%C3%A1lata\">" + \
              "Segítség a használathoz</a> &brvbar; "
    output += "<a href=\"" + \
              "https://github.com/vmiklos/osm-gimmisn/tree/master/doc/hu" + \
              "#%C3%BAj-rel%C3%A1ci%C3%B3-hozz%C3%A1ad%C3%A1sa\">" + \
              "Új terület hozzáadása</a>."

    return getHeader() + output + getFooter()


def getHeader(add_title='', add_links=''):
    output = "<html><head><title>Hol térképezzek?" + add_title + "</title></head><body>"
    output += "<div><a href=\"/osm\">Területek listája</a> &brvbar; " + add_links + \
              "<a href=\"https://github.com/vmiklos/osm-gimmisn\">github</a></div><hr/>"
    return output


def getFooter():
    output = "<hr/><div>OSM adatok © OpenStreetMap közreműködők.</div>"
    output += "</body></html>"
    return output


def our_application(environ, start_response):
    status = '200 OK'

    requestUri = environ.get("REQUEST_URI")

    workdir = getWorkdir()

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
    try:
        return our_application(environ, start_response)

    # pylint: disable=broad-except
    except Exception:
        return handle_exception(environ, start_response)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
