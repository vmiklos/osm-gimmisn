#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_stats module covers the stats module."""

from typing import Any
from typing import Dict
import datetime
import os
import unittest
import unittest.mock

import stats


def get_abspath(path: str) -> str:
    """Mock get_abspath() that uses the test directory."""
    if os.path.isabs(path):
        return path
    return os.path.join(os.path.dirname(__file__), path)


def mock_strftime(_fmt: str) -> str:
    """Mock time.strftime()."""
    return "2020-05-10"


def mock_strftime_old(_fmt: str) -> str:
    """Mock time.strftime(), returning an old date."""
    return "1970-01-01"


class MockDate(datetime.date):
    """Mock datetime.date."""
    @classmethod
    def today(cls) -> 'MockDate':
        """Returns today's date."""
        return cls(2020, 5, 10)


class TestHandleProgress(unittest.TestCase):
    """Tests handle_progress()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('time.strftime', mock_strftime):
                stats.handle_progress(src_root, j)
            progress = j["progress"]
            self.assertEqual(progress["date"], "2020-05-10")
            self.assertEqual(progress["percentage"], 7.81)

    def test_old_time(self) -> None:
        """Tests the case when the .count file doesn't exist for a date."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('time.strftime', mock_strftime_old):
                stats.handle_progress(src_root, j)
            progress = j["progress"]
            self.assertEqual(progress["date"], "1970-01-01")


class TestHandleTopusers(unittest.TestCase):
    """Tests handle_topusers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('time.strftime', mock_strftime):
                stats.handle_topusers(src_root, j)
            topusers = j["topusers"]
            self.assertEqual(len(topusers), 20)
            self.assertEqual(topusers[0], ["user1", "68885"])

    def test_old_time(self) -> None:
        """Tests the case when the .count file doesn't exist for a date."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('time.strftime', mock_strftime_old):
                stats.handle_topusers(src_root, j)
            topusers = j["topusers"]
            self.assertFalse(topusers)


class TestHandleDailyNew(unittest.TestCase):
    """Tests handle_daily_new()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('datetime.date', MockDate):
                # From now on, today is 2020-05-10, so this will read 2020-04-26, 2020-04-27, etc
                # (till a file is missing.)
                stats.handle_daily_new(src_root, j)
            daily = j["daily"]
            self.assertEqual(len(daily), 1)
            self.assertEqual(daily[0], ["2020-04-26", 364])

    def test_empty_day_range(self) -> None:
        """Tests the case when the day range is empty."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            stats.handle_daily_new(src_root, j, day_range=-1)
            daily = j["daily"]
            self.assertFalse(daily)


class TestHandleMonthlyNew(unittest.TestCase):
    """Tests handle_monthly_new()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('datetime.date', MockDate):
                stats.handle_monthly_new(src_root, j)
            monthly = j["monthly"]
            self.assertEqual(len(monthly), 2)
            # 2019-05 start -> end
            self.assertEqual(monthly[0], ["2019-05", 3799])
            # diff from last month end -> today
            self.assertEqual(monthly[1], ["2020-05", 51334])

    def test_empty_month_range(self) -> None:
        """Tests the case when the month range is empty."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            stats.handle_monthly_new(src_root, j, month_range=-1)
            monthly = j["monthly"]
            self.assertTrue(monthly)

    def test_incomplete_last_month(self) -> None:
        """Tests the case when we have no data for the last, incomplete month."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('datetime.date', MockDate):
                # This would be the data for the current state of the last, incomplete month.
                hide_path = get_abspath("workdir/stats/2020-05-10.count")
                real_exists = os.path.exists

                def mock_exists(path: str) -> bool:
                    if path == hide_path:
                        return False
                    return real_exists(path)
                with unittest.mock.patch('os.path.exists', mock_exists):
                    stats.handle_monthly_new(src_root, j)
            monthly = j["monthly"]
            # 1st element: 2019-05 start -> end
            # No 2nd element, would be diff from last month end -> today
            self.assertEqual(len(monthly), 1)
            self.assertEqual(monthly[0], ["2019-05", 3799])


class TestHandleDailyTotal(unittest.TestCase):
    """Tests handle_daily_total()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            with unittest.mock.patch('datetime.date', MockDate):
                stats.handle_daily_total(src_root, j)
            dailytotal = j["dailytotal"]
            self.assertEqual(len(dailytotal), 1)
            self.assertEqual(dailytotal[0], ["2020-04-27", 251614])

    def test_empty_day_range(self) -> None:
        """Tests the case when the day range is empty."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            stats.handle_daily_total(src_root, j, day_range=-1)
            dailytotal = j["dailytotal"]
            self.assertFalse(dailytotal)


if __name__ == '__main__':
    unittest.main()
