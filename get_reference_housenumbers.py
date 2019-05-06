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
from typing import List
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


suffixToDistricts = {
    "-nemetvolgy": "xii",
    "-farkasvolgy": "xii",
    "-magasut": "xii",
    "-farkasret": "xii",
    "-terezvaros": "vi",
    "-madarhegy": "xi",
    "-hosszuret": "xi",
    "-spanyolret": "xi",
    "-csilleberc": "xii",
    "-dobogo": "xi",
    "-kelenfold": "xi",
    "-orsod": "xi",
    "-szechenyihegy": "xii",
}


# Returns URL of a street based on config.
def getStreetURL(street, prefix):
    simplifiedStreet = helpers.simplify(street, spaceDecode=True)
    if simplifiedStreet == "zolyomi_koz":
        # Really strange, survey confirms OSM is correct here, so map it
        # instead.
        simplifiedStreet = "zolyom_koz"
    elif simplifiedStreet == "kiss_janos_altabornagy_utca":
        simplifiedStreet = "kiss_janos_altb._utca"
    elif simplifiedStreet == "felsohatar_ut":
        # OSM survey confirms the difference
        simplifiedStreet = "felso_hatar_ut"
    district = "xi"
    sashegy_extra_streets = ("brezno_lepcso", "kallo_esperes_utca", "sasfiok_utca", "sion_lepcso", "somorjai_utca")
    if suffix == "-sashegy" and simplifiedStreet in sashegy_extra_streets:
        # This city part isn't a strict subset of a city district, these are the exceptions.
        district = "xii"
    elif suffix in suffixToDistricts.keys():
        district = suffixToDistricts[suffix]
    return prefix + "/budapest%20" + district + ".ker./" + simplifiedStreet + "/all.json"


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
    elif mode == "-delete":
        os.unlink(cachePath)
        return []

    sock = open(cachePath)
    string = sock.read()

    try:
        j = json.loads(string)
    except json.decoder.JSONDecodeError:
        return []
    return [helpers.simplify(street + " " + i["label"]) for i in j]


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
