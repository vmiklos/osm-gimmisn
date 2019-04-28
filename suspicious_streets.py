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
import yaml

suffix = ""


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
        if n >= self.start and n <= self.end:
            return True
        return False


def getArea():
    return suffix


def simplify(s):
    """ Handles normalization of a street name."""
    s = s.replace('Á', 'A').replace('á', 'a')
    s = s.replace('É', 'E').replace('é', 'e')
    s = s.replace('Í', 'I').replace('í', 'i')
    s = s.replace('Ó', 'O').replace('ó', 'o')
    s = s.replace('Ö', 'O').replace('ö', 'o')
    s = s.replace('Ő', 'O').replace('ő', 'o')
    s = s.replace('Ú', 'U').replace('ú', 'u')
    s = s.replace('Ü', 'U').replace('ü', 'u')
    s = s.replace('Ű', 'U').replace('ű', 'u')
    s = s.replace(' ', '_').lower()
    return s


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
    houseNumbers = []
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
        houseNumbers += normalize(tokens[2], simplify(streetName))
    streetHouseNumbersSock.close()
    return sorted(set(houseNumbers))


def getHouseNumbersFromLst(streetName):
    houseNumbers = []
    lstStreetName = simplify(streetName)
    prefix = lstStreetName + "_"
    sock = open("street-housenumbers-reference%s.lst" % getArea())
    for line in sock.readlines():
        line = line.strip()
        if line.startswith(prefix):
            houseNumbers += normalize(line.replace(prefix, ''), lstStreetName)
    sock.close()
    return sorted(set(houseNumbers))


def getOnlyInFirst(first, second):
    ret = []
    for i in first:
        if i not in second:
            ret.append(i)
    return ret


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

        for streetName in streetNames:
            referenceHouseNumbers = getHouseNumbersFromLst(streetName)
            osmHouseNumbers = getHouseNumbersFromCsv(streetName)
            onlyInReference = getOnlyInFirst(referenceHouseNumbers, osmHouseNumbers)
            if len(onlyInReference):
                results.append((streetName, onlyInReference))

        # Sort by length.
        results.sort(key=lambda result: len(result[1]), reverse=True)

        self.suspiciousStreets = results


class Test(unittest.TestCase):
    def test_none(self):
        finder = Finder()

        for result in finder.suspiciousStreets:
            if len(result[1]):
                # House number, # of onlyInReference items.
                print("%s\t%s" % (result[0], len(result[1])))
                # onlyInReference items.
                print(result[1])

        self.assertEqual([], finder.suspiciousStreets)


if __name__ == '__main__':
    normalizers = {}
    if len(sys.argv) > 1:
        suffix = sys.argv[1]
        sys.argv = sys.argv[:1]
    if os.path.exists("housenumber-filters%s.yaml" % getArea()):
        with open("housenumber-filters%s.yaml" % getArea()) as sock:
            normalizers = yaml.load(sock)
    elif os.path.exists("housenumber-filters%s.json" % getArea()):
        with open("housenumber-filters%s.json" % getArea()) as sock:
            normalizers = json.load(sock)
    if "filters" in normalizers.keys():
        filters = normalizers["filters"]
        for street in filters.keys():
            i = []
            for r in filters[street]["ranges"]:
                i.append(Range(int(r["start"]), int(r["end"]), r["isOdd"] == "true"))
            normalizers[street] = Ranges(i)
    os.chdir("workdir")
    unittest.main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
