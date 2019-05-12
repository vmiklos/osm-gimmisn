#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

import configparser
import hashlib
import json
import os
import sys
import urllib.error
import urllib.request
# pylint: disable=unused-import
from typing import List
import yaml
import helpers


verbose = False


# Reads list of streets for an area from OSM.
def getStreets(workdir, relationName):
    ret = []

    sock = open(os.path.join(workdir, "streets-%s.csv" % relationName))
    first = True
    for line in sock.readlines():
        if first:
            first = False
            continue

        tokens = line.strip().split('\t')
        if len(tokens) < 2:
            continue

        ret.append(tokens[1])

    sock.close()
    return sorted(set(ret))


def getStreetDetails(datadir, street, relationName):
    relations = yaml.load(open(os.path.join(datadir, "relations.yaml")))
    relation = relations[relationName]
    if street == "Zólyomi köz":
        # Really strange, survey confirms OSM is correct here, so map it
        # instead.
        street = "Zólyom köz"
    elif street == "Felsőhatár út":
        # OSM survey confirms the difference
        street = "Felső határ út"
    refmegye = relation["refmegye"]
    reftelepules = relation["reftelepules"]
    sashegy_extra_streets = ("Breznó lépcső", "Kálló esperes utca", "Sasfiók utca", "Sion lépcső", "Somorjai utca")
    if relationName == "sashegy" and street in sashegy_extra_streets:
        # This city part isn't a strict subset of a city district, these are the exceptions.
        reftelepules = "012"

    tokens = street.split(' ')
    streetName = " ".join(tokens[:-1])
    streetType = tokens[-1]
    return refmegye, reftelepules, streetName, streetType


# Returns URL of a street based on config.
def getStreetURL(datadir, street, prefix, relationName):
    refmegye, reftelepules, streetName, streetType = getStreetDetails(datadir, street, relationName)
    url = prefix
    d = {
        "p_p_id": "wardsearch_WAR_nvinvrportlet",
        "p_p_lifecycle": "2",
        "p_p_state": "normal",
        "p_p_mode": "view",
        "p_p_resource_id": "resourceIdGetHazszam",
        "p_p_cacheability": "cacheLevelPage",
        "p_p_col_id": "column-2",
        "p_p_col_count": "1",
        "_wardsearch_WAR_nvinvrportlet_vlId": "291",
        "_wardsearch_WAR_nvinvrportlet_vltId": "684",
        "_wardsearch_WAR_nvinvrportlet_keywords": "",
        "_wardsearch_WAR_nvinvrportlet_megyeKod": refmegye,
        "_wardsearch_WAR_nvinvrportlet_telepulesKod": reftelepules,
        "_wardsearch_WAR_nvinvrportlet_kozterNev": streetName,
        "_wardsearch_WAR_nvinvrportlet_kozterJelleg": streetType,
    }
    url += "?" + urllib.parse.urlencode(d)  # type: ignore
    return url


# Returns SHA256 hash of an URL.
def getURLHash(url):
    return hashlib.sha256(url.encode('utf-8')).hexdigest()


# Gets known house numbers for a single street
def getHouseNumbersOfStreet(datadir, config, relationName, street):
    try:
        local = config.get('wsgi', 'reference_local').strip()
        return getHouseNumbersOfStreetLocal(datadir, local, relationName, street)
    except:
        prefix = config.get('wsgi', 'reference').strip()
        workdir = config.get('wsgi', 'workdir').strip()
        return getHouseNumbersOfStreetRemote(datadir, prefix, workdir, relationName, street)


def getHouseNumbersOfStreetLocal(datadir, local, relationName, street):
    if verbose:
        print("searching '" + street + "'")
    refmegye, reftelepules, _, _ = getStreetDetails(datadir, street, relationName)
    ret = []
    with open(local, "rb") as sock:
        prefix = "\t".join([refmegye, reftelepules, street, ""]).encode("utf-8")
        while True:
            line = sock.readline()
            if not line:
                break
            if line.startswith(prefix):
                houseNumber = line[len(prefix):].decode("utf-8").strip()
                ret.append(helpers.simplify(street + " " + houseNumber))
    return ret


def getHouseNumbersOfStreetRemote(datadir, prefix, workdir, relationName, street):
    url = getStreetURL(datadir, street, prefix, relationName)
    if verbose:
        print("considering '" + url + "'")
    urlHash = getURLHash(url)

    if not os.path.exists(os.path.join(workdir, "cache")):
        os.makedirs(os.path.join(workdir, "cache"))

    cachePath = os.path.join(workdir, "cache", urlHash)
    if not os.path.exists(cachePath):
        # Not in cache, download.
        if verbose:
            sys.stderr.write("downloading '" + url + "'...")
        try:
            urlSock = urllib.request.urlopen(url)
            buf = urlSock.read()
            if verbose:
                sys.stderr.write(" done.\n")
            urlSock.close()
        except urllib.error.HTTPError:
            buf = b''
            if verbose:
                sys.stderr.write(" not found.\n")
        cacheSock = open(cachePath, "w")
        string = buf.decode('utf-8')
        cacheSock.write(string)
        cacheSock.close()

    sock = open(cachePath)
    string = sock.read()

    try:
        j = json.loads(string)
    except json.decoder.JSONDecodeError:
        return []
    return [helpers.simplify(street + " " + i["displayValueHouseNumber"]) for i in j]


def getReferenceHousenumbers(config, relationName):
    datadir = os.path.join(os.path.dirname(__file__), "data")
    workdir = config.get('wsgi', 'workdir').strip()
    streets = getStreets(workdir, relationName)

    lst = []  # type: List[str]
    for street in streets:
        lst += getHouseNumbersOfStreet(datadir, config, relationName, street)

    lst = sorted(set(lst))
    sock = open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relationName), "w")
    for l in lst:
        sock.write(l + "\n")
    sock.close()


# Gets known house numbers (not their coordinates) from a reference site, based
# on street names from OSM.
def main():
    global verbose

    config = configparser.ConfigParser()
    configPath = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(configPath)

    if len(sys.argv) > 1:
        relationName = sys.argv[1]

    verbose = True
    getReferenceHousenumbers(config, relationName)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
