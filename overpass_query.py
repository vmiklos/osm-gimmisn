#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The overpass_query module allows getting data out of the OSM DB without a full download."""

from typing import Tuple
import re

import context


def overpass_query(ctx: context.Context, query: str) -> Tuple[str, str]:
    """Posts the query string to the overpass API and returns the result string."""
    url = ctx.get_ini().get_overpass_uri() + "/api/interpreter"

    urlopen = ctx.get_network().urlopen
    buf, err = urlopen(url, bytes(query, "utf-8"))

    return (buf.decode('utf-8'), err)


def overpass_query_need_sleep(ctx: context.Context) -> int:
    """Checks if we need to sleep before executing an overpass query."""
    urlopen = ctx.get_network().urlopen
    buf, err = urlopen(ctx.get_ini().get_overpass_uri() + "/api/status")
    if err:
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
