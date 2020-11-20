#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cache_yamls module caches YAML files from the data/ directory."""

from typing import Any
from typing import Dict
import glob
import json
import os
import pickle
import sys
import yaml

import config


def main() -> None:
    """Commandline interface to this module."""

    cache: Dict[str, Any] = {}
    datadir = config.get_abspath(sys.argv[1])
    for yaml_path in glob.glob(os.path.join(datadir, "*.yaml")):
        with open(yaml_path) as yaml_stream:
            cache_key = os.path.relpath(yaml_path, datadir)
            cache[cache_key] = yaml.safe_load(yaml_stream)

    cache_path = os.path.join(datadir, "yamls.pickle")
    with open(cache_path, "wb") as cache_stream:
        pickle.dump(cache, cache_stream)

    workdir = config.get_abspath(sys.argv[2])
    yaml_path = os.path.join(datadir, "relations.yaml")
    relation_ids = []
    with open(yaml_path) as stream:
        relations = yaml.safe_load(stream)
        for _key, value in relations.items():
            relation_ids.append(value["osmrelation"])
    relation_ids = sorted(set(relation_ids))
    statsdir = os.path.join(workdir, "stats")
    os.makedirs(statsdir, exist_ok=True)
    with open(os.path.join(statsdir, "relations.json"), "w") as stream:
        json.dump(relation_ids, stream)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
