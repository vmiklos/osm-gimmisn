#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cron module covers the cron module."""

import io
import unittest

import test_context

import cron
import util


class TestUpdateStatsTopusers(unittest.TestCase):
    """Tests update_stats_topuers()."""
    def test_happy(self) -> None:
        """Tests tha happy path."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()

        today_csv = util.to_bytes("""addr:postcode	addr:city	addr:street	addr:housenumber	@user
7677	Orfű	Dollár utca	1	mgpx
""")
        today_csv_value = io.BytesIO(today_csv)
        today_csv_value.__setattr__("close", lambda: None)
        today_topusers_value = io.BytesIO()
        today_topusers_value.__setattr__("close", lambda: None)
        today_usercount_value = io.BytesIO()
        today_usercount_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.csv"): today_csv_value,
            ctx.get_abspath("workdir/stats/2020-05-10.topusers"): today_topusers_value,
            ctx.get_abspath("workdir/stats/2020-05-10.usercount"): today_usercount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats_topusers(ctx, "2020-05-10")

        self.assertTrue(today_topusers_value.tell())
        self.assertTrue(today_usercount_value.tell())

    def test_no_csv(self) -> None:
        """Tests the case then the .csv is missing."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        today_topusers_value = io.BytesIO()
        today_topusers_value.__setattr__("close", lambda: None)
        today_usercount_value = io.BytesIO()
        today_usercount_value.__setattr__("close", lambda: None)
        file_system.set_hide_paths([ctx.get_abspath("workdir/stats/2020-05-10.csv")])
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.count"): today_topusers_value,
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): today_usercount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats_topusers(ctx, "2020-05-10")

        # No .csv, no .topusers or .usercount.
        self.assertFalse(today_topusers_value.tell())
        self.assertFalse(today_usercount_value.tell())


if __name__ == '__main__':
    unittest.main()
