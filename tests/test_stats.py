#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_stats module covers the stats module."""

from typing import Any
from typing import Dict
import calendar
import datetime
import os
import unittest

import test_context

import api
import stats


def make_test_time_old() -> api.Time:
    """Generates unix timestamp for an old date."""
    return test_context.TestTime(calendar.timegm(datetime.date(1970, 1, 1).timetuple()))


class TestHandleMonthlyNew(unittest.TestCase):
    """Tests handle_monthly_new()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_monthly_new(ctx, src_root, j, month_range=12)
        monthly = j["monthly"]
        self.assertEqual(len(monthly), 2)
        # 2019-05 start -> end
        self.assertEqual(monthly[0], ["2019-05", 3799])
        # diff from last month end -> today
        self.assertEqual(monthly[1], ["2020-05", 51334])

    def test_empty_month_range(self) -> None:
        """Tests the case when the month range is empty."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_monthly_new(ctx, src_root, j, month_range=-1)
        monthly = j["monthly"]
        self.assertTrue(monthly)

    def test_incomplete_last_month(self) -> None:
        """Tests the case when we have no data for the last, incomplete month."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        # This would be the data for the current state of the last, incomplete month.
        hide_path = ctx.get_abspath("workdir/stats/2020-05-10.count")
        file_system = test_context.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        ctx.set_file_system(file_system)

        j = stats.handle_monthly_new(ctx, src_root, j, month_range=12)
        monthly = j["monthly"]
        # 1st element: 2019-05 start -> end
        # No 2nd element, would be diff from last month end -> today
        self.assertEqual(len(monthly), 1)
        self.assertEqual(monthly[0], ["2019-05", 3799])


class TestHandleDailyTotal(unittest.TestCase):
    """Tests handle_daily_total()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_daily_total(ctx, src_root, j, day_range=13)
        dailytotal = j["dailytotal"]
        self.assertEqual(len(dailytotal), 1)
        self.assertEqual(dailytotal[0], ["2020-04-27", 251614])

    def test_empty_day_range(self) -> None:
        """Tests the case when the day range is empty."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_daily_total(ctx, src_root, j, day_range=-1)
        dailytotal = j["dailytotal"]
        self.assertFalse(dailytotal)


class TestHandleUserTotal(unittest.TestCase):
    """Tests handle_user_total()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_user_total(ctx, src_root, j, day_range=13)
        usertotal = j["usertotal"]
        self.assertEqual(len(usertotal), 1)
        self.assertEqual(usertotal[0], ["2020-04-27", 43])

    def test_empty_day_range(self) -> None:
        """Tests the case when the day range is empty."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_user_total(ctx, src_root, j, day_range=-1)
        usertotal = j["usertotal"]
        self.assertFalse(usertotal)


class TestHandleMonthlyTotal(unittest.TestCase):
    """Tests handle_monthly_total()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_monthly_total(ctx, src_root, j, month_range=11)
        monthlytotal = j["monthlytotal"]
        self.assertEqual(len(monthlytotal), 1)
        self.assertEqual(monthlytotal[0], ['2019-05', 203317])

    def test_empty_day_range(self) -> None:
        """Tests the case when the day range is empty."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_monthly_total(ctx, src_root, j, month_range=-1)
        monthlytotal = j["monthlytotal"]
        self.assertFalse(monthlytotal)

    def test_one_element_day_range(self) -> None:
        """Tests the case when the day range is of just one element."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        src_root = ctx.get_abspath("workdir/stats")
        j: Dict[str, Any] = {}
        j = stats.handle_monthly_total(ctx, src_root, j, month_range=0)
        monthlytotal = j["monthlytotal"]
        self.assertEqual(len(monthlytotal), 2)
        self.assertEqual(monthlytotal[0], ["2020-04", 253027])
        self.assertEqual(monthlytotal[1], ["2020-05", 254651])


class TestGetPreviousMonth(unittest.TestCase):
    """Tests get_previous_month()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        time = test_context.make_test_time()
        today = int(time.now())

        actual = datetime.date.fromtimestamp(stats.get_previous_month(today, 2))

        expected = datetime.date(2020, 3, 31)
        self.assertEqual(actual, expected)


class TestGetTopcities(unittest.TestCase):
    """Tests get_topcities()."""
    def test_old_missing(self) -> None:
        """Tests the case when the old path is missing."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        file_system = test_context.TestFileSystem()
        src_root = ctx.get_abspath("workdir/stats")
        file_system.set_hide_paths([os.path.join(src_root, "2020-04-10.citycount")])
        ctx.set_file_system(file_system)
        ret = stats.get_topcities(ctx, src_root)
        self.assertEqual(ret, [])

    def test_new_missing(self) -> None:
        """Tests the case when the new path is missing."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        file_system = test_context.TestFileSystem()
        src_root = ctx.get_abspath("workdir/stats")
        file_system.set_hide_paths([os.path.join(src_root, "2020-05-10.citycount")])
        ctx.set_file_system(file_system)
        ret = stats.get_topcities(ctx, src_root)
        self.assertEqual(ret, [])


if __name__ == '__main__':
    unittest.main()
