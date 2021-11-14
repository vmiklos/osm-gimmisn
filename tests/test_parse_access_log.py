#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_parse_access_log module covers the parse_access_log module."""

import io
import unittest

import test_context

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


if __name__ == '__main__':
    unittest.main()
