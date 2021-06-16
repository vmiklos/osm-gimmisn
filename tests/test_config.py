#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

from typing import Dict
from typing import List
import os

import config


def make_test_config() -> config.Config:
    """Creates a Config instance that has its root as /tests."""
    return config.Config("tests")


class TestFileSystem(config.FileSystem):
    """File system implementation, for test purposes."""
    def __init__(self) -> None:
        self.__hide_paths: List[str] = []
        self.__mtimes: Dict[str, float] = {}

    def set_hide_paths(self, hide_paths: List[str]) -> None:
        """Sets the hide paths."""
        self.__hide_paths = hide_paths

    def set_mtimes(self, mtimes: Dict[str, float]) -> None:
        """Sets the mtimes."""
        self.__mtimes = mtimes

    def path_exists(self, path: str) -> bool:
        if path in self.__hide_paths:
            return False
        return os.path.exists(path)

    def getmtime(self, path: str) -> float:
        if path in self.__mtimes:
            return self.__mtimes[path]
        return os.path.getmtime(path)
