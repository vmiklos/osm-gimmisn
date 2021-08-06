#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_accept_language module covers the accept_language module."""

import unittest

import rust


class TestParseAcceptLanguage(unittest.TestCase):
    """Tests parse_accept_language()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        parsed = rust.py_parse("hu,en;q=0.9,en-US;q=0.8")
        self.assertEqual(parsed[0], "hu")

    def test_english(self) -> None:
        """Tests when the language is not explicitly set."""
        parsed = rust.py_parse("en-US,en;q=0.5")
        self.assertEqual(parsed[0], "en-US")
