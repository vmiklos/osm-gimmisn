#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_accept_language module covers the accept_language module."""

import unittest
import unittest.mock

import accept_language


class TestParseAcceptLanguage(unittest.TestCase):
    """Tests parse_accept_language()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        parsed = accept_language.parse_accept_language("en-US,el;q=0.8")
        self.assertEqual(parsed[0].language, "en")

    def test_empty(self) -> None:
        """Tests empty input."""
        parsed = accept_language.parse_accept_language("")
        self.assertEqual(parsed, [])

    def test_too_long(self) -> None:
        """Tests too long input."""
        with unittest.mock.patch('accept_language.MAX_HEADER_LEN', 3):
            with self.assertRaises(ValueError):
                accept_language.parse_accept_language("en-US")

    def test_invalid_lang(self) -> None:
        """Tests the case when a language string is invalid."""
        parsed = accept_language.parse_accept_language("en42-US,el;q=0.8")
        self.assertEqual(parsed[0].language, "el")


if __name__ == '__main__':
    unittest.main()
