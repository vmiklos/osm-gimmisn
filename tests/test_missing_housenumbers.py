#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_missing_housenumbers module covers the missing_housenumbers module."""

import io
import unittest

import test_context

import rust


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        argv = ["", "gh195"]
        buf = io.BytesIO()
        buf.__setattr__("close", lambda: None)
        ctx = test_context.make_test_context()
        rust.py_missing_housenumbers_main(argv, buf, ctx)

        buf.seek(0)
        self.assertEqual(buf.read(), b'Kalotaszeg utca\t3\n["25", "27-37", "31*"]\n')


if __name__ == '__main__':
    unittest.main()
