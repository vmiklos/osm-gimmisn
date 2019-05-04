#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

import configparser
import hashlib
import io
import json
import os
import sys
import urllib.error
import urllib.request

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
    simplifiedStreet = simplify(street, spaceDecode=True)
    if simplifiedStreet == "zolyomi_koz":
        # Really strange, survey confirms OSM is correct here, so map it
        # instead.
        simplifiedStreet = "zolyom_koz"
    elif simplifiedStreet == "kiss_janos_altabornagy_utca":
        simplifiedStreet = "kiss_janos_altb._utca"
    district = "xi"
    if suffix == "-sashegy" and simplifiedStreet in ("brezno_lepcso", "kallo_esperes_utca", "sasfiok_utca", "sion_lepcso", "somorjai_utca"):
        # This city part isn't a strict subset of a city district, these are the exceptions.
        district = "xii"
    elif suffix == "-nemetvolgy":
        district = "xii"
    elif suffix == "-farkasvolgy":
        district = "xii"
    elif suffix == "-magasut":
        district = "xii"
    elif suffix == "-farkasret":
        district = "xii"
    elif suffix == "-terezvaros":
        district = "vi"
    elif suffix == "-madarhegy":
        district = "xi"
    elif suffix == "-hosszuret":
        district = "xi"
    elif suffix == "-spanyolret":
        district = "xi"
    return prefix + "/budapest%20" + district + ".ker./" + simplifiedStreet + "/all.json"


# Returns SHA256 hash of an URL.
def getURLHash(url):
    return hashlib.sha256(url.encode('utf-8')).hexdigest()


# Handles normalization of a street name.
def simplify(s, spaceDecode=False):
    s = s.replace('Á', 'A').replace('á', 'a')
    s = s.replace('É', 'E').replace('é', 'e')
    s = s.replace('Í', 'I').replace('í', 'i')
    s = s.replace('Ó', 'O').replace('ó', 'o')
    s = s.replace('Ö', 'O').replace('ö', 'o')
    s = s.replace('Ő', 'O').replace('ő', 'o')
    s = s.replace('Ú', 'U').replace('ú', 'u')
    s = s.replace('Ü', 'U').replace('ü', 'u')
    s = s.replace('Ű', 'U').replace('ű', 'u')
    if spaceDecode:
        s = s.replace(' ', '%20')
    else:
        s = s.replace(' ', '_')
    s = s.lower()
    return s


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
            sys.stderr.write(" done.\n")
        except urllib.error.HTTPError:
            urlSock = io.StringIO("")
            sys.stderr.write(" not found.\n")
        cacheSock = open(cachePath, "w")
        buf = urlSock.read()
        if type(buf) == bytes:
            buf = buf.decode('utf-8')
        cacheSock.write(buf)
        cacheSock.close()
        urlSock.close()
    elif mode == "-delete":
        os.unlink(cachePath)
        return []

    sock = open(cachePath)
    buf = sock.read()

    try:
        j = json.loads(buf)
    except Exception:
        return []
    return [simplify(street + " " + i["label"]) for i in j]


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

    lst = []
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
