#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cron module covers the cron module."""

import unittest
import unittest.mock

import cron


class TestOverpassSleep(unittest.TestCase):
    """Tests overpass_sleep()."""
    def test_no_sleep(self) -> None:
        """Tests the case when no sleep is needed."""
        def mock_overpass_query_need_sleep() -> int:
            return 0
        mock_sleep_called = False

        def mock_sleep(_seconds: float) -> None:
            nonlocal mock_sleep_called
            mock_sleep_called = True
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', mock_overpass_query_need_sleep):
            with unittest.mock.patch('time.sleep', mock_sleep):
                cron.overpass_sleep()
                self.assertFalse(mock_sleep_called)

    def test_need_sleep(self) -> None:
        """Tests the case when sleep is needed."""
        sleep_for = 42

        def mock_overpass_query_need_sleep() -> int:
            nonlocal sleep_for
            if sleep_for > 0:
                sleep_for = 0
                return 42
            return sleep_for
        captured_seconds = 0.0

        def mock_sleep(seconds: float) -> None:
            nonlocal captured_seconds
            captured_seconds = seconds
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', mock_overpass_query_need_sleep):
            with unittest.mock.patch('time.sleep', mock_sleep):
                cron.overpass_sleep()
                self.assertEqual(captured_seconds, 42.0)


if __name__ == '__main__':
    unittest.main()
