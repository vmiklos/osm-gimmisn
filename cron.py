#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

from typing import Callable
from typing import Dict
from typing import List
from typing import Set
from typing import TextIO
from typing import cast
import argparse
import datetime
import glob
import os
import sys
import time
import traceback

import areas
import context
import rust
import stats
import util


def info(msg: str) -> None:
    """Wrapper around py_info()."""
    rust.py_info(msg)


def error(msg: str) -> None:
    """Wrapper around py_error()."""
    rust.py_error(msg)


def overpass_sleep(ctx: rust.PyContext) -> None:
    """Sleeps to respect overpass rate limit."""
    rust.py_overpass_sleep(ctx)


def should_retry(retry: int) -> bool:
    """Decides if we should retry a query or not."""
    return retry < 20


def update_osm_streets(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the OSM street list of all relations."""
    rust.py_update_osm_streets(ctx, relations, update)


def update_osm_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the OSM housenumber list of all relations."""
    rust.py_update_osm_housenumbers(ctx, relations, update)


def update_ref_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the reference housenumber list of all relations."""
    rust.py_update_ref_housenumbers(ctx, relations, update)


def update_ref_streets(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the reference street list of all relations."""
    rust.py_update_ref_streets(ctx, relations, update)


def update_missing_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, update: bool) -> None:
    """Update the relation's house number coverage stats."""
    rust.py_update_missing_housenumbers(ctx, relations, update)


def update_missing_streets(relations: rust.PyRelations, update: bool) -> None:
    """Update the relation's street coverage stats."""
    rust.py_update_missing_streets(relations, update)


def update_additional_streets(relations: rust.PyRelations, update: bool) -> None:
    """Update the relation's "additional streets" stats."""
    rust.py_update_additional_streets(relations, update)


def write_count_path(ctx: rust.PyContext, count_path: str, house_numbers: Set[str]) -> None:
    """Writes a daily .count file."""
    with ctx.get_file_system().open_write(count_path) as stream:
        house_numbers_len = str(len(house_numbers))
        stream.write(util.to_bytes(house_numbers_len + "\n"))


def write_city_count_path(ctx: rust.PyContext, city_count_path: str, cities: Dict[str, Set[str]]) -> None:
    """Writes a daily .citycount file."""
    with ctx.get_file_system().open_write(city_count_path) as stream:
        # Locale-aware sort, by key.
        for key, value in sorted(cities.items(), key=lambda item: util.get_sort_key(item[0])):
            stream.write(util.to_bytes(key + "\t" + str(len(value)) + "\n"))


def update_stats_count(ctx: rust.PyContext, today: str) -> None:
    """Counts the # of all house numbers as of today."""
    statedir = ctx.get_abspath("workdir/stats")
    csv_path = os.path.join(statedir, "%s.csv" % today)
    if not ctx.get_file_system().path_exists(csv_path):
        return
    count_path = os.path.join(statedir, "%s.count" % today)
    city_count_path = os.path.join(statedir, "%s.citycount" % today)
    house_numbers = set()
    cities: Dict[str, Set[str]] = {}
    first = True
    valid_settlements = util.get_valid_settlements(ctx)
    with ctx.get_file_system().open_read(csv_path) as stream:
        for line_bytes in stream:
            line = util.from_bytes(line_bytes)
            if first:
                # Ignore the oneliner header.
                first = False
                continue
            # postcode, city name, street name, house number, user
            cells = line.split("\t")
            # Ignore last column, which is the user who touched the object last.
            house_numbers.add("\t".join(cells[:4]))
            city_key = util.get_city_key(cells[0], cells[1], valid_settlements)
            city_value = "\t".join(cells[2:4])
            if city_key in cities:
                cities[city_key].add(city_value)
            else:
                cities[city_key] = set([city_value])
    write_count_path(ctx, count_path, house_numbers)
    write_city_count_path(ctx, city_count_path, cities)


def update_stats_topusers(ctx: rust.PyContext, today: str) -> None:
    """Counts the top housenumber editors as of today."""
    statedir = ctx.get_abspath("workdir/stats")
    csv_path = os.path.join(statedir, "%s.csv" % today)
    if not ctx.get_file_system().path_exists(csv_path):
        return
    topusers_path = os.path.join(statedir, "%s.topusers" % today)
    usercount_path = os.path.join(statedir, "%s.usercount" % today)
    users: Dict[str, int] = {}
    with ctx.get_file_system().open_read(csv_path) as stream:
        for line_bytes in stream:
            line = util.from_bytes(line_bytes)
            # Only care about the last column.
            user = line[line.rfind("\t"):].strip()
            if user in users:
                users[user] += 1
            else:
                users[user] = 1
    with ctx.get_file_system().open_write(topusers_path) as stream:
        for user in sorted(users, key=cast(Callable[[str], int], users.get), reverse=True)[:20]:
            line = str(users[user]) + " " + user
            stream.write(util.to_bytes(line + "\n"))

    with ctx.get_file_system().open_write(usercount_path) as stream:
        stream.write(util.to_bytes(str(len(users)) + "\n"))


def update_stats_refcount(ctx: rust.PyContext, state_dir: str) -> None:
    """Performs the update of workdir/stats/ref.count."""
    count = 0
    with util.make_csv_io(ctx.get_file_system().open_read(ctx.get_ini().get_reference_citycounts_path())) as csv_stream:
        first = True
        for row in csv_stream.get_rows():
            if first:
                first = False
                continue

            count += int(row[1])

    with ctx.get_file_system().open_write(os.path.join(state_dir, "ref.count")) as stream:
        stream.write(util.to_bytes(str(count) + "\n"))


def update_stats(ctx: rust.PyContext, overpass: bool) -> None:
    """Performs the update of country-level stats."""

    # Fetch house numbers for the whole country.
    info("update_stats: start, updating whole-country csv")
    query = util.from_bytes(util.get_content(ctx.get_abspath("data/street-housenumbers-hungary.txt")))
    statedir = ctx.get_abspath("workdir/stats")
    os.makedirs(statedir, exist_ok=True)
    today = time.strftime("%Y-%m-%d")
    csv_path = os.path.join(statedir, "%s.csv" % today)

    if overpass:
        retry = 0
        while should_retry(retry):
            if retry > 0:
                info("update_stats: try #{}".format(retry))
            retry += 1
            overpass_sleep(ctx)
            try:
                response = rust.py_overpass_query(ctx, query)
            except OSError as err:
                info("update_stats: http error: {}".format(str(err)))
                continue
            with ctx.get_file_system().open_write(csv_path) as stream:
                stream.write(util.to_bytes(response))
            break

    update_stats_count(ctx, today)
    update_stats_topusers(ctx, today)
    update_stats_refcount(ctx, statedir)

    # Remove old CSV files as they are created daily and each is around 11M.
    current_time = time.time()
    for csv in glob.glob(os.path.join(statedir, "*.csv")):
        creation_time = os.path.getmtime(csv)
        if (current_time - creation_time) // (24 * 3600) >= 7:
            os.unlink(csv)
            info("update_stats: removed old {}".format(csv))

    info("update_stats: generating json")
    json_path = os.path.join(statedir, "stats.json")
    stats.generate_json(ctx, statedir, json_path)

    info("update_stats: end")


def our_main(ctx: rust.PyContext, relations: rust.PyRelations, mode: str, update: bool, overpass: bool) -> str:
    """Performs the actual nightly task."""
    try:
        if mode in ("all", "stats"):
            update_stats(ctx, overpass)
        if mode in ("all", "relations"):
            update_osm_streets(ctx, relations, update)
            update_osm_housenumbers(ctx, relations, update)
            update_ref_streets(ctx, relations, update)
            update_ref_housenumbers(ctx, relations, update)
            update_missing_streets(relations, update)
            update_missing_housenumbers(ctx, relations, update)
            update_additional_streets(relations, update)

        pid = str(os.getpid())
        with open("/proc/" + pid + "/status", "r") as stream:
            vm_peak = ""
            while True:
                line = stream.readline()
                if line.startswith("VmPeak:"):
                    vm_peak = line.strip()
                if vm_peak or not line:
                    info("our_main: {}".format(line.strip()))
                    break
    # pylint: disable=broad-except
    except Exception:  # pragma: no cover
        return traceback.format_exc()
    return ctx.get_unit().make_error()


def main(argv: List[str], _stdout: TextIO, ctx: rust.PyContext) -> None:
    """Commandline interface to this module."""

    relations = areas.make_relations(ctx)

    parser = argparse.ArgumentParser()
    parser.add_argument("--refcounty", type=str,
                        help="limit the list of relations to a given refcounty")
    parser.add_argument("--refsettlement", type=str,
                        help="limit the list of relations to a given refsettlement")
    parser.add_argument('--no-update', dest='update', action='store_false',
                        help="don't update existing state of relations")
    parser.add_argument("--mode", choices=["all", "stats", "relations"],
                        help="only perform the given sub-task or all of them")
    parser.add_argument("--no-overpass", dest="overpass", action="store_false",
                        help="when updating stats, don't perform any overpass update")
    parser.set_defaults(update=True, overpass=True, mode="relations")
    args = parser.parse_args(argv[1:])

    start = ctx.get_time().now()
    # Query inactive relations once a month.
    first_day_of_month = datetime.date.fromtimestamp(start).day == 1
    relations.activate_all(ctx.get_ini().get_cron_update_inactive() or first_day_of_month)
    relations.limit_to_refcounty(args.refcounty)
    relations.limit_to_refsettlement(args.refsettlement)
    err = our_main(ctx, relations, args.mode, args.update, args.overpass)
    if err:
        error("main: unhandled exception: {}".format(err))
    delta = ctx.get_time().now() - start
    info("main: finished in {}".format(str(datetime.timedelta(seconds=delta))))


if __name__ == "__main__":
    rust.py_setup_logging(context.make_context(""))
    main(sys.argv, sys.stdout, context.make_context(""))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
