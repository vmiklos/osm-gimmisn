#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cron module covers the cron module."""

from typing import Any
from typing import List
import os
import time
import unittest
import unittest.mock

import test_context

import areas
import context
import cron
import util


class TestOverpassSleep(unittest.TestCase):
    """Tests overpass_sleep()."""
    def test_no_sleep(self) -> None:
        """Tests the case when no sleep is needed."""
        def mock_overpass_query_need_sleep(_conf: context.Context) -> int:
            return 0
        mock_sleep_called = False

        def mock_sleep(_seconds: float) -> None:
            nonlocal mock_sleep_called
            mock_sleep_called = True
        ctx = test_context.make_test_context()
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', mock_overpass_query_need_sleep):
            with unittest.mock.patch('time.sleep', mock_sleep):
                cron.overpass_sleep(ctx)
                self.assertFalse(mock_sleep_called)

    def test_need_sleep(self) -> None:
        """Tests the case when sleep is needed."""
        sleep_for = 42

        def mock_overpass_query_need_sleep(_conf: context.Context) -> int:
            nonlocal sleep_for
            if sleep_for > 0:
                sleep_for = 0
                return 42
            return sleep_for
        captured_seconds = 0.0

        def mock_sleep(seconds: float) -> None:
            nonlocal captured_seconds
            captured_seconds = seconds
        ctx = test_context.make_test_context()
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', mock_overpass_query_need_sleep):
            with unittest.mock.patch('time.sleep', mock_sleep):
                cron.overpass_sleep(ctx)
                self.assertEqual(captured_seconds, 42.0)


class TestUpdateRefHousenumbers(unittest.TestCase):
    """Tests update_ref_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            if relation_name not in ("gazdagret", "ujbuda"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "street-housenumbers-reference-gazdagret.lst")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_ref_housenumbers(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_ref_housenumbers(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure housenumber ref is not created for the streets=only case.
        ujbuda_path = os.path.join(relations.get_workdir(), "street-housenumbers-reference-ujbuda.lst")
        self.assertFalse(os.path.exists(ujbuda_path))


class TestUpdateRefStreets(unittest.TestCase):
    """Tests update_ref_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            # gellerthegy is streets=no
            if relation_name not in ("gazdagret", "gellerthegy"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "streets-reference-gazdagret.lst")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_ref_streets(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_ref_streets(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure street ref is not created for the streets=no case.
        ujbuda_path = os.path.join(relations.get_workdir(), "streets-reference-gellerthegy.lst")
        self.assertFalse(os.path.exists(ujbuda_path))


class TestUpdateMissingHousenumbers(unittest.TestCase):
    """Tests update_missing_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            # ujbuda is streets=only
            if relation_name not in ("gazdagret", "ujbuda"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "gazdagret.percent")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_missing_housenumbers(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_missing_housenumbers(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure housenumber stat is not created for the streets=only case.
        self.assertFalse(os.path.exists(os.path.join(relations.get_workdir(), "ujbuda.percent")))


class TestUpdateMissingStreets(unittest.TestCase):
    """Tests update_missing_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            # gellerthegy is streets=no
            if relation_name not in ("gazdagret", "gellerthegy"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "gazdagret-streets.percent")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_missing_streets(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_missing_streets(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure street stat is not created for the streets=no case.
        self.assertFalse(os.path.exists(os.path.join(relations.get_workdir(), "gellerthegy-streets.percent")))


class TestUpdateAdditionalStreets(unittest.TestCase):
    """Tests update_additional_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            # gellerthegy is streets=no
            if relation_name not in ("gazdagret", "gellerthegy"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "gazdagret-additional-streets.count")
        expected = "1"
        if os.path.exists(path):
            util.get_content(path)
            os.unlink(path)
        cron.update_additional_streets(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_additional_streets(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path).decode("utf-8")
        self.assertEqual(actual, expected)
        # Make sure street stat is not created for the streets=no case.
        self.assertFalse(os.path.exists(os.path.join(relations.get_workdir(), "gellerthegy-additional-streets.count")))


class TestUpdateOsmHousenumbers(unittest.TestCase):
    """Tests update_osm_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-housenumbers-gazdagret.csv"),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            if relation_name != "gazdagret":
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "street-housenumbers-gazdagret.csv")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_osm_housenumbers(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_osm_housenumbers(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)

    def test_http_error(self) -> None:
        """Tests the case when we keep getting HTTP errors."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path=""),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            if relation_name != "gazdagret":
                relations.get_relation(relation_name).get_config().set_active(False)
        expected = util.get_content(relations.get_workdir(), "street-housenumbers-gazdagret.csv")
        cron.update_osm_housenumbers(ctx, relations, update=True)
        # Make sure that in case we keep getting errors we give up at some stage and
        # leave the last state unchanged.
        actual = util.get_content(relations.get_workdir(), "street-housenumbers-gazdagret.csv")
        self.assertEqual(actual, expected)


class TestUpdateOsmStreets(unittest.TestCase):
    """Tests update_osm_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-streets-gazdagret.csv"),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            if relation_name != "gazdagret":
                relations.get_relation(relation_name).get_config().set_active(False)
        expected = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
        path = os.path.join(relations.get_workdir(), "streets-gazdagret.csv")
        os.unlink(path)
        cron.update_osm_streets(ctx, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_osm_streets(ctx, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
        self.assertEqual(actual, expected)

    def test_http_error(self) -> None:
        """Tests the case when we keep getting HTTP errors."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path=""),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        relations = areas.Relations(ctx)
        for relation_name in relations.get_active_names():
            if relation_name != "gazdagret":
                relations.get_relation(relation_name).get_config().set_active(False)
        expected = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
        cron.update_osm_streets(ctx, relations, update=True)
        # Make sure that in case we keep getting errors we give up at some stage and
        # leave the last state unchanged.
        actual = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
        self.assertEqual(actual, expected)


def create_old_file(path: str) -> None:
    """Creates a 8 days old file."""
    current_time = time.time()
    old_time = current_time - (8 * 24 * 3600)
    old_access_time = old_time
    old_modification_time = old_time
    open(path, "w").close()
    os.utime(path, (old_access_time, old_modification_time))


class TestUpdateStats(unittest.TestCase):
    """Tests update_stats()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-stats.csv"),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)

        # Create a CSV that is definitely old enough to be removed.
        old_path = ctx.get_abspath("workdir/stats/old.csv")
        create_old_file(old_path)

        today = time.strftime("%Y-%m-%d")
        path = ctx.get_abspath("workdir/stats/%s.csv" % today)
        cron.update_stats(ctx, overpass=True)
        actual = util.get_content(path)
        self.assertEqual(actual, util.get_content("tests/network/overpass-stats.csv"))

        # Make sure that the old CSV is removed.
        self.assertFalse(os.path.exists(old_path))

        with open(ctx.get_abspath("workdir/stats/ref.count"), "r") as stream:
            num_ref = int(stream.read().strip())
        self.assertEqual(num_ref, 300)

    def test_http_error(self) -> None:
        """Tests the case when we keep getting HTTP errors."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        old_mtime = ctx.get_file_system().getmtime(ctx.get_abspath("workdir/stats/stats.json"))
        cron.update_stats(ctx, overpass=True)
        new_mtime = ctx.get_file_system().getmtime(ctx.get_abspath("workdir/stats/stats.json"))
        self.assertGreater(new_mtime, old_mtime)

    def test_no_overpass(self) -> None:
        """Tests the case when we don't call overpass."""
        ctx = test_context.make_test_context()
        ctx.set_time(test_context.make_test_time())
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            cron.update_stats(ctx, overpass=False)
        self.assertFalse(mock_overpass_sleep_called)


class TestOurMain(unittest.TestCase):
    """Tests our_main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        calls = 0

        def count_calls(_ctx: context.Context, _relations: areas.Relation, _update: bool) -> None:
            nonlocal calls
            calls += 1

        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        with unittest.mock.patch("cron.update_osm_streets", count_calls):
            with unittest.mock.patch("cron.update_osm_housenumbers", count_calls):
                with unittest.mock.patch("cron.update_ref_streets", count_calls):
                    with unittest.mock.patch("cron.update_ref_housenumbers", count_calls):
                        with unittest.mock.patch("cron.update_missing_streets", count_calls):
                            with unittest.mock.patch("cron.update_missing_housenumbers", count_calls):
                                with unittest.mock.patch("cron.update_additional_streets", count_calls):
                                    cron.our_main(ctx, relations, mode="relations", update=True, overpass=True)

        expected = 0
        # Consider what to update automatically: the 2 sources and the diff between them.
        for _ in ("osm", "ref", "missing"):
            # What object types we have.
            for _ in ("streets", "housenumbers"):
                expected += 1
        # "additional" is streets-only
        expected += 1

        self.assertEqual(calls, expected)

    def test_stats(self) -> None:
        """Tests the stats path."""
        calls = 0

        def count_calls(_ctx: context.Context, _overpass: bool) -> None:
            nonlocal calls
            calls += 1

        ctx = test_context.make_test_context()
        relations = areas.Relations(ctx)
        with unittest.mock.patch("cron.update_stats", count_calls):
            cron.our_main(ctx, relations, mode="stats", update=False, overpass=True)

        self.assertEqual(calls, 1)


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        mock_main_called = False

        def mock_main(
            _ctx: context.Context,
            _relations: areas.Relation,
            _mode: str,
            _update: bool,
            _overpass: bool
        ) -> None:
            nonlocal mock_main_called
            mock_main_called = True

        mock_info_called = False

        def mock_info(_msg: str, *_args: Any, **_kwargs: Any) -> None:
            nonlocal mock_info_called
            mock_info_called = True

        ctx = test_context.make_test_context()
        with unittest.mock.patch("cron.our_main", mock_main):
            with unittest.mock.patch("logging.info", mock_info):
                argv = [""]
                with unittest.mock.patch('sys.argv', argv):
                    cron.main(ctx)

        self.assertTrue(mock_main_called)
        self.assertTrue(mock_info_called)

    def test_exception(self) -> None:
        """Tests the path when main() throws."""
        def mock_our_main(_relations: areas.Relation) -> None:
            raise Exception()

        def mock_info(_msg: str, *_args: Any, **_kwargs: Any) -> None:
            pass

        mock_error_called = False

        def mock_error(_msg: str, *_args: Any, **_kwargs: Any) -> None:
            nonlocal mock_error_called
            mock_error_called = True

        ctx = test_context.make_test_context()
        with unittest.mock.patch("cron.our_main", mock_our_main):
            with unittest.mock.patch("logging.info", mock_info):
                with unittest.mock.patch("logging.error", mock_error):
                    argv = [""]
                    with unittest.mock.patch('sys.argv', argv):
                        cron.main(ctx)

        self.assertTrue(mock_error_called)


if __name__ == '__main__':
    unittest.main()
