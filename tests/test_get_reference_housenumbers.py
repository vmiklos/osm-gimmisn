#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_get_reference_housenumbers module covers the get_reference_housenumbers module."""

import unittest
import unittest.mock

import test_config

import config
import get_reference_housenumbers
import util


class TestMain(test_config.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        expected = util.get_content(config.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"))

        argv = ["", "gazdagret"]
        conf = config.Config("tests")
        with unittest.mock.patch('sys.argv', argv):
            get_reference_housenumbers.main(conf)

        actual = util.get_content(config.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"))
        self.assertEqual(actual, expected)


if __name__ == '__main__':
    unittest.main()
