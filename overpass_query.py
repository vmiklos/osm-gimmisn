#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

from urllib.request import urlopen
import sys


def overpassQuery(query):
    url = "http://overpass-api.de/api/interpreter"

    sock = urlopen(url, bytes(query, "utf-8"))
    buf = sock.read()
    sock.close()

    return buf.decode('utf-8')


if __name__ == "__main__":
    sock = open(sys.argv[1])
    query = sock.read()
    sock.close()

    buf = overpassQuery(query)

    sys.stdout.write(buf)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
