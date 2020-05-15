#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The stats module creates statistics about missing / non-missing house numbers."""

from typing import Any
from typing import Dict
from typing import TextIO
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
    num_osm = 0
    count_path = os.path.join(src_root, "%s.count" % today)
    if os.path.exists(count_path):
        with open(count_path, "r") as stream:
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
    topusers_path = os.path.join(src_root, "%s.topusers" % today)
    if os.path.exists(topusers_path):
        with open(topusers_path, "r") as stream:
            for line in stream.readlines():
                line = line.strip()
                count, _, user = line.partition(' ')
                ret.append([user, count])
    j["topusers"] = ret


def handle_daily_new(src_root: str, j: Dict[str, Any], day_range: int = 14) -> None:
    """Shows # of new housenumbers / day."""
    ret = []
    prev_count = 0
    prev_day = ""
    for day_offset in range(day_range, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % day)
        if not os.path.exists(count_path):
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        if prev_count:
            ret.append([prev_day, count - prev_count])
        prev_count = count
        prev_day = day
    j["daily"] = ret


def handle_monthly_new(src_root: str, j: Dict[str, Any], month_range: int = 12) -> None:
    """Shows # of new housenumbers / month."""
    ret = []
    prev_count = 0
    prev_month = ""
    for month_offset in range(month_range, -1, -1):
        # datetime.timedelta does not support months
        month_delta = datetime.date.today() - dateutil.relativedelta.relativedelta(months=month_offset)
        # Get the first day of each month.
        month = month_delta.replace(day=1).strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % month)
        if not os.path.exists(count_path):
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        if prev_count:
            ret.append([prev_month[:len("YYYY-MM")], count - prev_count])
        prev_count = count
        prev_month = month

    # Also show the current, incomplete month.
    day = datetime.date.today().strftime("%Y-%m-%d")
    count_path = os.path.join(src_root, "%s.count" % day)
    if os.path.exists(count_path):
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day[:len("YYYY-MM")], count - prev_count])

    j["monthly"] = ret


def handle_daily_total(src_root: str, j: Dict[str, Any]) -> None:
    """Shows # of total housenumbers / day."""
    ret = []
    for day_offset in range(13, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % day)
        if not os.path.exists(count_path):
            break
        with open(count_path, "r") as stream:
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
        count_path = os.path.join(src_root, "%s.count" % month)
        if not os.path.exists(count_path):
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([prev_month, count])

        if month_offset == 0:
            # Current month: show today's count as well.
            month = month_delta.strftime("%Y-%m-%d")
            with open(os.path.join(src_root, "%s.count" % month), "r") as stream:
                count = int(stream.read().strip())
            ret.append([month[:len("YYYY-MM")], count])
    j["monthlytotal"] = ret


def generate_json(state_dir: str, stream: TextIO) -> None:
    """Generates the stats json and writes it to `stream`."""
    j: Dict[str, Any] = {}
    handle_progress(state_dir, j)
    handle_topusers(state_dir, j)
    handle_daily_new(state_dir, j)
    handle_daily_total(state_dir, j)
    handle_monthly_new(state_dir, j)
    handle_monthly_total(state_dir, j)
    stream.write(json.dumps(j))


def main() -> None:
    """Commandline interface to this module."""
    src_root = sys.argv[1]
    stream = sys.stdout
    generate_json(src_root, stream)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
