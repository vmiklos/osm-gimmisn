#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_overpass_query module covers the overpass_query module."""

from typing import Callable
import unittest
import mock
import overpass_query


def mock_urlopen(path: str) -> Callable[[str], bytes]:
    """Mocks urllib.request.urlopen()."""
    with open("tests/mock/%s" % path, "rb") as stream:
        ret = stream.read()
        return lambda _url: ret


class TestOverpassQueryNeedSleeep(unittest.TestCase):
    """Tests overpass_query_need_sleep()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with mock.patch('urllib.request.urlopen', mock_urlopen("overpass-status-happy")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 0)


if __name__ == '__main__':
    unittest.main()
