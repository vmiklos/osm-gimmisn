#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_areas module covers the areas module."""

import unittest

import test_context

import areas


class TestRelationIsActive(unittest.TestCase):
    """Tests RelationConfig.is_active()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        self.assertTrue(relation.get_config().is_active())
