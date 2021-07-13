#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_parse_access_log module covers the parse_access_log module."""

from typing import List
from typing import Set
from typing import Tuple
import io
import unittest
import unittest.mock

import test_context

import areas
import context
import parse_access_log


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        argv = ["", "tests/mock/access_log"]
        buf = io.StringIO()
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        ctx.set_time(test_context.make_test_time())
        relations_path = ctx.get_abspath("data/relations.yaml")
        # 2020-05-09, so this will be recent
        outputs = {
            "git blame --line-porcelain " + relations_path: b"""
author-time 1588975200
\tujbuda:
"""
        }
        subprocess = test_context.TestSubprocess(outputs)
        ctx.set_subprocess(subprocess)
        parse_access_log.main(argv, buf, ctx)

        buf.seek(0)
        actual = buf.read()
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
        def mock_get_topcities(_ctx: context.Context, _src_root: str) -> List[Tuple[str, int]]:
            return [
                ("foo", 1000),
                ("city1", 1000),
                ("city2", 1000),
                ("city3", 1000),
                ("city4", 1000),
                ("bar", 2),
                ("baz", 2)
            ]
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        with unittest.mock.patch('stats.get_topcities', mock_get_topcities):
            frequent_relations: Set[str] = {"foo", "bar"}
            parse_access_log.check_top_edited_relations(ctx, frequent_relations)
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
        relations = areas.Relations(ctx)
        self.assertFalse(parse_access_log.is_complete_relation(relations, "gazdagret"))


if __name__ == '__main__':
    unittest.main()
