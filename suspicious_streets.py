#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

# What it does:
# Tries to find streets which do have at least one house number, but suspicious
# as lots of house numbers are probably missing.

import os
import re
import sys
# pylint: disable=unused-import
from typing import Dict, List
import configparser
import yaml
import helpers


class Finder:
    def __init__(self, datadir, workdir, relationName):
        self.normalizers = {}  # type: Dict[str, helpers.Ranges]
        # OSM name -> ref name map
        self.refStreets = {}  # type: Dict[str, str]

        self.loadNormalizers(datadir, relationName)
        streetNames = helpers.get_streets(workdir, relationName)

        results = []
        bothResults = []

        for streetName in streetNames:

            refStreet = streetName
            # See if we need to map the OSM name to ref name.
            if streetName in self.refStreets.keys():
                refStreet = self.refStreets[streetName]

            referenceHouseNumbers = self.getHouseNumbersFromLst(workdir, relationName, streetName, refStreet)
            osmHouseNumbers = self.getHouseNumbersFromCsv(workdir, relationName, streetName)
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

    def loadNormalizers(self, datadir, relationName):
        config = None
        if os.path.exists(os.path.join(datadir, "housenumber-filters-%s.yaml" % relationName)):
            with open(os.path.join(datadir, "housenumber-filters-%s.yaml" % relationName)) as sock:
                config = yaml.load(sock)
        if config and "filters" in config.keys():
            filters = config["filters"]
            for street in filters.keys():
                i = []
                for r in filters[street]["ranges"]:
                    i.append(helpers.Range(int(r["start"]), int(r["end"]), r["isOdd"] == "true"))
                self.normalizers[street] = helpers.Ranges(i)
        if config and "refstreets" in config.keys():
            self.refStreets = config["refstreets"]

    def normalize(self, houseNumbers: str, streetName: str) -> List[str]:
        """Strips down string input to bare minimum that can be interpreted as an
        actual number. Think about a/b, a-b, and so on."""
        ret = []
        for houseNumber in houseNumbers.split('-'):
            try:
                n = int(re.sub(r"([0-9]+).*", r"\1", houseNumber))
            except ValueError:
                continue
            if streetName in self.normalizers.keys():
                if n not in self.normalizers[streetName]:
                    continue
            ret.append(str(n))
        return ret

    def getHouseNumbersFromCsv(self, workdir, relationName, streetName):
        houseNumbers = []  # type: List[str]
        streetHouseNumbersSock = open(os.path.join(workdir, "street-housenumbers-%s.csv" % relationName))
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
            houseNumbers += self.normalize(tokens[2], helpers.simplify(streetName))
        streetHouseNumbersSock.close()
        return helpers.sort_numerically(set(houseNumbers))

    def getHouseNumbersFromLst(self, workdir, relationName, streetName, refStreet):
        houseNumbers = []  # type: List[str]
        lstStreetName = helpers.simplify(refStreet)
        prefix = lstStreetName + "_"
        sock = open(os.path.join(workdir, "street-housenumbers-reference-%s.lst" % relationName))
        for line in sock.readlines():
            line = line.strip()
            if line.startswith(prefix):
                houseNumbers += self.normalize(line.replace(prefix, ''), helpers.simplify(streetName))
        sock.close()
        return helpers.sort_numerically(set(houseNumbers))


def main():
    config = configparser.ConfigParser()
    configPath = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(configPath)
    workdir = config.get('wsgi', 'workdir').strip()
    datadir = os.path.join(os.path.dirname(__file__), "data")

    if len(sys.argv) > 1:
        relationName = sys.argv[1]

    finder = Finder(datadir, workdir, relationName)

    for result in finder.suspiciousStreets:
        if result[1]:
            # House number, # of onlyInReference items.
            print("%s\t%s" % (result[0], len(result[1])))
            # onlyInReference items.
            print(result[1])


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
