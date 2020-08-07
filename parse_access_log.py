#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Parses the Apache access log of osm-gimmisn for 1 month."""

from typing import Dict
from typing import Set
import re
import sys

import areas
import config


def get_frequent_relations(log_file: str) -> Set[str]:
    """Determine the top 20%: set of frequently visited relations."""
    counts: Dict[str, int] = {}
    with open(log_file, "r") as stream:
        # Example line:
        # a.b.c.d - - [01/Jul/2020:00:08:01 +0200] \
        # "GET /osm/street-housenumbers/budapest_12/update-result HTTP/1.1" 200 1747 "-" "Mozilla/5.0 ..."
        for line in stream.readlines():
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
    relation_count = len(count_list)
    frequent_count = int(round(relation_count * 0.2))
    count_list = count_list[:frequent_count]
    frequent_relations: Set[str] = {i[0] for i in count_list}
    return frequent_relations


def main() -> None:
    """Commandline interface."""
    log_file = sys.argv[1]

    frequent_relations = get_frequent_relations(log_file)

    # Now suggest what to change.
    relations = areas.Relations(config.Config.get_workdir())
    removals = 0
    additions = 0
    for relation_name in relations.get_names():
        relation = relations.get_relation(relation_name)
        actual = relation.get_config().is_active()
        expected = relation_name in frequent_relations
        if actual != expected:
            if actual:
                print("data/relation-{}.yaml: set inactive: true".format(relation_name))
                removals += 1
            else:
                print("data/relation-{}.yaml: set inactive: false".format(relation_name))
                additions += 1
    print("Suggested {} removals and {} additions.".format(removals, additions))


if __name__ == '__main__':
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
