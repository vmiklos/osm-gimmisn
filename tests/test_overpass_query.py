#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_overpass_query module covers the overpass_query module."""

from typing import List
import unittest

import test_config

import overpass_query


class TestOverpassQueryNeedSleeep(unittest.TestCase):
    """Tests overpass_query_need_sleep()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/status",
                                 data_path="",
                                 result_path="tests/network/overpass-status-happy.txt")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        self.assertEqual(overpass_query.overpass_query_need_sleep(conf), 0)

    def test_wait(self) -> None:
        """Tests the wait path."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/status",
                                 data_path="",
                                 result_path="tests/network/overpass-status-wait.txt")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        self.assertEqual(overpass_query.overpass_query_need_sleep(conf), 12)

    def test_wait_negative(self) -> None:
        """Tests the wait for negative amount path."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/status",
                                 data_path="",
                                 result_path="tests/network/overpass-status-wait-negative.txt")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        self.assertEqual(overpass_query.overpass_query_need_sleep(conf), 1)


class TestOverpassQuery(unittest.TestCase):
    """Tests overpass_query()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = test_config.make_test_config()
        routes: List[test_config.URLRoute] = [
            test_config.URLRoute(url="https://overpass-api.de/api/interpreter",
                                 data_path="tests/network/overpass-happy.expected-data",
                                 result_path="tests/network/overpass-happy.csv")
        ]
        network = test_config.TestNetwork(routes)
        conf.set_network(network)
        with open("tests/network/overpass-happy.expected-data") as stream:
            query = stream.read()
            ret = overpass_query.overpass_query(conf, query)
            self.assertEqual(ret[:3], "@id")


if __name__ == '__main__':
    unittest.main()
