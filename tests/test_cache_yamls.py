#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cache_yamls module covers the cache_yamls module."""

import json
import os
import unittest

import test_context

import areas
import cache_yamls


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        cache_path = "tests/data/yamls.cache"
        if os.path.exists(cache_path):
            os.remove(cache_path)
        argv = ["", "data", "workdir"]
        ctx = test_context.make_test_context()
        cache_yamls.main(argv, ctx)
        # Just assert that the result is created, the actual content is validated by the other
        # tests.
        self.assertTrue(os.path.exists(cache_path))

        relation_ids_path = "tests/workdir/stats/relations.json"
        relation_ids = []
        with open(relation_ids_path) as stream:
            relation_ids = json.load(stream)
        relations = areas.Relations(ctx)
        osmids = sorted([relation.get_config().get_osmrelation() for relation in relations.get_relations()])
        self.assertEqual(relation_ids, sorted(set(osmids)))


if __name__ == '__main__':
    unittest.main()
