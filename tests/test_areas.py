#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_areas module covers the areas module."""

import io
import os
import unittest

import test_context

import areas
import rust
import util


class TestRelationBuildRefStreets(unittest.TestCase):
    """Tests Relation.build_ref_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "utcak_20190514.tsv")
        memory_cache = util.build_street_reference_cache(refpath)
        relation_name = "gazdagret"
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation(relation_name)
        ret = relation.get_config().build_ref_streets(memory_cache)
        self.assertEqual(ret, ['Törökugrató utca',
                               'Tűzkő utca',
                               'Ref Name 1',
                               'Only In Ref utca',
                               'Only In Ref Nonsense utca',
                               'Hamzsabégi út'])


class TestRelationWriteRefHousenumbers(unittest.TestCase):
    """Tests Relation.write_ref_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        refpath2 = os.path.join(refdir, "hazszamok_kieg_20190808.tsv")
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        ref_value = io.BytesIO()
        ref_value.__setattr__("close", lambda: None)
        files = {
            os.path.join(ctx.get_ini().get_workdir(), "street-housenumbers-reference-gazdagret.lst"): ref_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        relations = areas.make_relations(ctx)
        relation_name = "gazdagret"
        expected = util.get_content(relations.get_workdir() + "/street-housenumbers-reference-gazdagret.lst")
        relation = relations.get_relation(relation_name)

        relation.write_ref_housenumbers([refpath, refpath2])

        ref_value.seek(0)
        self.assertEqual(ref_value.read(), expected)

    def test_nosuchrefcounty(self) -> None:
        """Tests the case when the refcounty code is missing in the reference."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        ref_value = io.BytesIO()
        ref_value.__setattr__("close", lambda: None)
        files = {
            os.path.join(ctx.get_ini().get_workdir(), "street-housenumbers-reference-nosuchrefcounty.lst"): ref_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        relations = areas.make_relations(ctx)
        relation_name = "nosuchrefcounty"
        relation = relations.get_relation(relation_name)
        try:
            relation.write_ref_housenumbers([refpath])
        except KeyError:
            self.fail("write_ref_housenumbers() raised KeyError unexpectedly")

    def test_nosuchrefsettlement(self) -> None:
        """Tests the case when the refsettlement code is missing in the reference."""
        refdir = os.path.join(os.path.dirname(__file__), "refdir")
        refpath = os.path.join(refdir, "hazszamok_20190511.tsv")
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        ref_value = io.BytesIO()
        ref_value.__setattr__("close", lambda: None)
        files = {
            os.path.join(ctx.get_ini().get_workdir(), "street-housenumbers-reference-nosuchrefsettlement.lst"): ref_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        relations = areas.make_relations(ctx)
        relation_name = "nosuchrefsettlement"
        relation = relations.get_relation(relation_name)
        try:
            relation.write_ref_housenumbers([refpath])
        except KeyError:
            self.fail("write_ref_housenumbers() raised KeyError unexpectedly")


class TestRelationWriteRefStreets(unittest.TestCase):
    """Tests Relation.WriteRefStreets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        ref_value = io.BytesIO()
        ref_value.__setattr__("close", lambda: None)
        files = {
            os.path.join(ctx.get_ini().get_workdir(), "streets-reference-gazdagret.lst"): ref_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        refpath = ctx.get_abspath(os.path.join("refdir", "utcak_20190514.tsv"))
        relations = areas.make_relations(ctx)
        relation_name = "gazdagret"
        relation = relations.get_relation(relation_name)
        expected = util.get_content(relations.get_workdir() + "/streets-reference-gazdagret.lst")
        relation.write_ref_streets(refpath)
        ref_value.seek(0)
        self.assertEqual(ref_value.read(), expected)


class TestRelations(unittest.TestCase):
    """Tests the Relations class."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relations = areas.make_relations(test_context.make_test_context())
        expected_relation_names = [
            "budafok",
            "empty",
            "gazdagret",
            "gellerthegy",
            "inactiverelation",
            "nosuchrefcounty",
            "nosuchrefsettlement",
            "nosuchrelation",
            "test",
            "ujbuda"
        ]
        self.assertEqual(relations.get_names(), expected_relation_names)
        self.assertTrue("inactiverelation" not in relations.get_active_names())
        osmids = sorted([relation.get_config().get_osmrelation() for relation in relations.get_relations()])
        self.assertEqual([13, 42, 42, 43, 44, 45, 66, 221998, 2702687, 2713748], osmids)
        self.assertEqual("only", relations.get_relation("ujbuda").get_config().should_check_missing_streets())

        relations.activate_all(True)
        self.assertTrue("inactiverelation" in relations.get_active_names())

        # Allow seeing data of a relation even if it's not in relations.yaml.
        relations.get_relation("gh195")

        # Test limit_to_refcounty().
        # 01
        self.assertTrue("gazdagret" in relations.get_active_names())
        # 43
        self.assertTrue("budafok" in relations.get_active_names())
        relations.limit_to_refcounty("01")
        self.assertTrue("gazdagret" in relations.get_active_names())
        self.assertTrue("budafok" not in relations.get_active_names())

        # Test limit_to_refsettlement().
        # 011
        self.assertTrue("gazdagret" in relations.get_active_names())
        # 99
        self.assertTrue("nosuchrefsettlement" in relations.get_active_names())
        relations.limit_to_refsettlement("99")
        self.assertTrue("gazdagret" not in relations.get_active_names())
        self.assertTrue("nosuchrefsettlement" in relations.get_active_names())


class TestRelationConfigMissingStreets(unittest.TestCase):
    """Tests RelationConfig.should_check_missing_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        relation_name = "ujbuda"
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation(relation_name)
        ret = relation.get_config().should_check_missing_streets()
        self.assertEqual(ret, "only")

    def test_empty(self) -> None:
        """Tests the default value."""
        relation_name = "empty"
        relations = areas.make_relations(test_context.make_test_context())
        relation = relations.get_relation(relation_name)
        self.assertEqual(relation.get_name(), "empty")
        ret = relation.get_config().should_check_missing_streets()
        self.assertEqual(ret, "yes")

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
