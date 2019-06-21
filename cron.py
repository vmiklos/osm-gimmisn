#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

import configparser
import datetime
import logging
import os
import time

import helpers
import overpass_query


def get_srcdir(subdir: str = "") -> str:
    """Gets the directory which is tracked in version control."""
    dirname = os.path.dirname(__file__)

    if subdir:
        dirname = os.path.join(dirname, subdir)

    return dirname


def overpass_sleep() -> None:
    """Sleeps to respect overpass rate limit."""
    while True:
        sleep = overpass_query.overpass_query_need_sleep()
        if not sleep:
            break
        logging.info("overpass_sleep: waiting for %s seconds", sleep)
        time.sleep(sleep)


def update_streets(workdir: str) -> None:
    """Update the existing street list of all relations."""
    datadir = get_srcdir("data")
    relations = helpers.get_relations(datadir)
    for relation in relations.keys():
        logging.info("update_streets: start: %s", relation)
        overpass_sleep()
        query = helpers.get_streets_query(datadir, relations, relation)
        helpers.write_streets_result(workdir, relation, overpass_query.overpass_query(query))
        logging.info("update_streets: end: %s", relation)


def update_street_housenumbers(workdir: str) -> None:
    """Update the existing street housenumber list of all relations."""
    datadir = get_srcdir("data")
    relations = helpers.get_relations(datadir)
    for relation in relations.keys():
        logging.info("update_street_housenumbers: start: %s", relation)
        overpass_sleep()
        query = helpers.get_street_housenumbers_query(datadir, relations, relation)
        helpers.write_street_housenumbers(workdir, relation, overpass_query.overpass_query(query))
        logging.info("update_street_housenumbers: end: %s", relation)


def update_suspicious_streets_stats(workdir: str) -> None:
    """Update the relation's house number coverage stats."""
    datadir = get_srcdir("data")
    relations = helpers.get_relations(datadir)
    logging.info("update_suspicious_streets_stats: start")
    for relation in relations.keys():
        streets = helpers.get_relation_missing_streets(datadir, relation)
        if streets == "only":
            continue

        helpers.write_suspicious_streets_result(datadir, workdir, relation)
    logging.info("update_suspicious_streets_stats: end")


def update_missing_streets_stats(workdir: str) -> None:
    """Update the relation's street coverage stats."""
    datadir = get_srcdir("data")
    relations = helpers.get_relations(datadir)
    logging.info("update_missing_streets_stats: start")
    for relation in relations.keys():
        streets = helpers.get_relation_missing_streets(datadir, relation)
        if streets == "no":
            continue

        helpers.write_missing_relations_result(datadir, workdir, relation)
    logging.info("update_missing_streets_stats: end")


def main() -> None:
    """Commandline interface to this module."""

    config = configparser.ConfigParser()
    config_path = os.path.join(os.path.dirname(__file__), "wsgi.ini")
    config.read(config_path)

    workdir = config.get('wsgi', 'workdir').strip()
    logpath = os.path.join(workdir, "cron.log")
    logging.basicConfig(filename=logpath,
                        level=logging.INFO,
                        format='%(asctime)s %(levelname)s %(message)s',
                        datefmt='%Y-%m-%d %H:%M:%S')
    logging.getLogger().addHandler(logging.StreamHandler())

    start = time.time()
    update_streets(workdir)
    update_street_housenumbers(workdir)
    update_suspicious_streets_stats(workdir)
    update_missing_streets_stats(workdir)
    delta = time.time() - start
    logging.info("main: finished in %s", str(datetime.timedelta(seconds=delta)))


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
