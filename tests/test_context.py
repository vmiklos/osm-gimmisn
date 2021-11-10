#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

from typing import BinaryIO
from typing import Dict
from typing import List
import calendar
import datetime
import io
import os

import api
import context
import rust


def make_test_context() -> rust.PyContext:
    """Creates a Context instance for text purposes."""
    return context.make_context("tests")


class TestFileSystem(api.FileSystem):
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

    def open_read(self, path: str) -> BinaryIO:
        if path in self.__files:
            self.__files[path].seek(0)
            return self.__files[path]
        # The caller will do this:
        # pylint: disable=consider-using-with
        return open(path, "rb")

    def open_write(self, path: str) -> BinaryIO:
        if path in self.__files:
            self.__files[path].seek(0)
            return self.__files[path]
        # The caller will do this:
        # pylint: disable=consider-using-with
        return open(path, "wb")


class TestTime(api.Time):
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


class TestSubprocess(api.Subprocess):
    """Subprocess implementation, for test purposes."""
    def __init__(self, outputs: Dict[str, str]) -> None:
        self.__outputs = outputs
        self.__environments: Dict[str, Dict[str, str]] = {}
        self.__runs: List[str] = []
        self.__exits: List[int] = []

    def get_environment(self, args: str) -> Dict[str, str]:
        """Gets the environment used for one specific cmdline."""
        return self.__environments[args]

    def get_runs(self) -> List[str]:
        """Gets a list of invoked commands."""
        return self.__runs

    def get_exits(self) -> List[int]:
        """Gets a list of exit codes."""
        return self.__exits

    def run(self, args: List[str], env: Dict[str, str]) -> str:
        key = " ".join(args)
        self.__environments[key] = env
        self.__runs.append(key)
        return self.__outputs[key]

    def exit(self, code: int) -> None:
        self.__exits.append(code)


class TestUnit(api.Unit):
    """Unit implementation, which intentionally fails."""
    def make_error(self) -> str:
        return "TestError"


def make_test_time() -> TestTime:
    """Generates unix timestamp for 2020-05-10."""
    return TestTime(calendar.timegm(datetime.date(2020, 5, 10).timetuple()))
