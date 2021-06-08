#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

from typing import Any
from typing import Callable
from typing import Dict
from typing import Set
from typing import cast
import argparse
import datetime
import glob
import locale
import logging
import os
import time
import traceback
import urllib.error

import areas
import cache
import config
import i18n
import overpass_query
import stats
import util


def get_date_prefix() -> str:
    """Generates the current date as a log prefix."""
    return time.strftime("%Y-%m-%d %H:%M:%S")


def info(msg: str, *args: Any, **kwargs: Any) -> None:
    """Wrapper around logging.info()."""
    logging.info(get_date_prefix() + " INFO " + msg, *args, **kwargs)


def error(msg: str, *args: Any, **kwargs: Any) -> None:
    """Wrapper around logging.error()."""
    logging.error(get_date_prefix() + " ERROR" + msg, *args, **kwargs)


def overpass_sleep(conf: config.Config) -> None:
    """Sleeps to respect overpass rate limit."""
    while True:
        sleep = overpass_query.overpass_query_need_sleep(conf)
        if not sleep:
            break
        info("overpass_sleep: waiting for %s seconds", sleep)
        time.sleep(sleep)


def should_retry(retry: int) -> bool:
    """Decides if we should retry a query or not."""
    return retry < 20


def update_osm_streets(conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the OSM street list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_osm_streets_path()):
            continue
        info("update_osm_streets: start: %s", relation_name)
        retry = 0
        while should_retry(retry):
            if retry > 0:
                info("update_osm_streets: try #%s", retry)
            retry += 1
            try:
                overpass_sleep(conf)
                query = relation.get_osm_streets_query()
                relation.get_files().write_osm_streets(overpass_query.overpass_query(conf, query))
                break
            except urllib.error.HTTPError as http_error:
                info("update_osm_streets: http error: %s", str(http_error))
        info("update_osm_streets: end: %s", relation_name)


def update_osm_housenumbers(conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the OSM housenumber list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_osm_housenumbers_path()):
            continue
        info("update_osm_housenumbers: start: %s", relation_name)
        retry = 0
        while should_retry(retry):
            if retry > 0:
                info("update_osm_housenumbers: try #%s", retry)
            retry += 1
            try:
                overpass_sleep(conf)
                query = relation.get_osm_housenumbers_query()
                relation.get_files().write_osm_housenumbers(overpass_query.overpass_query(conf, query))
                break
            except urllib.error.HTTPError as http_error:
                info("update_osm_housenumbers: http error: %s", str(http_error))
        info("update_osm_housenumbers: end: %s", relation_name)


def update_ref_housenumbers(conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the reference housenumber list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_ref_housenumbers_path()):
            continue
        references = conf.get_reference_housenumber_paths()
        streets = relation.get_config().should_check_missing_streets()
        if streets == "only":
            continue

        info("update_ref_housenumbers: start: %s", relation_name)
        relation.write_ref_housenumbers(references)
        info("update_ref_housenumbers: end: %s", relation_name)


def update_ref_streets(conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the reference street list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_ref_streets_path()):
            continue
        reference = conf.get_reference_street_path()
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        info("update_ref_streets: start: %s", relation_name)
        relation.write_ref_streets(reference)
        info("update_ref_streets: end: %s", relation_name)


def update_missing_housenumbers(conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the relation's house number coverage stats."""
    info("update_missing_housenumbers: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_housenumbers_percent_path()):
            continue
        streets = relation.get_config().should_check_missing_streets()
        if streets == "only":
            continue

        orig_language = i18n.get_language()
        relation.write_missing_housenumbers()
        for language in ["en", "hu"]:
            i18n.set_language(language)
            cache.get_missing_housenumbers_html(conf, relation)
        i18n.set_language(orig_language)
        cache.get_missing_housenumbers_txt(relation)
    info("update_missing_housenumbers: end")


def update_missing_streets(_conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the relation's street coverage stats."""
    info("update_missing_streets: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_streets_percent_path()):
            continue
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        relation.write_missing_streets()
    info("update_missing_streets: end")


def update_additional_streets(_conf: config.Config, relations: areas.Relations, update: bool) -> None:
    """Update the relation's "additional streets" stats."""
    info("update_additional_streets: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_streets_additional_count_path()):
            continue
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        relation.write_additional_streets()
    info("update_additional_streets: end")


def write_count_path(count_path: str, house_numbers: Set[str]) -> None:
    """Writes a daily .count file."""
    with open(count_path, "w") as stream:
        house_numbers_len = str(len(house_numbers))
        stream.write(house_numbers_len + "\n")


def write_city_count_path(city_count_path: str, cities: Dict[str, Set[str]]) -> None:
    """Writes a daily .citycount file."""
    with open(city_count_path, "w") as stream:
        # Locale-aware sort, by key.
        for key, value in sorted(cities.items(), key=lambda item: locale.strxfrm(item[0])):
            stream.write(key + "\t" + str(len(value)) + "\n")


def update_stats_count(conf: config.Config, today: str) -> None:
    """Counts the # of all house numbers as of today."""
    statedir = config.get_abspath("workdir/stats")
    csv_path = os.path.join(statedir, "%s.csv" % today)
    count_path = os.path.join(statedir, "%s.count" % today)
    city_count_path = os.path.join(statedir, "%s.citycount" % today)
    house_numbers = set()
    cities: Dict[str, Set[str]] = {}
    first = True
    valid_settlements = util.get_valid_settlements(conf)
    with open(csv_path, "r") as stream:
        for line in stream.readlines():
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
    write_count_path(count_path, house_numbers)
    write_city_count_path(city_count_path, cities)


def update_stats_topusers(today: str) -> None:
    """Counts the top housenumber editors as of today."""
    statedir = config.get_abspath("workdir/stats")
    csv_path = os.path.join(statedir, "%s.csv" % today)
    topusers_path = os.path.join(statedir, "%s.topusers" % today)
    usercount_path = os.path.join(statedir, "%s.usercount" % today)
    users: Dict[str, int] = {}
    with open(csv_path, "r") as stream:
        for line in stream.readlines():
            # Only care about the last column.
            user = line[line.rfind("\t"):].strip()
            if user in users:
                users[user] += 1
            else:
                users[user] = 1
    with open(topusers_path, "w") as stream:
        for user in sorted(users, key=cast(Callable[[str], int], users.get), reverse=True)[:20]:
            line = str(users[user]) + " " + user
            stream.write(line + "\n")

    with open(usercount_path, "w") as stream:
        stream.write(str(len(users)) + "\n")


def update_stats_refcount(conf: config.Config, state_dir: str) -> None:
    """Performs the update of workdir/stats/ref.count."""
    count = 0
    with open(conf.get_reference_citycounts_path(), "r") as stream:
        first = True
        for line in stream.readlines():
            if first:
                first = False
                continue

            cells = line.strip().split('\t')
            if len(cells) < 2:
                continue

            count += int(cells[1])

    with open(os.path.join(state_dir, "ref.count"), "w") as stream:
        stream.write(str(count) + "\n")


def update_stats(conf: config.Config, overpass: bool) -> None:
    """Performs the update of country-level stats."""

    # Fetch house numbers for the whole country.
    info("update_stats: start, updating whole-country csv")
    query = util.get_content(config.get_abspath("data/street-housenumbers-hungary.txt")).decode("utf-8")
    statedir = config.get_abspath("workdir/stats")
    os.makedirs(statedir, exist_ok=True)
    today = time.strftime("%Y-%m-%d")
    csv_path = os.path.join(statedir, "%s.csv" % today)

    if overpass:
        retry = 0
        while should_retry(retry):
            if retry > 0:
                info("update_stats: try #%s", retry)
            retry += 1
            try:
                overpass_sleep(conf)
                response = overpass_query.overpass_query(conf, query)
                with open(csv_path, "w") as stream:
                    stream.write(response)
                break
            except urllib.error.HTTPError as http_error:
                info("update_stats: http error: %s", str(http_error))

    update_stats_count(conf, today)
    update_stats_topusers(today)
    update_stats_refcount(conf, statedir)

    # Remove old CSV files as they are created daily and each is around 11M.
    current_time = time.time()
    for csv in glob.glob(os.path.join(statedir, "*.csv")):
        creation_time = os.path.getmtime(csv)
        if (current_time - creation_time) // (24 * 3600) >= 7:
            os.unlink(csv)
            info("update_stats: removed old %s", csv)

    info("update_stats: generating json")
    json_path = os.path.join(statedir, "stats.json")
    with open(json_path, "w") as stream:
        stats.generate_json(statedir, stream)

    info("update_stats: end")


def our_main(conf: config.Config, relations: areas.Relations, mode: str, update: bool, overpass: bool) -> None:
    """Performs the actual nightly task."""
    if mode in ("all", "stats"):
        update_stats(conf, overpass)
    if mode in ("all", "relations"):
        update_osm_streets(conf, relations, update)
        update_osm_housenumbers(conf, relations, update)
        update_ref_streets(conf, relations, update)
        update_ref_housenumbers(conf, relations, update)
        update_missing_streets(conf, relations, update)
        update_missing_housenumbers(conf, relations, update)
        update_additional_streets(conf, relations, update)

    pid = str(os.getpid())
    with open("/proc/" + pid + "/status", "r") as stream:
        vm_peak = ""
        while True:
            line = stream.readline()
            if line.startswith("VmPeak:"):
                vm_peak = line.strip()
            if vm_peak or not line:
                info("our_main: %s", line.strip())
                break


def main() -> None:
    """Commandline interface to this module."""

    conf = config.make_config()
    util.set_locale(conf)

    workdir = conf.get_workdir()
    relations = areas.Relations(workdir)
    logpath = os.path.join(workdir, "cron.log")
    logging.basicConfig(filename=logpath,
                        level=logging.INFO,
                        format='%(message)s')
    handler = logging.StreamHandler()
    logging.getLogger().addHandler(handler)

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
    args = parser.parse_args()

    start = time.time()
    # Query inactive relations once a month.
    first_day_of_month = time.localtime(start).tm_mday == 1
    relations.activate_all(conf.get_cron_update_inactive() or first_day_of_month)
    relations.limit_to_refcounty(args.refcounty)
    relations.limit_to_refsettlement(args.refsettlement)
    try:
        our_main(conf, relations, args.mode, args.update, args.overpass)
    # pylint: disable=broad-except
    except Exception:
        error("main: unhandled exception: %s", traceback.format_exc())
    delta = time.time() - start
    info("main: finished in %s", str(datetime.timedelta(seconds=delta)))
    logging.getLogger().removeHandler(handler)
    logging.shutdown()


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
