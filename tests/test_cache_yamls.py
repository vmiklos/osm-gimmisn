#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cache_yamls module covers the cache_yamls module."""

import io
import json
import unittest

import test_context

import areas
import rust


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        cache_path = "tests/data/yamls.cache"
        argv = ["", "data", "workdir"]
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([cache_path])
        cache_value = io.BytesIO()
        cache_value.__setattr__("close", lambda: None)
        stats_value = io.BytesIO()
        stats_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("data/yamls.cache"): cache_value,
            ctx.get_abspath("workdir/stats/relations.json"): stats_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        rust.py_cache_yamls_main(argv, ctx)
        # Just assert that the result is created, the actual content is validated by the other
        # tests.
        self.assertTrue(cache_value.tell())

        relation_ids_path = "tests/workdir/stats/relations.json"
        relation_ids = []
        with open(relation_ids_path) as stream:
            relation_ids = json.load(stream)
        relations = areas.make_relations(ctx)
        osmids = sorted([relation.get_config().get_osmrelation() for relation in relations.get_relations()])
        self.assertEqual(relation_ids, sorted(set(osmids)))


if __name__ == '__main__':
    unittest.main()
