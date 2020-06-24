#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cache_yamls module covers the cache_yamls module."""

import os
import unittest
import unittest.mock

import cache_yamls
import test_config


class TestMain(test_config.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        cache_path = "tests/data/yamls.pickle"
        if os.path.exists(cache_path):
            os.remove(cache_path)
        argv = ["", "data"]
        with unittest.mock.patch('sys.argv', argv):
            cache_yamls.main()
        # Just assert that the result is created, the actual content is validated by the other
        # tests.
        self.assertTrue(os.path.exists(cache_path))


if __name__ == '__main__':
    unittest.main()
