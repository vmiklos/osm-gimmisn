#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Parses the Apache access log of osm-gimmisn for 1 month."""

from typing import Dict
from typing import List
from typing import Set
from typing import TextIO
import datetime
import os
import re
import sys

import unidecode

import areas
import config
import stats
import util


def is_complete_relation(relations: areas.Relations, relation_name: str) -> bool:
    """Does this relation have 100% house number coverage?"""
    assert relation_name in relations.get_names()

    relation = relations.get_relation(relation_name)
    if not os.path.exists(relation.get_files().get_housenumbers_percent_path()):
        return False

    percent = util.get_content(relation.get_files().get_housenumbers_percent_path()).decode("utf-8")
    return percent == "100.00"


def is_search_bot(line: str) -> bool:
    """Determine if 'line' has a user agent which looks like a search bot."""
    search_bots = [
        "AhrefsBot",
        "AhrefsBot",
        "CCBot",
        "Googlebot",
        "SemrushBot",
        "YandexBot",
        "bingbot",
    ]
    for search_bot in search_bots:
        if search_bot in line:
            return True
    return False


def get_frequent_relations(conf: config.Config, log_file: str) -> Set[str]:
    """Determine the top 20%: set of frequently visited relations."""
    counts: Dict[str, int] = {}
    with open(log_file, "r") as stream:
        # Example line:
        # a.b.c.d - - [01/Jul/2020:00:08:01 +0200] \
        # "GET /osm/street-housenumbers/budapest_12/update-result HTTP/1.1" 200 1747 "-" "Mozilla/5.0 ..."
        for line in stream.readlines():
            if is_search_bot(line):
                continue
            match = re.match('.*"GET ([^ ]+) .*', line)
            if not match:
                # Not GET.
                continue
            request_uri = match.group(1)
            if not request_uri.startswith("/osm"):
                continue

            # Expect: /osm/missing-streets/budapest_01/view-turbo
            tokens = request_uri.split("/")
            if len(tokens) != 5:
                continue
            relation_name = tokens[3]
            if relation_name in counts:
                counts[relation_name] += 1
            else:
                counts[relation_name] = 1
    count_list = sorted(counts.items(), key=lambda x: x[1], reverse=True)

    # Dump relations and their visit count to workdir for further inspection.
    with open(os.path.join(conf.get_workdir(), "frequent-relations.csv"), "w") as stream:
        for item in count_list:
            stream.write("{}\t{}\n".format(item[0], item[1]))

    relation_count = len(count_list)
    frequent_count = int(round(relation_count * 0.2))
    count_list = count_list[:frequent_count]
    frequent_relations: Set[str] = {i[0] for i in count_list}
    return frequent_relations


def get_relation_create_dates(conf: config.Config) -> Dict[str, datetime.date]:
    """Builds a name -> create_date dictionary for relations."""
    ret: Dict[str, datetime.date] = {}
    relations_path = conf.get_abspath("data/relations.yaml")
    process_stdout = conf.get_subprocess().run(["git", "blame", "--line-porcelain", relations_path], env={})
    timestamp = 0

    for line_bytes in process_stdout.splitlines():
        line = line_bytes.decode('utf-8')
        match = re.match("\t([^ :]+):", line)
        if match:
            name = match.group(1)
            ret[name] = datetime.date.fromtimestamp(timestamp)
            continue

        match = re.match("author-time ([0-9]+)", line)
        if match:
            timestamp = int(match.group(1))

    return ret


def is_relation_recently_added(conf: config.Config, create_dates: Dict[str, datetime.date], name: str) -> bool:
    """Decides if the given relation is recent, based on create_dates."""
    today = datetime.date.fromtimestamp(conf.get_time().now())
    month_ago = today - datetime.timedelta(days=30)
    return name in create_dates and create_dates[name] > month_ago


def check_top_edited_relations(conf: config.Config, frequent_relations: Set[str], workdir: str) -> None:
    """
    Update frequent_relations based on get_topcities():
    1) The top 5 edited cities count as frequent, even if they have ~no visitors.
    2) If a relation got <5 house numbers in the last 30 days, then they are not frequent, even with
    lots of visitors.
    """
    # List of 'city name' <-> '# of new house numbers' pairs.
    topcities = stats.get_topcities(conf, os.path.join(workdir, "stats"))
    topcities = [(unidecode.unidecode(city[0]), city[1]) for city in topcities]
    # Top 5: these should be frequent.
    for city in topcities[:5]:
        frequent_relations.add(city[0])
    # Bottom: anything with <5 new house numbers is not frequent.
    bottomcities = [city for city in topcities if city[1] < 5]
    for city in bottomcities:
        if city[0] in frequent_relations:
            frequent_relations.remove(city[0])


def main(argv: List[str], stdout: TextIO, conf: config.Config) -> None:
    """Commandline interface."""
    log_file = argv[1]

    relation_create_dates: Dict[str, datetime.date] = get_relation_create_dates(conf)

    relations = areas.Relations(conf)
    frequent_relations = get_frequent_relations(conf, log_file)
    check_top_edited_relations(conf, frequent_relations, conf.get_workdir())

    # Now suggest what to change.
    removals = 0
    additions = 0
    for relation_name in relations.get_names():
        relation = relations.get_relation(relation_name)
        actual = relation.get_config().is_active()
        expected = relation_name in frequent_relations and not is_complete_relation(relations, relation_name)
        if actual != expected:
            if actual:
                if not is_relation_recently_added(conf, relation_create_dates, relation_name):
                    stdout.write("data/relation-{}.yaml: set inactive: true\n".format(relation_name))
                    removals += 1
            else:
                stdout.write("data/relation-{}.yaml: set inactive: false\n".format(relation_name))
                additions += 1
    stdout.write("Suggested {} removals and {} additions.\n".format(removals, additions))


if __name__ == '__main__':
    main(sys.argv, sys.stdout, config.Config(""))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
