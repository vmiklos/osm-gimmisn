#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_parse_access_log module covers the parse_access_log module."""

from typing import Set
import io
import unittest

import test_context

import areas
import parse_access_log


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        argv = ["", "tests/mock/access_log"]
        buf = io.BytesIO()
        buf.__setattr__("close", lambda: None)
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        relations_path = ctx.get_abspath("data/relations.yaml")
        # 2020-05-09, so this will be recent
        outputs = {
            "git blame --line-porcelain " + relations_path: """
author-time 1588975200
\tujbuda:
"""
        }
        subprocess = test_context.TestSubprocess(outputs)
        ctx.set_subprocess(subprocess)
        parse_access_log.main(argv, buf, ctx)

        buf.seek(0)
        actual = buf.read().decode("utf-8")
        self.assertIn("data/relation-inactiverelation.yaml: set inactive: false\n", actual)
        self.assertIn("data/relation-gazdagret.yaml: set inactive: true\n", actual)
        self.assertNotIn("data/relation-nosuchrelation.yaml: set inactive: ", actual)

        # This is not in the output because it's considered as a recent relation.
        self.assertNotIn("data/relation-ujbuda.yaml: set inactive: ", actual)

        # This is not in the output as it's not a valid relation name.
        self.assertNotIn("budafokxxx", actual)

        # This is not in the output as it's a search bot, so such visits don't count.
        # Also, if this would be not ignored, it would push 'inactiverelation' out of the active
        # relation list.
        self.assertNotIn("gyomaendrod", actual)


class TestCheckTopEditedRelations(unittest.TestCase):
    """Tests check_top_edited_relations()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        file_system = test_context.TestFileSystem()
        old_citycount = b"""foo\t0
city1\t0
city2\t0
city3\t0
city4\t0
bar\t0
baz\t0
"""
        old_citycount_value = io.BytesIO(old_citycount)
        old_citycount_value.__setattr__("close", lambda: None)
        new_citycount = b"""foo\t1000
city1\t1000
city2\t1000
city3\t1000
city4\t1000
bar\t2
baz\t2
"""
        new_citycount_value = io.BytesIO(new_citycount)
        new_citycount_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-04-10.citycount"): old_citycount_value,
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): new_citycount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        frequent_relations: Set[str] = {"foo", "bar"}
        frequent_relations = parse_access_log.check_top_edited_relations(ctx, frequent_relations)

        self.assertIn("foo", frequent_relations)
        self.assertIn("city1", frequent_relations)
        self.assertIn("city2", frequent_relations)
        self.assertIn("city3", frequent_relations)
        self.assertIn("city4", frequent_relations)
        self.assertNotIn("bar", frequent_relations)
        self.assertNotIn("baz", frequent_relations)


class TestIsCompleteRelation(unittest.TestCase):
    """Tests is_complete_relation()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        self.assertFalse(parse_access_log.is_complete_relation(relations, "gazdagret"))


if __name__ == '__main__':
    unittest.main()
