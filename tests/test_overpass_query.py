#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_overpass_query module covers the overpass_query module."""

from typing import Any
import unittest
import io
import mock
import overpass_query


def mock_urlopen(path: str) -> Any:
    """Mocks urllib.request.urlopen()."""
    with open("tests/mock/%s" % path, "rb") as stream:
        buf = stream.read()
        # work around mypy 'error: Cannot infer type of lambda'
        return lambda _url, _data=None: io.BytesIO(buf)


class TestOverpassQueryNeedSleeep(unittest.TestCase):
    """Tests overpass_query_need_sleep()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with mock.patch('urllib.request.urlopen', mock_urlopen("overpass-status-happy")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 0)


class TestOverpassQuery(unittest.TestCase):
    """Tests overpass_query()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with mock.patch('urllib.request.urlopen', mock_urlopen("overpass-interpreter-happy.out")):
            with open("tests/mock/overpass-interpreter-happy.in") as stream:
                query = stream.read()
                ret = overpass_query.overpass_query(query)
                self.assertEqual(ret[:3], "@id")


if __name__ == '__main__':
    unittest.main()
