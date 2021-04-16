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
import config


class TestIsMissingHousenumbersHtmlCached(test_config.TestCase):
    """Tests is_missing_housenumbers_html_cached()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(relation)
        self.assertTrue(cache.is_missing_housenumbers_html_cached(relation))

    def test_no_cache(self) -> None:
        """Tests the case when there is no cache."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        orig_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == cache_path:
                return False
            return orig_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            self.assertFalse(cache.is_missing_housenumbers_html_cached(relation))

    def test_osm_housenumbers_new(self) -> None:
        """Tests the case when osm_housenumbers is new, so the cache entry is old."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        osm_housenumbers_path = relation.get_files().get_osm_housenumbers_path()
        orig_getmtime = os.path.getmtime

        def mock_getmtime(path: str) -> float:
            if path == osm_housenumbers_path:
                return orig_getmtime(cache_path) + 1
            return orig_getmtime(path)
        with unittest.mock.patch('os.path.getmtime', mock_getmtime):
            self.assertFalse(cache.is_missing_housenumbers_html_cached(relation))

    def test_ref_housenumbers_new(self) -> None:
        """Tests the case when ref_housenumbers is new, so the cache entry is old."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        ref_housenumbers_path = relation.get_files().get_ref_housenumbers_path()
        orig_getmtime = os.path.getmtime

        def mock_getmtime(path: str) -> float:
            if path == ref_housenumbers_path:
                return orig_getmtime(cache_path) + 1
            return orig_getmtime(path)
        with unittest.mock.patch('os.path.getmtime', mock_getmtime):
            self.assertFalse(cache.is_missing_housenumbers_html_cached(relation))

    def test_relation_new(self) -> None:
        """Tests the case when relation is new, so the cache entry is old."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        datadir = config.get_abspath("data")
        relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())
        orig_getmtime = os.path.getmtime

        def mock_getmtime(path: str) -> float:
            if path == relation_path:
                return orig_getmtime(cache_path) + 1
            return orig_getmtime(path)
        with unittest.mock.patch('os.path.getmtime', mock_getmtime):
            self.assertFalse(cache.is_missing_housenumbers_html_cached(relation))


class TestIsMissingHousenumbersTxtCached(test_config.TestCase):
    """Tests is_missing_housenumbers_txt_cached()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relations = areas.Relations(config.Config.get_workdir())
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_txt(relation)
        self.assertTrue(cache.is_missing_housenumbers_txt_cached(relation))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
