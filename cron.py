#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

import datetime
import logging
import os
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


def update_osm_streets(relations: areas.Relations) -> None:
    """Update the OSM street list of all relations."""
    for relation_name in relations.get_active_names():
        logging.info("update_osm_streets: start: %s", relation_name)
        relation = relations.get_relation(relation_name)
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


def update_osm_housenumbers(relations: areas.Relations) -> None:
    """Update the OSM housenumber list of all relations."""
    for relation_name in relations.get_active_names():
        logging.info("update_osm_housenumbers: start: %s", relation_name)
        retry = 0
        while should_retry(retry):
            if retry > 0:
                logging.info("update_osm_housenumbers: try #%s", retry)
            retry += 1
            try:
                overpass_sleep()
                relation = relations.get_relation(relation_name)
                query = relation.get_osm_housenumbers_query()
                relation.get_files().write_osm_housenumbers(overpass_query.overpass_query(query))
                break
            except urllib.error.HTTPError as http_error:
                logging.info("update_osm_housenumbers: http error: %s", str(http_error))
        logging.info("update_osm_housenumbers: end: %s", relation_name)


def update_ref_housenumbers(relations: areas.Relations) -> None:
    """Update the reference housenumber list of all relations."""
    for relation_name in relations.get_active_names():
        logging.info("update_ref_housenumbers: start: %s", relation_name)
        relation = relations.get_relation(relation_name)
        reference = util.Config.get().get('wsgi', 'reference_housenumbers').strip().split(' ')
        streets = relation.get_config().should_check_missing_streets()
        if streets == "only":
            continue

        relation.write_ref_housenumbers(reference)
        logging.info("update_ref_housenumbers: end: %s", relation_name)


def update_ref_streets(relations: areas.Relations) -> None:
    """Update the reference street list of all relations."""
    for relation_name in relations.get_active_names():
        logging.info("update_ref_streets: start: %s", relation_name)
        relation = relations.get_relation(relation_name)
        reference = util.Config.get().get('wsgi', 'reference_street').strip()
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        relation.write_ref_streets(reference)
        logging.info("update_ref_streets: end: %s", relation_name)


def update_missing_housenumbers(relations: areas.Relations) -> None:
    """Update the relation's house number coverage stats."""
    logging.info("update_missing_housenumbers: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        streets = relation.get_config().should_check_missing_streets()
        if streets == "only":
            continue

        relation.write_missing_housenumbers()
    logging.info("update_missing_housenumbers: end")


def update_missing_streets(relations: areas.Relations) -> None:
    """Update the relation's street coverage stats."""
    logging.info("update_missing_streets: start")
    for relation_name in relations.get_active_names():
        relation = relations.get_relation(relation_name)
        streets = relation.get_config().should_check_missing_streets()
        if streets == "no":
            continue

        relation.write_missing_streets()
    logging.info("update_missing_streets: end")


def our_main(relations: areas.Relations) -> None:
    """Performs the actual nightly task."""
    update_osm_streets(relations)
    update_osm_housenumbers(relations)
    update_ref_streets(relations)
    update_ref_housenumbers(relations)
    update_missing_streets(relations)
    update_missing_housenumbers(relations)


def main() -> None:
    """Commandline interface to this module."""

    config = util.Config.get()
    util.set_locale(config)

    workdir = util.Config.get_workdir()
    relations = areas.Relations(workdir)
    logpath = os.path.join(workdir, "cron.log")
    logging.basicConfig(filename=logpath,
                        level=logging.INFO,
                        format='%(asctime)s %(levelname)s %(message)s',
                        datefmt='%Y-%m-%d %H:%M:%S')
    handler = logging.StreamHandler()
    logging.getLogger().addHandler(handler)

    start = time.time()
    # Query inactive relations once a month.
    relations.activate_all(time.localtime(start).tm_mday == 1)
    try:
        our_main(relations)
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
