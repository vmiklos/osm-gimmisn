#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The stats module creates statistics about missing / non-missing house numbers."""

from typing import Any
from typing import BinaryIO
from typing import Dict
from typing import List
from typing import Tuple
from cron import warning
from cron import info
import datetime
import json
import os
import time

import context
import util


def handle_progress(ctx: context.Context, src_root: str, j: Dict[str, Any]) -> None:
    """Generates stats for a global progressbar."""
    info("+ Generating progress")
    ret: Dict[str, Any] = {}
    with open(os.path.join(src_root, "ref.count"), "r") as stream:
        num_ref = int(stream.read().strip())
    today = time.strftime("%Y-%m-%d", time.gmtime(ctx.get_time().now()))
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

def handle_capital_progress(ctx: context.Context, src_root: str, j: Dict[str, Any]) -> None:
    """Generates status for the progress of the capital."""
    info("+ Generating capital progress")
    ret: Dict[str, Any] = {}
    num_ref = 0
    num_osm = 0
    today = time.strftime("%Y-%m-%d", time.gmtime(ctx.get_time().now()))
    ref_citycount_path = ctx.get_ini().get_reference_citycounts_path()
    osm_citycount_path = os.path.join(src_root, "%s.citycount" % today)

 
    with open(ref_citycount_path, "r") as stream:
        first = True
        for line in stream.readlines():
            if first:
                first = False
                continue
            cells = line.strip().split('\t')
            city = cells[0]
            count = int(cells[1])

            if city.startswith("budapest_"):
                num_ref += int(count)

    with ctx.get_file_system().open_read(osm_citycount_path) as stream:
        for line_bytes in stream.readlines():
            line = util.from_bytes(line_bytes).strip()
            city, _, count = line.partition('\t')
            if city.startswith("budapest_"):
                num_osm += int(count)

    percentage = round(num_osm * 100 / num_ref, 2)
    ret["date"] = today
    ret["percentage"] = percentage
    ret["reference"] = num_ref
    ret["osm"] = num_osm
    j["capital-progress"] = ret

def handle_topusers(ctx: context.Context, src_root: str, j: Dict[str, Any]) -> None:
    """Generates stats for top users."""
    info("+ Generating topusers")
    today = time.strftime("%Y-%m-%d", time.gmtime(ctx.get_time().now()))
    ret = []
    topusers_path = os.path.join(src_root, "%s.topusers" % today)
    if os.path.exists(topusers_path):
        with open(topusers_path, "r") as stream:
            for line in stream.readlines():
                line = line.strip()
                count, _, user = line.partition(' ')
                ret.append([user, count])
    j["topusers"] = ret


def get_topcities(ctx: context.Context, src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    info("+ Generating topcities")
    ret: List[Tuple[str, int]] = []
    new_day = datetime.date.fromtimestamp(ctx.get_time().now()).strftime("%Y-%m-%d")
    day_delta = datetime.date.fromtimestamp(ctx.get_time().now()) - datetime.timedelta(days=30)
    old_day = day_delta.strftime("%Y-%m-%d")
    old_counts: Dict[str, int] = {}
    counts: List[Tuple[str, int]] = []

    old_count_path = os.path.join(src_root, "%s.citycount" % old_day)
    if not ctx.get_file_system().path_exists(old_count_path):
        return ret
    with ctx.get_file_system().open_read(old_count_path) as stream:
        for line_bytes in stream.readlines():
            line = util.from_bytes(line_bytes).strip()
            city, _, count = line.partition('\t')
            if count:
                old_counts[city] = int(count)

    new_count_path = os.path.join(src_root, "%s.citycount" % new_day)
    if not ctx.get_file_system().path_exists(new_count_path):
        return ret
    with ctx.get_file_system().open_read(new_count_path) as stream:
        for line_bytes in stream.readlines():
            line = util.from_bytes(line_bytes.strip())
            city, _, count = line.partition('\t')
            if count and city in old_counts:
                counts.append((city, int(count) - old_counts[city]))
    ret = sorted(counts, key=lambda x: x[1], reverse=True)
    return ret


def handle_topcities(ctx: context.Context, src_root: str, j: Dict[str, Any]) -> None:
    """
    Generates stats for top cities.
    This lists the top 20 cities which got lots of new house numbers in the past 30 days.
    """
    ret = get_topcities(ctx, src_root)
    ret = ret[:20]
    j["topcities"] = ret


def handle_user_total(ctx: context.Context, src_root: str, j: Dict[str, Any], day_range: int = 13) -> None:
    """Shows # of total users / day."""
    info("+ Generating user total")
    ret = []
    for day_offset in range(day_range, -1, -1):
        day_delta = datetime.date.fromtimestamp(ctx.get_time().now()) - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.usercount" % day)
        if not os.path.exists(count_path):
            warning("file %s not exists, exiting", count_path)
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day, count])
    j["usertotal"] = ret


def handle_daily_new(ctx: context.Context, src_root: str, j: Dict[str, Any], day_range: int = 14) -> None:
    """Shows # of new housenumbers / day."""
    info("+ Generating daily new")
    ret = []
    prev_count = 0
    prev_day = ""
    for day_offset in range(day_range, -1, -1):
        day_delta = datetime.date.fromtimestamp(ctx.get_time().now()) - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % day)
        if not os.path.exists(count_path):
            warning("file %s not exists, exiting", count_path)
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


def handle_monthly_new(ctx: context.Context, src_root: str, j: Dict[str, Any], month_range: int = 12) -> None:
    """Shows # of new housenumbers / month."""
    info("+ Generating monthly new")
    ret = []
    prev_count = 0
    prev_month = ""
    path_exists = ctx.get_file_system().path_exists
    for month_offset in range(month_range, -1, -1):
        month_delta = get_previous_month(datetime.date.fromtimestamp(ctx.get_time().now()), month_offset)
        # Get the first day of each month.
        month = month_delta.replace(day=1).strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % month)
        if not path_exists(count_path):
            warning("file %s not exists, exiting", count_path)
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        if prev_count:
            ret.append([prev_month[:len("YYYY-MM")], count - prev_count])
        prev_count = count
        prev_month = month

    # Also show the current, incomplete month.
    day = datetime.date.fromtimestamp(ctx.get_time().now()).strftime("%Y-%m-%d")
    count_path = os.path.join(src_root, "%s.count" % day)
    if path_exists(count_path):
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day[:len("YYYY-MM")], count - prev_count])

    j["monthly"] = ret


def handle_daily_total(ctx: context.Context, src_root: str, j: Dict[str, Any], day_range: int = 13) -> None:
    """Shows # of total housenumbers / day."""
    info("+ Generating daily total")
    ret = []
    for day_offset in range(day_range, -1, -1):
        day_delta = datetime.date.fromtimestamp(ctx.get_time().now()) - datetime.timedelta(day_offset)
        day = day_delta.strftime("%Y-%m-%d")
        count_path = os.path.join(src_root, "%s.count" % day)
        if not os.path.exists(count_path):
            warning("file %s not exists, exiting", count_path)
            break
        with open(count_path, "r") as stream:
            count = int(stream.read().strip())
        ret.append([day, count])
    j["dailytotal"] = ret


def handle_monthly_total(ctx: context.Context, src_root: str, j: Dict[str, Any], month_range: int = 11) -> None:
    """Shows # of total housenumbers / month."""
    info("+ Generating monthly total")
    ret = []
    for month_offset in range(month_range, -1, -1):
        today = datetime.date.fromtimestamp(ctx.get_time().now())
        month_delta = get_previous_month(today, month_offset)
        prev_month_delta = get_previous_month(today, month_offset + 1)
        # Get the first day of each past month.
        month = month_delta.replace(day=1).strftime("%Y-%m-%d")
        prev_month = prev_month_delta.replace(day=1).strftime("%Y-%m")
        count_path = os.path.join(src_root, "%s.count" % month)
        if not os.path.exists(count_path):
            warning("file %s not exists, exiting", count_path)
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


def generate_json(ctx: context.Context, state_dir: str, stream: BinaryIO) -> None:
    """Generates the stats json and writes it to `stream`."""
    j: Dict[str, Any] = {}
    handle_progress(ctx, state_dir, j)
    handle_capital_progress(ctx, state_dir, j)
    handle_topusers(ctx, state_dir, j)
    handle_topcities(ctx, state_dir, j)
    handle_user_total(ctx, state_dir, j)
    handle_daily_new(ctx, state_dir, j)
    handle_daily_total(ctx, state_dir, j)
    handle_monthly_new(ctx, state_dir, j)
    handle_monthly_total(ctx, state_dir, j)
    stream.write(util.to_bytes(json.dumps(j)))


# vim:set shiftwidth=4 softtabstop=4 expandtab:
