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
            self.assertTrue(topusers)

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
            self.assertTrue(daily)

    def test_empty_day_range(self) -> None:
        """Tests the case when the day range is empty."""
        with unittest.mock.patch('config.get_abspath', get_abspath):
            src_root = get_abspath("workdir/stats")
            j: Dict[str, Any] = {}
            stats.handle_daily_new(src_root, j, day_range=-1)
            daily = j["daily"]
            self.assertFalse(daily)


if __name__ == '__main__':
    unittest.main()
