#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

import argparse
import datetime
import logging
import os
import subprocess
import time
import traceback
import urllib.error

import areas
import overpass_query
import util


def overpass_sleep() -> None:
    """Sleeps to respect overpass rate limit."""
    while True:
        sleep = overpass_query.overpass_query_need_sleep()
        if not sleep:
            break
        logging.info("overpass_sleep: waiting for %s seconds", sleep)
        time.sleep(sleep)


def should_retry(retry: int) -> bool:
    """Decides if we should retry a query or not."""
    return retry < 20


def update_osm_streets(relations: areas.Relations, update: bool) -> None:
    """Update the OSM street list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_osm_streets_path()):
            continue
        logging.info("update_osm_streets: start: %s", relation_name)
        retry = 0
        while should_retry(retry):
            if retry > 0:
                logging.info("update_osm_streets: try #%s", retry)
            retry += 1
            try:
                overpass_sleep()
                query = relation.get_osm_streets_query()
                relation.get_files().write_osm_streets(overpass_query.overpass_query(query))
                break
            except urllib.error.HTTPError as http_error:
                logging.info("update_osm_streets: http error: %s", str(http_error))
        logging.info("update_osm_streets: end: %s", relation_name)


def update_osm_housenumbers(relations: areas.Relations, update: bool) -> None:
    """Update the OSM housenumber list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_osm_housenumbers_path()):
            continue
        logging.info("update_osm_housenumbers: start: %s", relation_name)
        retry = 0
        while should_retry(retry):
            if retry > 0:
                logging.info("update_osm_housenumbers: try #%s", retry)
            retry += 1
            try:
                overpass_sleep()
                query = relation.get_osm_housenumbers_query()
                relation.get_files().write_osm_housenumbers(overpass_query.overpass_query(query))
                break
            except urllib.error.HTTPError as http_error:
                logging.info("update_osm_housenumbers: http error: %s", str(http_error))
        logging.info("update_osm_housenumbers: end: %s", relation_name)


def update_ref_housenumbers(relations: areas.Relations, update: bool) -> None:
    """Update the reference housenumber list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_ref_housenumbers_path()):
            continue
        references = util.Config.get_reference_housenumber_paths()
        streets = relation.get_config().should_check_missing_streets()
        if streets == "only":
            continue

        logging.info("update_ref_housenumbers: start: %s", relation_name)
        relation.write_ref_housenumbers(references)
        logging.info("update_ref_housenumbers: end: %s", relation_name)


def update_ref_streets(relations: areas.Relations, update: bool) -> None:
    """Update the reference street list of all relations."""
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_ref_streets_path()):
            continue
        reference = util.Config.get_reference_street_path()
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        logging.info("update_ref_streets: start: %s", relation_name)
        relation.write_ref_streets(reference)
        logging.info("update_ref_streets: end: %s", relation_name)


def update_missing_housenumbers(relations: areas.Relations, update: bool) -> None:
    """Update the relation's house number coverage stats."""
    logging.info("update_missing_housenumbers: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_housenumbers_percent_path()):
            continue
        streets = relation.get_config().should_check_missing_streets()
        if streets == "only":
            continue

        relation.write_missing_housenumbers()
    logging.info("update_missing_housenumbers: end")


def update_missing_streets(relations: areas.Relations, update: bool) -> None:
    """Update the relation's street coverage stats."""
    logging.info("update_missing_streets: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        if not update and os.path.exists(relation.get_files().get_streets_percent_path()):
            continue
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        relation.write_missing_streets()
    logging.info("update_missing_streets: end")


def update_stats() -> None:
    """Performs the update of country-level stats."""

    # Fetch house numbers for the whole country.
    logging.info("update_stats: start, updating whole-country csv")
    query = util.get_content(util.get_abspath("data/street-housenumbers-hungary.txt"))
    statedir = util.get_abspath("workdir/stats")
    os.makedirs(statedir, exist_ok=True)
    today = time.strftime("%Y-%m-%d")
    csv_path = os.path.join(statedir, "%s.csv" % today)

    retry = 0
    while should_retry(retry):
        if retry > 0:
            logging.info("update_stats: try #%s", retry)
        retry += 1
        try:
            overpass_sleep()
            response = overpass_query.overpass_query(query)
            with open(csv_path, "w") as stream:
                stream.write(response)
            break
        except urllib.error.HTTPError as http_error:
            logging.info("update_stats: http error: %s", str(http_error))

    # Shell part.
    logging.info("update_stats: executing the shell part")
    subprocess.run([util.get_abspath("stats-daily.sh")], check=True)

    logging.info("update_stats: end")


def our_main(relations: areas.Relations, mode: str, update: bool) -> None:
    """Performs the actual nightly task."""
    if mode in ("all", "stats"):
        update_stats()
    if mode in ("all", "relations"):
        update_osm_streets(relations, update)
        update_osm_housenumbers(relations, update)
        update_ref_streets(relations, update)
        update_ref_housenumbers(relations, update)
        update_missing_streets(relations, update)
        update_missing_housenumbers(relations, update)


def main() -> None:
    """Commandline interface to this module."""

    util.set_locale()

    workdir = util.Config.get_workdir()
    relations = areas.Relations(workdir)
    logpath = os.path.join(workdir, "cron.log")
    logging.basicConfig(filename=logpath,
                        level=logging.INFO,
                        format='%(asctime)s %(levelname)s %(message)s',
                        datefmt='%Y-%m-%d %H:%M:%S')
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
    parser.set_defaults(update=True, mode="relations")
    args = parser.parse_args()

    start = time.time()
    relations.activate_all(util.Config.get_cron_update_inactive())
    relations.limit_to_refcounty(args.refcounty)
    relations.limit_to_refsettlement(args.refsettlement)
    try:
        our_main(relations, args.mode, args.update)
    # pylint: disable=broad-except
    except Exception:
        logging.error("main: unhandled exception: %s", traceback.format_exc())
    delta = time.time() - start
    logging.info("main: finished in %s", str(datetime.timedelta(seconds=delta)))
    logging.getLogger().removeHandler(handler)
    logging.shutdown()


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
