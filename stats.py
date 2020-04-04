#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The stats module creates statistics about missing / non-missing house numbers."""

from typing import Any
from typing import Dict
import datetime
import json
import time


def handle_progress(j: Dict[str, Any]) -> None:
    """Generates stats for a global progressbar."""
    ret: Dict[str, Any] = {}
    with open("ref.count", "r") as stream:
        num_ref = int(stream.read().strip())
    today = time.strftime("%Y-%m-%d")
    with open("%s.count" % today, "r") as stream:
        num_osm = int(stream.read().strip())
    percentage = round(num_osm * 100 / num_ref, 2)
    ret["date"] = today
    ret["percentage"] = percentage
    ret["reference"] = num_ref
    ret["osm"] = num_osm
    j["progress"] = ret


def handle_topusers(j: Dict[str, Any]) -> None:
    """Generates stats for top users."""
    today = time.strftime("%Y-%m-%d")
    ret = []
    with open("%s.topusers" % today, "r") as stream:
        for line in stream.readlines():
            line = line.strip()
            count, _, user = line.partition(' ')
            ret.append([user, count])
    j["topusers"] = ret


def handle_daily_new(j: Dict[str, Any]) -> None:
    """Shows # of new housenumbers / day."""
    ret = []
    prev_count = 0
    prev_day = ""
    for day_offset in range(7, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        with open("%s.count" % day, "r") as stream:
            count = int(stream.read().strip())
        if prev_count:
            ret.append([prev_day, count - prev_count])
        prev_count = count
        prev_day = day
    j["daily"] = ret


def handle_daily_total(j: Dict[str, Any]) -> None:
    """Shows # of total housenumbers / day."""
    ret = []
    for day_offset in range(6, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        with open("%s.count" % day, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day, count])
    j["dailytotal"] = ret


def main() -> None:
    """Commandline interface to this module."""
    j: Dict[str, Any] = {}
    handle_progress(j)
    handle_topusers(j)
    handle_daily_new(j)
    handle_daily_total(j)
    print(json.dumps(j))


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
