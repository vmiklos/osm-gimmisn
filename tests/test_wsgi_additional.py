#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi_additional module covers the wsgi_additional module."""

import os
import unittest
import unittest.mock

import test_wsgi

import areas
import config


class TestStreets(test_wsgi.TestWsgi):
    """Tests additional streets."""
    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "Only In OSM utca\n")

    def test_view_result_chkl(self) -> None:
        """Tests the chkl output."""
        result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.chkl")
        self.assertEqual(result, "[ ] Only In OSM utca\n")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_ref_streets(self) -> None:
        """Tests the txt output, no ref streets case."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "No reference streets")

    def test_view_turbo_well_formed(self) -> None:
        """Tests if the view-turbo output is well-formed."""
        root = self.get_dom_for_path("/additional-streets/gazdagret/view-turbo")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)


if __name__ == '__main__':
    unittest.main()
