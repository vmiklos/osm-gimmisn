#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_overpass_query module covers the overpass_query module."""

from typing import Any
import unittest
import unittest.mock
import io
import urllib.error
import overpass_query


def mock_urlopen(path: str) -> Any:
    """Mocks urllib.request.urlopen()."""
    if path:
        with open("tests/mock/%s" % path, "rb") as stream:
            buf = stream.read()
            # work around mypy 'error: Cannot infer type of lambda'
            return lambda _url, _data=None: io.BytesIO(buf)

    def fail(_url: str, _data: bytes) -> None:
        raise urllib.error.HTTPError(url=None, code=None, msg=None, hdrs=None, fp=None)
    return fail


class TestOverpassQueryNeedSleeep(unittest.TestCase):
    """Tests overpass_query_need_sleep()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen("overpass-status-happy")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 0)

    def test_wait(self) -> None:
        """Tests the wait path."""
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen("overpass-status-wait")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 12)

    def test_wait_negative(self) -> None:
        """Tests the wait for negative amount path."""
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen("overpass-status-wait-negative")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 1)


class TestOverpassQuery(unittest.TestCase):
    """Tests overpass_query()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen("overpass-interpreter-happy.out")):
            with open("tests/mock/overpass-interpreter-happy.in") as stream:
                query = stream.read()
                ret = overpass_query.overpass_query(query)
                self.assertEqual(ret[:3], "@id")


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen("overpass-interpreter-happy.out")):
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                argv = ["", "tests/mock/overpass-interpreter-happy.in"]
                with unittest.mock.patch('sys.argv', argv):
                    overpass_query.main()
            buf.seek(0)
            self.assertTrue(buf.read().startswith("@id"))

    def test_failure(self) -> None:
        """Tests the failure path."""
        with unittest.mock.patch('urllib.request.urlopen', mock_urlopen("")):
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                argv = ["", "tests/mock/overpass-interpreter-happy.in"]
                with unittest.mock.patch('sys.argv', argv):
                    overpass_query.main()
            buf.seek(0)
            self.assertTrue(buf.read().startswith("overpass query failed"))


if __name__ == '__main__':
    unittest.main()
