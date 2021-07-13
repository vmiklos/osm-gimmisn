#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_i18n module covers the i18n module."""

from typing import Any
import unittest

import test_context

import context
import i18n


class LanguageContext:
    """Context manager for i18n.translate()."""
    def __init__(self, ctx: context.Context, language: str) -> None:
        """Remembers what should be the new language."""
        self.ctx = ctx
        self.language = language

    def __enter__(self) -> 'LanguageContext':
        """Switches to the new language."""
        i18n.set_language(self.ctx, self.language)
        return self

    def __exit__(self, _exc_type: Any, _exc_value: Any, _exc_traceback: Any) -> bool:
        """Switches back to the old language."""
        i18n.set_language(self.ctx, "en")
        return True


class TestTranslate(unittest.TestCase):
    """Tests translate()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        with LanguageContext(ctx, "hu"):
            self.assertEqual(i18n.translate("Area"), "Terület")


if __name__ == '__main__':
    unittest.main()
