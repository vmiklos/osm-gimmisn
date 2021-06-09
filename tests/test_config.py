#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_config module covers the config module."""

import os
import unittest
import unittest.mock

import config


def make_test_config() -> config.Config:
    """Creates a Config instance that has its root as /tests."""
    return config.Config("tests")


class TestCase(unittest.TestCase):
    """Same as unittest.TestCase, but sets up get_abspath to use the test root."""


class TestGetAbspath(unittest.TestCase):
    """Tests get_abspath()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = config.Config("")
        abs_path = conf.get_abspath("file")
        self.assertEqual(abs_path, os.path.join(os.getcwd(), "file"))


if __name__ == '__main__':
    unittest.main()
