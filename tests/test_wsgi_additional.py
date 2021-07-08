#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi_additional module covers the wsgi_additional module."""

import io
import json
import unittest

import test_config
import test_wsgi

import areas
import wsgi


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
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_ref_streets(self) -> None:
        """Tests the txt output, no ref streets case."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
        self.assertEqual(result, "No reference streets")

    def test_view_turbo_well_formed(self) -> None:
        """Tests if the view-turbo output is well-formed."""
        root = self.get_dom_for_path("/additional-streets/gazdagret/view-turbo")
        results = root.findall("body/pre")
        self.assertEqual(len(results), 1)


class TestHandleMainHousenrAdditionalCount(test_wsgi.TestWsgi):
    """Tests handle_main_housenr_additional_count()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("budafok")
        actual = wsgi.handle_main_housenr_additional_count(conf, relation)
        self.assertIn("42 house numbers", actual.getvalue())

    def test_no_count_file(self) -> None:
        """Tests what happens when the count file is not there."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("budafok")
        hide_path = relation.get_files().get_housenumbers_additional_count_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        conf.set_file_system(file_system)
        actual = wsgi.handle_main_housenr_additional_count(conf, relation)
        self.assertNotIn("42 housenumbers", actual.getvalue())


class TestAdditionalHousenumbers(test_wsgi.TestWsgi):
    """Tests the additional house numbers page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-streets']")
        self.assertEqual(len(results), 1)

    def test_no_osm_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm housenumbers case."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_housenumbers_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-housenumbers']")
        self.assertEqual(len(results), 1)

    def test_no_ref_housenumbers_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref housenumbers case."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_housenumbers_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        root = self.get_dom_for_path("/additional-housenumbers/gazdagret/view-result")
        results = root.findall("body/div[@id='no-ref-housenumbers']")
        self.assertEqual(len(results), 1)


class TestAdditionalStreets(test_wsgi.TestWsgi):
    """Tests the additional streets page."""
    def test_well_formed(self) -> None:
        """Tests if the output is well-formed."""
        root = self.get_dom_for_path("/additional-streets/gazdagret/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)
        # refstreets: >0 invalid osm name
        results = root.findall("body/div[@id='osm-invalids-container']")
        self.assertEqual(len(results), 1)
        # refstreets: >0 invalid ref name
        results = root.findall("body/div[@id='ref-invalids-container']")
        self.assertEqual(len(results), 1)

    def test_street_from_housenr_well_formed(self) -> None:
        """Tests if the output is well-formed when the street name comes from a housenr."""
        file_system = test_config.TestFileSystem()
        yamls_cache = {
            "relations.yaml": {
                "gh611": {
                    "osmrelation": 42,
                },
            },
            "refcounty-names.yaml": {
            },
            "refsettlement-names.yaml": {
            },
        }
        yamls_cache_value = io.BytesIO()
        yamls_cache_value.write(json.dumps(yamls_cache).encode("utf-8"))
        yamls_cache_value.seek(0)
        file_system.set_files({self.conf.get_abspath("data/yamls.cache"): yamls_cache_value})
        self.conf.set_file_system(file_system)
        root = self.get_dom_for_path("/additional-streets/gh611/view-result")
        results = root.findall("body/table")
        self.assertEqual(len(results), 1)

    def test_no_osm_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no osm streets case."""
        relations = areas.Relations(test_config.make_test_config())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        root = self.get_dom_for_path("/additional-streets/gazdagret/view-result")
        results = root.findall("body/div[@id='no-osm-streets']")
        self.assertEqual(len(results), 1)

    def test_no_ref_streets_well_formed(self) -> None:
        """Tests if the output is well-formed, no ref streets case."""
        relations = areas.Relations(test_config.make_test_config())
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([hide_path])
        self.conf.set_file_system(file_system)
        root = self.get_dom_for_path("/additional-streets/gazdagret/view-result")
        results = root.findall("body/div[@id='no-ref-streets']")
        self.assertEqual(len(results), 1)


if __name__ == '__main__':
    unittest.main()
