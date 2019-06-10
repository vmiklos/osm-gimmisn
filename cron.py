#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

import configparser
import logging
import os
import time

import helpers
import overpass_query


def get_srcdir(subdir=""):
    """Gets the directory which is tracked in version control."""
    dirname = os.path.dirname(__file__)

    if subdir:
        dirname = os.path.join(dirname, subdir)

    return dirname


def update_streets(workdir):
    """Update the existing street list of all relations."""
    datadir = get_srcdir("data")
    relations = helpers.get_relations(datadir)
    for relation in relations.keys():
        logging.info("update_streets: start: %s", relation)
        sleep = overpass_query.overpass_query_need_sleep()
        if sleep:
            logging.info("update_streets: sleeping for %s seconds", sleep)
            time.sleep(sleep)
        query = helpers.get_streets_query(datadir, relations, relation)
        helpers.write_streets_result(workdir, relation, overpass_query.overpass_query(query))
        logging.info("update_streets: end: %s", relation)


def main():
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

    update_streets(workdir)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
