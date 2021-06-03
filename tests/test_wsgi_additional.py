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
import wsgi


class TestStreets(test_wsgi.TestWsgi):
    """Tests additional streets."""
    def test_view_result_txt(self) -> None:
        """Tests the txt output."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "Only In OSM utca\n")

    def test_view_result_chkl(self) -> None:
        """Tests the chkl output."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.chkl")
            self.assertEqual(result, "[ ] Only In OSM utca\n")

    def test_view_result_txt_no_osm_streets(self) -> None:
        """Tests the txt output, no osm streets case."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
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
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
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
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            root = self.get_dom_for_path("/additional-streets/gazdagret/view-turbo")
            results = root.findall("body/pre")
            self.assertEqual(len(results), 1)


class TestHandleMainHousenrAdditionalCount(test_wsgi.TestWsgi):
    """Tests handle_main_housenr_additional_count()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
            relation = relations.get_relation("budafok")
            actual = wsgi.handle_main_housenr_additional_count(relation)
            self.assertIn("42 house numbers", actual.getvalue())

    def test_no_count_file(self) -> None:
        """Tests what happens when the count file is not there."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
            relation = relations.get_relation("budafok")
            hide_path = relation.get_files().get_housenumbers_additional_count_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                actual = wsgi.handle_main_housenr_additional_count(relation)
            self.assertNotIn("42 housenumbers", actual.getvalue())


class TestAdditionalHousenumbers(test_wsgi.TestWsgi):
    """Tests the additional house numbers page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
            results = root.findall("body/table")
            self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_streets_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-osm-streets']")
                self.assertEqual(len(results), 1)

    def test_no_osm_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm housenumbers case."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_osm_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-osm-housenumbers']")
                self.assertEqual(len(results), 1)

    def test_no_ref_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref housenumbers case."""
        def mock_make_config() -> config.Config2:
            return config.Config2("tests")
        with unittest.mock.patch("config.make_config", mock_make_config):
            conf = mock_make_config()
            relations = areas.Relations(conf.get_workdir())
            relation = relations.get_relation("gazdagret")
            hide_path = relation.get_files().get_ref_housenumbers_path()
            real_exists = os.path.exists

            def mock_exists(path: str) -> bool:
                if path == hide_path:
                    return False
                return real_exists(path)
            with unittest.mock.patch('os.path.exists', mock_exists):
                root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
                results = root.findall("body/div[@id='no-ref-housenumbers']")
                self.assertEqual(len(results), 1)


if __name__ == '__main__':
    unittest.main()
