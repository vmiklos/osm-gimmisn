#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

from typing import BinaryIO
from typing import Dict
from typing import List
from typing import Optional
from typing import Tuple
from typing import cast
import calendar
import datetime
import io
import os
import unittest

import context


def make_test_context() -> context.Context:
    """Creates a Context instance for text purposes."""
    return context.Context("tests")


class TestFileSystem(context.FileSystem):
    """File system implementation, for test purposes."""
    def __init__(self) -> None:
        self.__hide_paths: List[str] = []
        self.__mtimes: Dict[str, float] = {}
        self.__files: Dict[str, io.BytesIO] = {}

    def set_hide_paths(self, hide_paths: List[str]) -> None:
        """Sets the hide paths."""
        self.__hide_paths = hide_paths

    def set_mtimes(self, mtimes: Dict[str, float]) -> None:
        """Sets the mtimes."""
        self.__mtimes = mtimes

    def path_exists(self, path: str) -> bool:
        if path in self.__hide_paths:
            return False
        if path in self.__files:
            return True
        return os.path.exists(path)

    def getmtime(self, path: str) -> float:
        if path in self.__mtimes:
            return self.__mtimes[path]
        return os.path.getmtime(path)

    def set_files(self, files: Dict[str, io.BytesIO]) -> None:
        """Sets the files."""
        self.__files = files

    def open(self, path: str, mode: str) -> BinaryIO:
        if path in self.__files:
            self.__files[path].seek(0)
            return self.__files[path]
        # The caller will do this:
        # pylint: disable=consider-using-with
        return cast(BinaryIO, open(path, mode))


class URLRoute:
    """Contains info about how to patch out one URL."""
    # The request URL
    url: str
    # Path of expected POST data, empty for GET
    data_path: str
    # Path of expected result data
    result_path: str

    def __init__(self, url: str, data_path: str, result_path: str) -> None:
        self.url = url
        self.data_path = data_path
        self.result_path = result_path


class TestNetwork(context.Network):
    """Network implementation, for test purposes."""
    def __init__(self, routes: List[URLRoute]) -> None:
        self.__routes = routes

    def urlopen(self, url: str, data: Optional[bytes] = None) -> Tuple[bytes, str]:
        for route in self.__routes:
            if url != route.url:
                continue

            if route.data_path:
                with open(route.data_path, "rb") as stream:
                    expected = stream.read()
                    if data != expected:
                        assert data
                        assert data == expected, \
                            "bad data: actual is '" + str(data, 'utf-8') + \
                            "', expected '" + str(expected, "utf-8") + "'"

            if not route.result_path:
                return (bytes(), "empty result_path for url '" + url + "'")
            with open(route.result_path, "rb") as stream:
                # Allow specifying multiple results for the same URL.
                self.__routes.remove(route)
                return (stream.read(), str())

        return (bytes(), "url missing from route list: '" + url + "'")


class TestTime(context.Time):
    """Time implementation, for test purposes."""
    def __init__(self, now: float) -> None:
        self.__now = now
        self.__sleep: float = 0

    def now(self) -> float:
        return self.__now

    def sleep(self, seconds: float) -> None:
        self.__sleep = seconds

    def get_sleep(self) -> float:
        """Gets the duration of the last sleep."""
        return self.__sleep


class TestSubprocess(context.Subprocess):
    """Subprocess implementation, for test purposes."""
    def __init__(self, outputs: Dict[str, bytes]) -> None:
        self.__outputs = outputs
        self.__environments: Dict[str, Dict[str, str]] = {}
        self.__runs: List[str] = []

    def get_environment(self, args: str) -> Dict[str, str]:
        """Gets the environment used for one specific cmdline."""
        return self.__environments[args]

    def get_runs(self) -> List[str]:
        """Gets a list of invoked commands."""
        return self.__runs

    def run(self, args: List[str], env: Dict[str, str]) -> bytes:
        key = " ".join(args)
        self.__environments[key] = env
        self.__runs.append(key)
        return self.__outputs[key]


class TestUnit(context.Unit):
    """Unit implementation, which intentionally fails."""
    def make_error(self) -> str:
        return "TestError"


def make_test_time() -> context.Time:
    """Generates unix timestamp for 2020-05-10."""
    return TestTime(calendar.timegm(datetime.date(2020, 5, 10).timetuple()))


class TestIniGetTcpPort(unittest.TestCase):
    """Tests Ini.get_tcp_port()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = make_test_context()
        self.assertEqual(ctx.get_ini().get_tcp_port(), 8000)
