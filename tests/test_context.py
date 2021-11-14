#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

from typing import Dict
from typing import List
import calendar
import datetime

import api
import context
import rust


def make_test_context() -> rust.PyContext:
    """Creates a Context instance for text purposes."""
    return context.make_context("tests")


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


def make_test_time() -> TestTime:
    """Generates unix timestamp for 2020-05-10."""
    return TestTime(calendar.timegm(datetime.date(2020, 5, 10).timetuple()))
