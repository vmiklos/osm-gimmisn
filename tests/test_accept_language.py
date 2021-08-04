#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_accept_language module covers the accept_language module."""

import unittest

import accept_language


class TestParseAcceptLanguage(unittest.TestCase):
    """Tests parse_accept_language()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        parsed = accept_language.parse("en-US,el;q=0.8")
        self.assertEqual(parsed[0], "en")

    def test_empty(self) -> None:
        """Tests empty input."""
        parsed = accept_language.parse("")
        self.assertEqual(parsed, [])

    def test_invalid_lang(self) -> None:
        """Tests the case when a language string is invalid."""
        parsed = accept_language.parse("en42-US,el;q=0.8")
        self.assertEqual(parsed[0], "el")


if __name__ == '__main__':
    unittest.main()
