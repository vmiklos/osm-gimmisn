#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The stats module creates statistics about missing / non-missing house numbers."""

from typing import Any
from typing import Dict
from typing import List
from typing import TextIO
from typing import Tuple
import datetime
import json
import os
import time

import config


def handle_progress(conf: config.Config, src_root: str, j: Dict[str, Any]) -> None:
    """Generates stats for a global progressbar."""
    ret: Dict[str, Any] = {}
    with open(os.path.join(src_root, "ref.count"), "r") as stream:
        num_ref = int(stream.read().strip())
    today = time.strftime("%Y-%m-%d", time.gmtime(conf.get_time().now()))
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


def handle_topusers(conf: config.Config, src_root: str, j: Dict[str, Any]) -> None:
    """Generates stats for top users."""
    today = time.strftime("%Y-%m-%d", time.gmtime(conf.get_time().now()))
    ret = []
    topusers_path = os.path.join(src_root, "%s.topusers" % today)
    if os.path.exists(topusers_path):
        with open(topusers_path, "r") as stream:
            for line in stream.readlines():
                line = line.strip()
                count, _, user = line.partition(' ')
                ret.append([user, count])
    j["topusers"] = ret


def get_topcities(src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    ret = []
    new_day = datetime.date.today().strftime("%Y-%m-%d")
    day_delta = datetime.date.today() - datetime.timedelta(days=30)
    old_day = day_delta.strftime("%Y-%m-%d")
    old_counts: Dict[str, int] = {}
    counts: List[Tuple[str, int]] = []

    old_count_path = os.path.join(src_root, "%s.citycount" % old_day)
    with open(old_count_path, "r") as stream:
        for line in stream.readlines():
            line = line.strip()
            city, _, count = line.partition('\t')
            if count:
                old_counts[city] = int(count)

    new_count_path = os.path.join(src_root, "%s.citycount" % new_day)
    with open(new_count_path, "r") as stream:
        for line in stream.readlines():
            line = line.strip()
            city, _, count = line.partition('\t')
            if count and city in old_counts:
                counts.append((city, int(count) - old_counts[city]))
    ret = sorted(counts, key=lambda x: x[1], reverse=True)
    return ret


def handle_topcities(src_root: str, j: Dict[str, Any]) -> None:
    """
    Generates stats for top cities.
    This lists the top 20 cities which got lots of new house numbers in the past 30 days.
    """
    ret = get_topcities(src_root)
    ret = ret[:20]
    j["topcities"] = ret


def handle_user_total(src_root: str, j: Dict[str, Any], day_range: int = 13) -> None:
    """Shows # of total users / day."""
    ret = []
    for day_offset in range(day_range, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.usercount" % day)
        if not os.path.exists(count_path):
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day, count])
    j["usertotal"] = ret


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


def get_previous_month(today: datetime.date, months: int) -> datetime.date:
    """Returns a date that was today N months ago."""
    month_ago = today
    for _month in range(months):
        first_of_current = month_ago.replace(day=1)
        month_ago = first_of_current - datetime.timedelta(days=1)
    return month_ago


def handle_monthly_new(src_root: str, j: Dict[str, Any], month_range: int = 12) -> None:
    """Shows # of new housenumbers / month."""
    ret = []
    prev_count = 0
    prev_month = ""
    for month_offset in range(month_range, -1, -1):
        month_delta = get_previous_month(datetime.date.today(), month_offset)
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


def handle_daily_total(src_root: str, j: Dict[str, Any], day_range: int = 13) -> None:
    """Shows # of total housenumbers / day."""
    ret = []
    for day_offset in range(day_range, -1, -1):
        day_delta = datetime.date.today() - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % day)
        if not os.path.exists(count_path):
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day, count])
    j["dailytotal"] = ret


def handle_monthly_total(src_root: str, j: Dict[str, Any], month_range: int = 11) -> None:
    """Shows # of total housenumbers / month."""
    ret = []
    for month_offset in range(month_range, -1, -1):
        month_delta = get_previous_month(datetime.date.today(), month_offset)
        prev_month_delta = get_previous_month(datetime.date.today(), month_offset + 1)
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


def generate_json(conf: config.Config, state_dir: str, stream: TextIO) -> None:
    """Generates the stats json and writes it to `stream`."""
    j: Dict[str, Any] = {}
    handle_progress(conf, state_dir, j)
    handle_topusers(conf, state_dir, j)
    handle_topcities(state_dir, j)
    handle_user_total(state_dir, j)
    handle_daily_new(state_dir, j)
    handle_daily_total(state_dir, j)
    handle_monthly_new(state_dir, j)
    handle_monthly_total(state_dir, j)
    stream.write(json.dumps(j))


# vim:set shiftwidth=4 softtabstop=4 expandtab:
