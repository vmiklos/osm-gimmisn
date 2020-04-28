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
import os
import sys
import time

import dateutil.relativedelta


def handle_progress(src_root: str, j: Dict[str, Any]) -> None:
    """Generates stats for a global progressbar."""
    ret: Dict[str, Any] = {}
    with open(os.path.join(src_root, "ref.count"), "r") as stream:
        num_ref = int(stream.read().strip())
    today = time.strftime("%Y-%m-%d")
    with open(os.path.join(src_root, "%s.count" % today), "r") as stream:
        num_osm = int(stream.read().strip())
    percentage = round(num_osm * 100 / num_ref, 2)
    ret["date"] = today
    ret["percentage"] = percentage
    ret["reference"] = num_ref
    ret["osm"] = num_osm
    j["progress"] = ret


def handle_topusers(src_root: str, j: Dict[str, Any]) -> None:
    """Generates stats for top users."""
    today = time.strftime("%Y-%m-%d")
    ret = []
    with open(os.path.join(src_root, "%s.topusers" % today), "r") as stream:
        for line in stream.readlines():
            line = line.strip()
            count, _, user = line.partition(' ')
            ret.append([user, count])
    j["topusers"] = ret


def handle_daily_new(src_root: str, j: Dict[str, Any]) -> None:
    """Shows # of new housenumbers / day."""
    ret = []
    prev_count = 0
    prev_day = ""
    for day_offset in range(14, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        with open(os.path.join(src_root, "%s.count" % day), "r") as stream:
            count = int(stream.read().strip())
        if prev_count:
            ret.append([prev_day, count - prev_count])
        prev_count = count
        prev_day = day
    j["daily"] = ret


def handle_monthly_new(src_root: str, j: Dict[str, Any]) -> None:
    """Shows # of new housenumbers / month."""
    ret = []
    prev_count = 0
    prev_month = ""
    for month_offset in range(12, -1, -1):
        # datetime.timedelta does not support months
        month_delta = datetime.date.today() - dateutil.relativedelta.relativedelta(months=month_offset)
        # Get the first day of each month.
        month = month_delta.replace(day=1).strftime("%Y-%m-%d")
        with open(os.path.join(src_root, "%s.count" % month), "r") as stream:
            count = int(stream.read().strip())
        if prev_count:
            ret.append([prev_month[:len("YYYY-MM")], count - prev_count])
        prev_count = count
        prev_month = month

    # Also show the current, incomplete month.
    day = datetime.date.today().strftime("%Y-%m-%d")
    with open(os.path.join(src_root, "%s.count" % day), "r") as stream:
        count = int(stream.read().strip())
    ret.append([day[:len("YYYY-MM")], count - prev_count])

    j["monthly"] = ret


def handle_daily_total(src_root: str, j: Dict[str, Any]) -> None:
    """Shows # of total housenumbers / day."""
    ret = []
    for day_offset in range(13, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        with open(os.path.join(src_root, "%s.count" % day), "r") as stream:
            count = int(stream.read().strip())
        ret.append([day, count])
    j["dailytotal"] = ret


def handle_monthly_total(src_root: str, j: Dict[str, Any]) -> None:
    """Shows # of total housenumbers / month."""
    ret = []
    for month_offset in range(11, -1, -1):
        # datetime.timedelta does not support months
        month_delta = datetime.date.today() - dateutil.relativedelta.relativedelta(months=month_offset)
        prev_month_delta = datetime.date.today() - dateutil.relativedelta.relativedelta(months=month_offset + 1)
        # Get the first day of each past month.
        month = month_delta.replace(day=1).strftime("%Y-%m-%d")
        prev_month = prev_month_delta.replace(day=1).strftime("%Y-%m")
        with open(os.path.join(src_root, "%s.count" % month), "r") as stream:
            count = int(stream.read().strip())
        ret.append([prev_month, count])

        if month_offset == 0:
            # Current month: show today's count as well.
            month = month_delta.strftime("%Y-%m-%d")
            with open(os.path.join(src_root, "%s.count" % month), "r") as stream:
                count = int(stream.read().strip())
            ret.append([month[:len("YYYY-MM")], count])
    j["monthlytotal"] = ret


def main() -> None:
    """Commandline interface to this module."""
    src_root = sys.argv[1]
    j: Dict[str, Any] = {}
    handle_progress(src_root, j)
    handle_topusers(src_root, j)
    handle_daily_new(src_root, j)
    handle_daily_total(src_root, j)
    handle_monthly_new(src_root, j)
    handle_monthly_total(src_root, j)
    print(json.dumps(j))


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
