#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The overpass_query module allows getting data out of the OSM DB without a full download."""

import re

import context
import rust


overpass_query = rust.py_overpass_query


def overpass_query_need_sleep(ctx: context.Context) -> int:
    """Checks if we need to sleep before executing an overpass query."""
    try:
        buf = ctx.get_network().urlopen(ctx.get_ini().get_overpass_uri() + "/api/status", str())
    except OSError:
        return 0
    status = buf
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
