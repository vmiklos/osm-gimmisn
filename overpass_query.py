#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The overpass_query module allows getting data out of the OSM DB without a full download."""

import urllib.request
import urllib.error
import re

import config


def overpass_query(conf: config.Config, query: str) -> str:
    """Posts the query string to the overpass API and returns the result string."""
    url = conf.get_overpass_uri() + "/api/interpreter"

    urlopen = conf.get_network().urlopen
    buf = urlopen(url, bytes(query, "utf-8"))

    return buf.decode('utf-8')


def overpass_query_need_sleep(conf: config.Config) -> int:
    """Checks if we need to sleep before executing an overpass query."""
    urlopen = conf.get_network().urlopen
    try:
        buf = urlopen(conf.get_overpass_uri() + "/api/status")
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


# vim:set shiftwidth=4 softtabstop=4 expandtab:
