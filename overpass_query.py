#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The overpass_query module allows getting data out of the OSM DB without a full download."""

from typing import cast
import urllib.request
import urllib.error
import re
import sys

import config


def overpass_query(conf: config.Config2, query: str) -> str:
    """Posts the query string to the overpass API and returns the result string."""
    url = conf.get_overpass_uri() + "/api/interpreter"

    with urllib.request.urlopen(url, bytes(query, "utf-8")) as stream:
        buf = stream.read()

    return cast(str, buf.decode('utf-8'))


def overpass_query_need_sleep(conf: config.Config2) -> int:
    """Checks if we need to sleep before executing an overpass query."""
    try:
        with urllib.request.urlopen(conf.get_overpass_uri() + "/api/status") as sock:
            buf = sock.read()
    except urllib.error.HTTPError:
        return 0
    status = buf.decode('utf-8')
    sleep = 0
    available = False
    for line in status.splitlines():
        if line.startswith("Slot available after:"):
            # Wait one more second just to be safe.
            sleep = int(re.sub(r".*in (-?\d+) seconds.*", r"\1", line.strip())) + 1
            if sleep <= 0:
                sleep = 1
            break
        if "available now" in line:
            available = True
    if available:
        return 0
    return sleep


def main() -> None:
    """Commandline interface to this module."""
    conf = config.make_config()
    with open(sys.argv[1]) as stream:
        query = stream.read()

    try:
        buf = overpass_query(conf, query)

        sys.stdout.write(buf)
    except urllib.error.HTTPError as http_error:
        print("overpass query failed: " + str(http_error))


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
