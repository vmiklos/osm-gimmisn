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
from typing import Dict
from typing import List
import yaml
import helpers


verbose = False
memoryCache = {}  # type: Dict[str, Dict[str, Dict[str, List[str]]]]


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

    # See if config wants to map from OSM name to ref name.
    refstreets = {}  # type: Dict[str, str]
    if os.path.exists("data/housenumber-filters-%s.yaml" % relationName):
        with open("data/housenumber-filters-%s.yaml" % relationName) as sock:
            y = yaml.load(sock)
            if "refstreets" in y.keys():
                refstreets = y["refstreets"]

    if street in refstreets.keys():
        street = refstreets[street]

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
    except configparser.NoOptionError:
        prefix = config.get('wsgi', 'reference').strip()
        workdir = config.get('wsgi', 'workdir').strip()
        return getHouseNumbersOfStreetRemote(datadir, prefix, workdir, relationName, street)


def getHouseNumbersOfStreetLocal(datadir, local, relationName, street):
    global memoryCache

    if not memoryCache:
        if verbose:
            print("building in-memory cache")
        with open(local, "r") as sock:
            first = True
            while True:
                line = sock.readline()
                if first:
                    first = False
                    continue

                if not line:
                    break

                refmegye, reftelepules, street, num = line.strip().split("\t")
                if refmegye not in memoryCache.keys():
                    memoryCache[refmegye] = {}
                if reftelepules not in memoryCache[refmegye].keys():
                    memoryCache[refmegye][reftelepules] = {}
                if street not in memoryCache[refmegye][reftelepules].keys():
                    memoryCache[refmegye][reftelepules][street] = []
                memoryCache[refmegye][reftelepules][street].append(num)

    if verbose:
        print("searching '" + street + "'")
    refmegye, reftelepules, streetName, streetType = getStreetDetails(datadir, street, relationName)
    street = streetName + " " + streetType
    if street in memoryCache[refmegye][reftelepules].keys():
        houseNumbers = memoryCache[refmegye][reftelepules][street]
        return [helpers.simplify(street + " " + i) for i in houseNumbers]

    return []


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
