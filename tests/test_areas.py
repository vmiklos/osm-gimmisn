#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_areas module covers the areas module."""

import unittest

import test_context

import areas
import rust


class TestRelationConfigMissingStreets(unittest.TestCase):
    """Tests RelationConfig.should_check_missing_streets()."""
    def test_nosuchrelation(self) -> None:
        """Tests a relation without a filter file."""
        relation_name = "nosuchrelation"
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation(relation_name)
        ret = relation.get_config().should_check_missing_streets()
        self.assertEqual(ret, "yes")


class TestRelationConfigLetterSuffixStyle(unittest.TestCase):
    """Tests RelationConfig.get_letter_suffix_style()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relation_name = "empty"
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation(relation_name)
        self.assertEqual(relation.get_config().get_letter_suffix_style(), rust.PyLetterSuffixStyle.upper())
        config = relation.get_config()
        config.set_letter_suffix_style(rust.PyLetterSuffixStyle.lower())
        relation.set_config(config)
        self.assertEqual(relation.get_config().get_letter_suffix_style(), rust.PyLetterSuffixStyle.lower())


class TestRefmegyeGetName(unittest.TestCase):
    """Tests refcounty_get_name()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        self.assertEqual(relations.refcounty_get_name("01"), "Budapest")
        self.assertEqual(relations.refcounty_get_name("99"), "")


class TestRefmegyeGetReftelepulesIds(unittest.TestCase):
    """Tests refcounty_get_refsettlement_ids()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        self.assertEqual(relations.refcounty_get_refsettlement_ids("01"), ["011", "012"])
        self.assertEqual(relations.refcounty_get_refsettlement_ids("99"), [])


class TestReftelepulesGetName(unittest.TestCase):
    """Tests refsettlement_get_name()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        self.assertEqual(relations.refsettlement_get_name("01", "011"), "Újbuda")
        self.assertEqual(relations.refsettlement_get_name("99", ""), "")
        self.assertEqual(relations.refsettlement_get_name("01", "99"), "")


class TestRelationsGetAliases(unittest.TestCase):
    """Tests Relalations.get_aliases()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        # Expect an alias -> canonicalname map.
        expected = {
            "budapest_22": "budafok"
        }
        self.assertEqual(relations.get_aliases(), expected)


class TestRelationStreetIsEvenOdd(unittest.TestCase):
    """Tests RelationConfig.get_street_is_even_odd()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.make_relations(ctx)
        relation = relations.get_relation("gazdagret")
        self.assertFalse(relation.get_config().get_street_is_even_odd("Hamzsabégi út"))

        self.assertTrue(relation.get_config().get_street_is_even_odd("Teszt utca"))


class TestRelationShowRefstreet(unittest.TestCase):
    """Tests RelationConfig.should_show_ref_street()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        self.assertFalse(relation.should_show_ref_street("Törökugrató utca"))
        self.assertTrue(relation.should_show_ref_street("Hamzsabégi út"))


class TestRelationIsActive(unittest.TestCase):
    """Tests RelationConfig.is_active()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation("gazdagret")
        self.assertTrue(relation.get_config().is_active())
