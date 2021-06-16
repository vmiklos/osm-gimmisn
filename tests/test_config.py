#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

from typing import List
import os

import config


def make_test_config() -> config.Config:
    """Creates a Config instance that has its root as /tests."""
    return config.Config("tests")


class TestFileSystem(config.FileSystem):
    """File system implementation, for test purposes."""
    def __init__(self, hide_paths: List[str]) -> None:
        self.__hide_paths = hide_paths

    def path_exists(self, path: str) -> bool:
        if path in self.__hide_paths:
            return False
        return os.path.exists(path)
