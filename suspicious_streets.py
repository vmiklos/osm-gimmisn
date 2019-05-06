#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

# What it does:
# Tries to find streets which do have at least one house number, but suspicious
# as lots of house numbers are probably missing.

import json
import os
import re
import sys
import unittest
from typing import Dict, List
import yaml
import helpers

suffix = ""
normalizers = {}  # type: Dict[str, Ranges]


# A Ranges object contains an item if any of its Range objects contains it.
class Ranges:
    def __init__(self, items):
        self.items = items

    def __call__(self, item):
        for i in self.items:
            if item in i:
                return True
        return False


# A range object represents an odd or even range of integer numbers.
class Range:
    def __init__(self, start, end, isOdd):
        self.start = start
        self.end = end
        self.isOdd = isOdd

    def __contains__(self, n):
        if self.isOdd != (n % 2 == 1):
            return False
        if self.start <= n <= self.end:
            return True
        return False


def getArea():
    return suffix


def normalize(houseNumbers, streetName):
    """Strips down string input to bare minimum that can be interpreted as an
    actual number. Think about a/b, a-b, and so on."""
    ret = []
    for houseNumber in houseNumbers.split('-'):
        try:
            n = int(re.sub(r"([0-9]+).*", r"\1", houseNumber))
        except ValueError:
            continue
        if streetName in normalizers.keys():
            if not normalizers[streetName](n):
                continue
        ret.append(str(n))
    return ret


def getHouseNumbersFromCsv(streetName):
    houseNumbers = []  # type: List[int]
    streetHouseNumbersSock = open("street-housenumbers%s.csv" % getArea())
    first = True
    for line in streetHouseNumbersSock.readlines():
        if first:
            first = False
            continue
        tokens = line.strip().split('\t')
        if len(tokens) < 3:
            continue
        if tokens[1] != streetName:
            continue
        houseNumbers += normalize(tokens[2], helpers.simplify(streetName))
    streetHouseNumbersSock.close()
    return helpers.sort_numerically(set(houseNumbers))


def getHouseNumbersFromLst(streetName):
    houseNumbers = []  # type: List[int]
    lstStreetName = helpers.simplify(streetName)
    prefix = lstStreetName + "_"
    sock = open("street-housenumbers-reference%s.lst" % getArea())
    for line in sock.readlines():
        line = line.strip()
        if line.startswith(prefix):
            houseNumbers += normalize(line.replace(prefix, ''), lstStreetName)
    sock.close()
    return helpers.sort_numerically(set(houseNumbers))


class Finder:
    def __init__(self):
        streetsSock = open("streets%s.csv" % getArea())
        streetNames = []
        firstStreet = True
        for streetLine in streetsSock.readlines():
            if firstStreet:
                firstStreet = False
                continue
            streetTokens = streetLine.strip().split('\t')
            if len(streetTokens) > 1:
                streetNames.append(streetTokens[1])
        streetsSock.close()
        streetNames = sorted(set(streetNames))

        results = []
        bothResults = []

        for streetName in streetNames:
            referenceHouseNumbers = getHouseNumbersFromLst(streetName)
            osmHouseNumbers = getHouseNumbersFromCsv(streetName)
            onlyInReference = helpers.get_only_in_first(referenceHouseNumbers, osmHouseNumbers)
            inBoth = helpers.get_in_both(referenceHouseNumbers, osmHouseNumbers)
            if onlyInReference:
                results.append((streetName, onlyInReference))
            if inBoth:
                bothResults.append((streetName, inBoth))

        # Sort by length.
        results.sort(key=lambda result: len(result[1]), reverse=True)

        self.suspiciousStreets = results
        self.doneStreets = bothResults


class Test(unittest.TestCase):
    def test_none(self):
        finder = Finder()

        for result in finder.suspiciousStreets:
            if result[1]:
                # House number, # of onlyInReference items.
                print("%s\t%s" % (result[0], len(result[1])))
                # onlyInReference items.
                print(result[1])

        self.assertEqual([], finder.suspiciousStreets)


def loadNormalizers():
    global normalizers
    cwd = os.getcwd()
    os.chdir(os.path.dirname(__file__))
    if os.path.exists("data/housenumber-filters%s.yaml" % getArea()):
        with open("data/housenumber-filters%s.yaml" % getArea()) as sock:
            config = yaml.load(sock)
    elif os.path.exists("data/housenumber-filters%s.json" % getArea()):
        with open("data/housenumber-filters%s.json" % getArea()) as sock:
            config = json.load(sock)
    if "filters" in config.keys():
        filters = config["filters"]
        for street in filters.keys():
            i = []
            for r in filters[street]["ranges"]:
                i.append(Range(int(r["start"]), int(r["end"]), r["isOdd"] == "true"))
            normalizers[street] = Ranges(i)
    os.chdir(cwd)


if __name__ == '__main__':
    if len(sys.argv) > 1:
        suffix = sys.argv[1]
        sys.argv = sys.argv[:1]
    loadNormalizers()
    os.chdir("workdir")
    unittest.main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
