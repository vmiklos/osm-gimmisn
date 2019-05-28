#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The overpass_query module allows getting data out of the OSM DB without a full download."""

from urllib.request import urlopen
import sys


def overpass_query(query):
    """Posts the query string to the overpass API and returns the result string."""
    url = "http://overpass-api.de/api/interpreter"

    sock = urlopen(url, bytes(query, "utf-8"))
    buf = sock.read()
    sock.close()

    return buf.decode('utf-8')


def main():
    """Commandline interface to this module."""
    sock = open(sys.argv[1])
    query = sock.read()
    sock.close()

    buf = overpass_query(query)

    sys.stdout.write(buf)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
