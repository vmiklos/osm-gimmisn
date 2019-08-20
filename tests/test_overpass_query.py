#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_overpass_query module covers the overpass_query module."""

from typing import BinaryIO
from typing import Callable
from typing import Optional
import io
import os
import unittest
import unittest.mock
import urllib.error
import overpass_query


def gen_urlopen(name: str) -> Callable[[str, Optional[bytes]], BinaryIO]:
    """Generates a mock for urllib.request.urlopen()."""
    def mock_urlopen(url: str, data: Optional[bytes] = None) -> BinaryIO:
        """Mocks urllib.request.urlopen()."""
        base_path = os.path.join("tests/mock", name)
        with open(base_path + ".url", "r") as stream:
            assert url == stream.read()
        if data:
            with open(base_path + ".request-data", "rb") as request_stream:
                assert data == request_stream.read()

        with open(base_path + ".response-data", "rb") as response_stream:
            buf = io.BytesIO()
            buf.write(response_stream.read())
            buf.seek(0)
            return buf

    def fail(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
        raise urllib.error.HTTPError(url=None, code=None, msg=None, hdrs=None, fp=None)

    if name:
        return mock_urlopen

    return fail


class TestOverpassQueryNeedSleeep(unittest.TestCase):
    """Tests overpass_query_need_sleep()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('urllib.request.urlopen', gen_urlopen("overpass-status-happy")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 0)

    def test_wait(self) -> None:
        """Tests the wait path."""
        with unittest.mock.patch('urllib.request.urlopen', gen_urlopen("overpass-status-wait")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 12)

    def test_wait_negative(self) -> None:
        """Tests the wait for negative amount path."""
        with unittest.mock.patch('urllib.request.urlopen', gen_urlopen("overpass-status-wait-negative")):
            self.assertEqual(overpass_query.overpass_query_need_sleep(), 1)


class TestOverpassQuery(unittest.TestCase):
    """Tests overpass_query()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('urllib.request.urlopen', gen_urlopen("overpass-interpreter-happy")):
            with open("tests/mock/overpass-interpreter-happy.request-data") as stream:
                query = stream.read()
                ret = overpass_query.overpass_query(query)
                self.assertEqual(ret[:3], "@id")


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        with unittest.mock.patch('urllib.request.urlopen', gen_urlopen("overpass-interpreter-happy")):
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                argv = ["", "tests/mock/overpass-interpreter-happy.request-data"]
                with unittest.mock.patch('sys.argv', argv):
                    overpass_query.main()
            buf.seek(0)
            self.assertTrue(buf.read().startswith("@id"))

    def test_failure(self) -> None:
        """Tests the failure path."""
        with unittest.mock.patch('urllib.request.urlopen', gen_urlopen("")):
            buf = io.StringIO()
            with unittest.mock.patch('sys.stdout', buf):
                argv = ["", "tests/mock/overpass-interpreter-happy.request-data"]
                with unittest.mock.patch('sys.argv', argv):
                    overpass_query.main()
            buf.seek(0)
            self.assertTrue(buf.read().startswith("overpass query failed"))


if __name__ == '__main__':
    unittest.main()
