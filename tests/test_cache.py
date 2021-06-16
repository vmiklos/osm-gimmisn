#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cache module covers the cache module."""

import os
import unittest

import test_config

import areas
import cache


class TestIsMissingHousenumbersHtmlCached(unittest.TestCase):
    """Tests is_missing_housenumbers_html_cached()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(conf, relation)
        self.assertTrue(cache.is_missing_housenumbers_html_cached(conf, relation))

    def test_no_cache(self) -> None:
        """Tests the case when there is no cache."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(conf, relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()

        file_system = test_config.TestFileSystem()
        file_system.set_hide_paths([cache_path])
        conf.set_file_system(file_system)
        self.assertFalse(cache.is_missing_housenumbers_html_cached(conf, relation))

    def test_osm_housenumbers_new(self) -> None:
        """Tests the case when osm_housenumbers is new, so the cache entry is old."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(conf, relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        osm_housenumbers_path = relation.get_files().get_osm_housenumbers_path()

        file_system = test_config.TestFileSystem()
        mtimes = {
            osm_housenumbers_path: os.path.getmtime(cache_path) + 1,
        }
        file_system.set_mtimes(mtimes)
        conf.set_file_system(file_system)
        self.assertFalse(cache.is_missing_housenumbers_html_cached(conf, relation))

    def test_ref_housenumbers_new(self) -> None:
        """Tests the case when ref_housenumbers is new, so the cache entry is old."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(conf, relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        ref_housenumbers_path = relation.get_files().get_ref_housenumbers_path()

        file_system = test_config.TestFileSystem()
        mtimes = {
            ref_housenumbers_path: os.path.getmtime(cache_path) + 1,
        }
        file_system.set_mtimes(mtimes)
        conf.set_file_system(file_system)
        self.assertFalse(cache.is_missing_housenumbers_html_cached(conf, relation))

    def test_relation_new(self) -> None:
        """Tests the case when relation is new, so the cache entry is old."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(conf, relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        datadir = conf.get_abspath("data")
        relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())

        file_system = test_config.TestFileSystem()
        mtimes = {
            relation_path: os.path.getmtime(cache_path) + 1,
        }
        file_system.set_mtimes(mtimes)
        conf.set_file_system(file_system)
        self.assertFalse(cache.is_missing_housenumbers_html_cached(conf, relation))


class TestGetAdditionalHousenumbersHtml(unittest.TestCase):
    """Tests get_additional_housenumbers_html()."""
    def test_happy(self) -> None:
        """Tests the case when we find the result in cache."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        first = cache.get_additional_housenumbers_html(conf, relation)
        second = cache.get_additional_housenumbers_html(conf, relation)
        self.assertEqual(first.getvalue(), second.getvalue())


class TestIsMissingHousenumbersTxtCached(unittest.TestCase):
    """Tests is_missing_housenumbers_txt_cached()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = test_config.make_test_config()
        relations = areas.Relations(conf)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_txt(conf, relation)
        self.assertTrue(cache.is_missing_housenumbers_txt_cached(conf, relation))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
