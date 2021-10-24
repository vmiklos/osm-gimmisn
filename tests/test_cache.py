#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cache module covers the cache module."""

import os
import unittest

import test_context

import areas
import cache


class TestIsMissingHousenumbersHtmlCached(unittest.TestCase):
    """Tests is_missing_housenumbers_html_cached()."""
    def test_ref_housenumbers_new(self) -> None:
        """Tests the case when ref_housenumbers is new, so the cache entry is old."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(ctx, relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        ref_housenumbers_path = relation.get_files().get_ref_housenumbers_path()

        file_system = test_context.TestFileSystem()
        mtimes = {
            ref_housenumbers_path: os.path.getmtime(cache_path) + 1,
        }
        file_system.set_mtimes(mtimes)
        ctx.set_file_system(file_system)
        self.assertFalse(cache.is_missing_housenumbers_html_cached(ctx, relation))

    def test_relation_new(self) -> None:
        """Tests the case when relation is new, so the cache entry is old."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_html(ctx, relation)
        cache_path = relation.get_files().get_housenumbers_htmlcache_path()
        datadir = ctx.get_abspath("data")
        relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())

        file_system = test_context.TestFileSystem()
        mtimes = {
            relation_path: os.path.getmtime(cache_path) + 1,
        }
        file_system.set_mtimes(mtimes)
        ctx.set_file_system(file_system)
        self.assertFalse(cache.is_missing_housenumbers_html_cached(ctx, relation))


class TestGetAdditionalHousenumbersHtml(unittest.TestCase):
    """Tests get_additional_housenumbers_html()."""
    def test_happy(self) -> None:
        """Tests the case when we find the result in cache."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        relation = relations.get_relation("gazdagret")
        first = cache.get_additional_housenumbers_html(ctx, relation)
        second = cache.get_additional_housenumbers_html(ctx, relation)
        self.assertEqual(first.get_value(), second.get_value())


class TestIsMissingHousenumbersTxtCached(unittest.TestCase):
    """Tests is_missing_housenumbers_txt_cached()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        relation = relations.get_relation("gazdagret")
        cache.get_missing_housenumbers_txt(ctx, relation)
        self.assertTrue(cache.is_missing_housenumbers_txt_cached(ctx, relation))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
