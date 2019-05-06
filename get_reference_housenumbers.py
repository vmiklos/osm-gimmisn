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

suffix = ""
mode = ""


# Reads list of streets for an area from OSM.
def getStreets():
    ret = []

    sock = open("workdir/streets%s.csv" % suffix)
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


# Returns URL of a street based on config.
def getStreetURL(street, prefix):
    relations = yaml.load(open("data/relations.yaml"))
    relation = relations[suffix[1:]]
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
    if suffix == "-sashegy" and street in sashegy_extra_streets:
        # This city part isn't a strict subset of a city district, these are the exceptions.
        reftelepules = "012"

    tokens = street.split(' ')
    streetName = " ".join(tokens[:-1])
    streetType = tokens[-1]

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
def getReferenceHouseNumbers(street, prefix):
    url = getStreetURL(street, prefix)
    print("considering '" + url + "'")
    urlHash = getURLHash(url)

    if not os.path.exists("workdir/cache"):
        os.makedirs("workdir/cache")

    cachePath = "workdir/cache/" + urlHash
    if not os.path.exists(cachePath):
        # Not in cache, download.
        sys.stderr.write("downloading '" + url + "'...")
        try:
            urlSock = urllib.request.urlopen(url)
            buf = urlSock.read()
            sys.stderr.write(" done.\n")
        except urllib.error.HTTPError:
            buf = b''
            sys.stderr.write(" not found.\n")
        cacheSock = open(cachePath, "w")
        string = buf.decode('utf-8')
        cacheSock.write(string)
        cacheSock.close()
        urlSock.close()

    sock = open(cachePath)
    string = sock.read()

    try:
        j = json.loads(string)
    except json.decoder.JSONDecodeError:
        return []
    return [helpers.simplify(street + " " + i["displayValueHouseNumber"]) for i in j]


# Gets known house numbers (not their coordinates) from a reference site, based
# on street names from OSM.
def main():
    global suffix
    global mode
    if len(sys.argv) > 1:
        suffix = sys.argv[1]
    if len(sys.argv) > 2:
        mode = sys.argv[2]
    # Sample config:
    # [get-reference-housenumbers]
    # prefix = ...
    config = configparser.ConfigParser()
    rc = os.path.join(os.environ['HOME'], '.get-reference-housenumbersrc')
    config.read(rc)
    prefix = config.get('get-reference-housenumbers', 'prefix').strip()
    streets = getStreets()

    lst = []  # type: List[str]
    for street in streets:
        lst += getReferenceHouseNumbers(street, prefix)

    lst = sorted(set(lst))
    sock = open("workdir/street-housenumbers-reference%s.lst" % suffix, "w")
    for l in lst:
        sock.write(l + "\n")
    sock.close()


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
