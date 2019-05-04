#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

import configparser
import datetime
import os
import overpass_query
import suspicious_streets
import yaml


def getWorkdir():
    config = configparser.ConfigParser()
    configPath = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(configPath)
    return config.get('wsgi', 'workdir').strip()


# Returns a name -> relation ID dictionary.
# Expected YAML format (without quotes):
#
# "ormezo: 2713749
# terezvaros: 3229919"
def getRelations():
    return yaml.load(open(os.path.join(os.path.dirname(__file__), "relations.yaml")))


# Expected requestUri: e.g. /osm/streets/ormezo/view-query
def handleStreets(requestUri, workdir):
    output = ""

    tokens = requestUri.split("/")
    relation = tokens[-2]
    action = tokens[-1]

    if action == "view-query":
        output += "<pre>"
        with open(os.path.join(workdir, "streets-%s.txt" % relation)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "view-result":
        output += "<pre>"
        with open(os.path.join(workdir, "streets-%s.csv" % relation)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "update-result":
        with open(os.path.join(workdir, "streets-%s.txt" % relation)) as sock:
            query = sock.read()
        result = overpass_query.overpassQuery(query)
        with open(os.path.join(workdir, "streets-%s.csv" % relation), mode="w") as sock:
            sock.write(result)
            output += "update finished. <a href=\"/osm/streets/" + relation + "/view-result\">view</a>"

    return getHeader() + output + getFooter()


# Expected requestUri: e.g. /osm/street-housenumbers/ormezo/view-query
def handleStreetHousenumbers(requestUri, workdir):
    output = ""

    tokens = requestUri.split("/")
    relation = tokens[-2]
    action = tokens[-1]

    if action == "view-query":
        output += "<pre>"
        with open(os.path.join(workdir, "street-housenumbers-%s.txt" % relation)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "view-result":
        output += "<pre>"
        with open(os.path.join(workdir, "street-housenumbers-%s.csv" % relation)) as sock:
            output += sock.read()
        output += "</pre>"
    elif action == "update-result":
        with open(os.path.join(workdir, "street-housenumbers-%s.txt" % relation)) as sock:
            query = sock.read()
        result = overpass_query.overpassQuery(query)
        with open(os.path.join(workdir, "street-housenumbers-%s.csv" % relation), mode="w") as sock:
            sock.write(result)
            output += "update finished. <a href=\"/osm/street-housenumbers/" + relation + "/view-result\">view</a>"

    return getHeader() + output + getFooter()


# Expected requestUri: e.g. /osm/suspicious-streets/ormezo/view-result
def handleSuspiciousStreets(requestUri, workdir):
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
            if len(result[1]):
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
        output += str(len(finder.suspiciousStreets)) + " suspicious streets, " + str(houseNrCount) + " missing house numbers in total"
        if doneNrCount > 0 or houseNrCount > 0:
            percent = "%.2f" % (doneNrCount / (doneNrCount + houseNrCount) * 100)
        else:
            percent = "100"
        output += " (vs " + str(doneNrCount) + " present, ie " + str(percent) + "% complete).\n"

        # Write the bottom line to a file, so the index page show it fast.
        with open(os.path.join(workdir, relation + ".percent"), "w") as sock:
            sock.write(percent)

    return getHeader() + output + getFooter()


def getLastModified(workdir, path):
    t = os.path.getmtime(os.path.join(workdir, path))
    return datetime.datetime.fromtimestamp(t).isoformat()


def getContent(workdir, path):
    ret = ""
    with open(os.path.join(workdir, path)) as sock:
        ret = sock.read()
    return ret


def handleMain(relations, workdir):
    output = ""

    output += "<h1>osm scripts</h1>"

    output += "<h2>streets</h2>"

    output += "<ul>"
    for k, v in relations.items():
        output += "<li>"
        output += "<a href=\"https://www.openstreetmap.org/relation/" + str(v) + "\">" + k + "</a>: <ul>"
        output += "<li><a href=\"/osm/streets/" + k + "/view-query\">view query</a></li>"
        date = getLastModified(workdir, "streets-" + k + ".csv")
        output += "<li><a href=\"/osm/streets/" + k + "/view-result\">view result</a> (updated on " + date + ")</li>"
        output += "<li><strong><a href=\"/osm/streets/" + k + "/update-result\">query overpass</a></strong></li>"
        output += "</ul></li>"
    output += "</ul>"

    output += "<h2>street-housenumbers</h2>"

    output += "<ul>"
    for k, v in relations.items():
        output += "<li>"
        output += "<a href=\"https://www.openstreetmap.org/relation/" + str(v) + "\">" + k + "</a>: <ul>"
        output += "<li><a href=\"/osm/street-housenumbers/" + k + "/view-query\">view query</a></li>"
        date = getLastModified(workdir, "street-housenumbers-" + k + ".csv")
        output += "<li><a href=\"/osm/street-housenumbers/" + k + "/view-result\">view result</a> (updated on " + date + ")</li>"
        output += "<li><strong><a href=\"/osm/street-housenumbers/" + k + "/update-result\">query overpass</a></strong></li>"
        output += "</ul></li>"
    output += "</ul>"

    output += "<h2>suspicious-streets</h2>"

    output += "<ul>"
    for k, v in relations.items():
        output += "<li>"
        output += "<a href=\"https://www.openstreetmap.org/relation/" + str(v) + "\">" + k + "</a>: "
        output += "<strong><a href=\"/osm/suspicious-streets/" + k + "/view-result\">view result</a></strong>"
        percentFile = k + ".percent"
        if os.path.exists(os.path.join(workdir, percentFile)):
            percent = getContent(workdir, percentFile)
            date = getLastModified(workdir, percentFile)
            output += ": " + percent + "% (updated on " + date + ")"
        output += "</li>"
    output += "</ul>"

    return getHeader() + output + getFooter()


def getHeader():
    output = "<html><body>"
    output += "<div><a href=\"/osm\">index</a> &brvbar; <a href=\"https://github.com/vmiklos/osm-gimmisn\">github</a></div><hr/>"
    return output


def getFooter():
    output = "<hr/><div>OSM data © OpenStreetMap contributors.</div>"
    output += "</body></html>"
    return output


def application(environ, start_response):
    status = '200 OK'

    requestUri = environ.get("REQUEST_URI")

    workdir = getWorkdir()

    relations = getRelations()

    if requestUri.startswith("/osm/streets/"):
        output = handleStreets(requestUri, workdir)
    elif requestUri.startswith("/osm/street-housenumbers/"):
        output = handleStreetHousenumbers(requestUri, workdir)
    elif requestUri.startswith("/osm/suspicious-streets/"):
        output = handleSuspiciousStreets(requestUri, workdir)
    else:
        output = handleMain(relations, workdir)

    outputBytes = output.encode('utf-8')
    response_headers = [('Content-type', 'text/html; charset=utf-8'),
                        ('Content-Length', str(len(outputBytes)))]
    start_response(status, response_headers)
    return [outputBytes]

# vim:set shiftwidth=4 softtabstop=4 expandtab:
