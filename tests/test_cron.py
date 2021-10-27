#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cron module covers the cron module."""

from typing import List
import io
import json
import os
import time
import unittest

import test_context

import areas
import cron
import util


class TestUpdateOsmStreets(unittest.TestCase):
    """Tests update_osm_streets()."""
    def test_xml_as_csv(self) -> None:
        """Tests the case when we ask for CSV but get XML."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass.xml"),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        relations = areas.make_relations(ctx)
        for relation_name in relations.get_active_names():
            if relation_name != "gazdagret":
                relation = relations.get_relation(relation_name)
                config = relation.get_config()
                config.set_active(False)
                relation.set_config(config)
                relations.set_relation(relation_name, relation)
        path = os.path.join(relations.get_workdir(), "streets-gazdagret.csv")
        expected = util.get_content(path)
        cron.update_osm_streets(ctx, relations, update=True)
        self.assertEqual(util.get_content(path), expected)


def create_old_file(path: str) -> None:
    """Creates a 8 days old file."""
    current_time = time.time()
    old_time = current_time - (8 * 24 * 3600)
    old_access_time = old_time
    old_modification_time = old_time
    with open(path, "w"):
        pass
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

        file_system = test_context.TestFileSystem()
        citycount_value = io.BytesIO()
        citycount_value.__setattr__("close", lambda: None)
        count_value = io.BytesIO()
        count_value.__setattr__("close", lambda: None)
        topusers_value = io.BytesIO()
        topusers_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): citycount_value,
            ctx.get_abspath("workdir/stats/2020-05-10.count"): count_value,
            ctx.get_abspath("workdir/stats/2020-05-10.topusers"): topusers_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

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

        file_system = test_context.TestFileSystem()
        citycount_value = io.BytesIO()
        citycount_value.__setattr__("close", lambda: None)
        count_value = io.BytesIO()
        count_value.__setattr__("close", lambda: None)
        topusers_value = io.BytesIO()
        topusers_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): citycount_value,
            ctx.get_abspath("workdir/stats/2020-05-10.count"): count_value,
            ctx.get_abspath("workdir/stats/2020-05-10.topusers"): topusers_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        old_mtime = ctx.get_file_system().getmtime(ctx.get_abspath("workdir/stats/stats.json"))
        time.sleep(0.1)
        cron.update_stats(ctx, overpass=True)
        new_mtime = ctx.get_file_system().getmtime(ctx.get_abspath("workdir/stats/stats.json"))
        self.assertGreater(new_mtime, old_mtime)

    def test_no_overpass(self) -> None:
        """Tests the case when we don't call overpass."""
        ctx = test_context.make_test_context()
        test_time = test_context.make_test_time()
        ctx.set_time(test_time)
        routes: List[test_context.URLRoute] = [
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-wait.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt")
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)

        file_system = test_context.TestFileSystem()
        citycount_value = io.BytesIO()
        citycount_value.__setattr__("close", lambda: None)
        count_value = io.BytesIO()
        count_value.__setattr__("close", lambda: None)
        topusers_value = io.BytesIO()
        topusers_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): citycount_value,
            ctx.get_abspath("workdir/stats/2020-05-10.count"): count_value,
            ctx.get_abspath("workdir/stats/2020-05-10.topusers"): topusers_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats(ctx, overpass=False)

        self.assertEqual(test_time.get_sleep(), 0)


class TestOurMain(unittest.TestCase):
    """Tests our_main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        routes: List[test_context.URLRoute] = [
            # For update_osm_streets().
            test_context.URLRoute(url="https://overpass-api.de/api/status",
                                  data_path="",
                                  result_path="tests/network/overpass-status-happy.txt"),
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-streets-gazdagret.csv"),
            # For update_osm_housenumbers().
            test_context.URLRoute(url="https://overpass-api.de/api/interpreter",
                                  data_path="",
                                  result_path="tests/network/overpass-housenumbers-gazdagret.csv"),
        ]
        network = test_context.TestNetwork(routes)
        ctx.set_network(network)
        file_system = test_context.TestFileSystem()
        yamls_cache = {
            "relations.yaml": {
                "gazdagret": {
                    "osmrelation": 2713748,
                    "refcounty": "01",
                    "refsettlement": "011",
                },
            },
            "refcounty-names.yaml": {
            },
            "refsettlement-names.yaml": {
            },
        }
        yamls_cache_value = io.BytesIO()
        yamls_cache_value.__setattr__("close", lambda: None)
        yamls_cache_value.write(util.to_bytes(json.dumps(yamls_cache)))
        yamls_cache_value.seek(0)
        osm_streets_value = io.BytesIO()
        osm_streets_value.__setattr__("close", lambda: None)
        osm_housenumbers_value = io.BytesIO()
        osm_housenumbers_value.__setattr__("close", lambda: None)
        ref_streets_value = io.BytesIO()
        ref_streets_value.__setattr__("close", lambda: None)
        ref_housenumbers_value = io.BytesIO()
        ref_housenumbers_value.__setattr__("close", lambda: None)
        missing_streets_value = io.BytesIO()
        missing_streets_value.__setattr__("close", lambda: None)
        missing_housenumbers_value = io.BytesIO()
        missing_housenumbers_value.__setattr__("close", lambda: None)
        additional_streets_value = io.BytesIO()
        additional_streets_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("data/yamls.cache"): yamls_cache_value,
            ctx.get_abspath("workdir/streets-gazdagret.csv"): osm_streets_value,
            ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv"): osm_housenumbers_value,
            ctx.get_abspath("workdir/streets-reference-gazdagret.lst"): ref_streets_value,
            ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"): ref_housenumbers_value,
            ctx.get_abspath("workdir/gazdagret-streets.percent"): missing_streets_value,
            ctx.get_abspath("workdir/gazdagret.percent"): missing_housenumbers_value,
            ctx.get_abspath("workdir/gazdagret-additional-streets.count"): additional_streets_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        relations = areas.make_relations(ctx)

        cron.our_main(ctx, relations, mode="relations", update=True, overpass=True)

        # update_osm_streets() is called.
        self.assertTrue(osm_streets_value.tell())
        # update_osm_housenumbers() is called.
        self.assertTrue(osm_housenumbers_value.tell())
        # update_ref_streets() is called.
        self.assertTrue(ref_streets_value.tell())
        # update_ref_housenumbers() is called.
        self.assertTrue(ref_housenumbers_value.tell())
        # update_missing_streets() is called.
        self.assertTrue(missing_streets_value.tell())
        # update_missing_housenumbers() is called.
        self.assertTrue(missing_housenumbers_value.tell())
        # update_additional_streets() is called.
        self.assertTrue(additional_streets_value.tell())

    def test_stats(self) -> None:
        """Tests the stats path."""
        ctx = test_context.make_test_context()
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
        file_system = test_context.TestFileSystem()
        stats_value = io.BytesIO()
        stats_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/stats.json"): stats_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        relations = areas.make_relations(ctx)
        cron.our_main(ctx, relations, mode="stats", update=False, overpass=True)

        self.assertTrue(stats_value.tell())


class TestMain(unittest.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        stats_value = io.BytesIO()
        stats_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/stats.json"): stats_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        argv = ["", "--mode", "stats", "--no-overpass"]
        buf = io.BytesIO()
        buf.__setattr__("close", lambda: None)

        cron.main(argv, buf, ctx)

        # Make sure that stats.json is updated without an error.
        self.assertTrue(stats_value.tell())
        self.assertNotIn(b"ERROR", buf.getvalue())

    def test_error(self) -> None:
        """Tests the path when our_main() returns an error."""
        ctx = test_context.make_test_context()
        ctx.set_unit(test_context.TestUnit())
        file_system = test_context.TestFileSystem()
        log_value = io.BytesIO()
        log_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/cron.log"): log_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)
        argv = ["", "--mode", "stats", "--no-overpass"]
        buf = io.BytesIO()
        buf.__setattr__("close", lambda: None)

        cron.main(argv, buf, ctx)

        # No logging initialized -> no output.
        self.assertEqual(buf.getvalue(), b"")


class TestUpdateStatsCount(unittest.TestCase):
    """Tests update_stats_count()."""
    def test_happy(self) -> None:
        """Tests tha happy path."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        today_csv = util.to_bytes("""addr:postcode	addr:city	addr:street	addr:housenumber	@user
7677	Orfű	Dollár utca	1	mgpx
""")
        today_csv_value = io.BytesIO(today_csv)
        today_csv_value.__setattr__("close", lambda: None)
        today_count_value = io.BytesIO()
        today_count_value.__setattr__("close", lambda: None)
        today_citycount_value = io.BytesIO()
        today_citycount_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.csv"): today_csv_value,
            ctx.get_abspath("workdir/stats/2020-05-10.count"): today_count_value,
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): today_citycount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats_count(ctx, "2020-05-10")

        self.assertTrue(today_count_value.tell())
        self.assertTrue(today_citycount_value.tell())

    def test_no_csv(self) -> None:
        """Tests the case then the .csv is missing."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        today_count_value = io.BytesIO()
        today_count_value.__setattr__("close", lambda: None)
        today_citycount_value = io.BytesIO()
        today_citycount_value.__setattr__("close", lambda: None)
        file_system.set_hide_paths([ctx.get_abspath("workdir/stats/2020-05-10.csv")])
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.count"): today_count_value,
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): today_citycount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats_count(ctx, "2020-05-10")

        # No .csv, no .count or .citycount.
        self.assertFalse(today_count_value.tell())
        self.assertFalse(today_citycount_value.tell())


class TestUpdateStatsTopusers(unittest.TestCase):
    """Tests update_stats_topuers()."""
    def test_happy(self) -> None:
        """Tests tha happy path."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()

        today_csv = util.to_bytes("""addr:postcode	addr:city	addr:street	addr:housenumber	@user
7677	Orfű	Dollár utca	1	mgpx
""")
        today_csv_value = io.BytesIO(today_csv)
        today_csv_value.__setattr__("close", lambda: None)
        today_topusers_value = io.BytesIO()
        today_topusers_value.__setattr__("close", lambda: None)
        today_usercount_value = io.BytesIO()
        today_usercount_value.__setattr__("close", lambda: None)
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.csv"): today_csv_value,
            ctx.get_abspath("workdir/stats/2020-05-10.topusers"): today_topusers_value,
            ctx.get_abspath("workdir/stats/2020-05-10.usercount"): today_usercount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats_topusers(ctx, "2020-05-10")

        self.assertTrue(today_topusers_value.tell())
        self.assertTrue(today_usercount_value.tell())

    def test_no_csv(self) -> None:
        """Tests the case then the .csv is missing."""
        ctx = test_context.make_test_context()
        file_system = test_context.TestFileSystem()
        today_topusers_value = io.BytesIO()
        today_topusers_value.__setattr__("close", lambda: None)
        today_usercount_value = io.BytesIO()
        today_usercount_value.__setattr__("close", lambda: None)
        file_system.set_hide_paths([ctx.get_abspath("workdir/stats/2020-05-10.csv")])
        files = {
            ctx.get_abspath("workdir/stats/2020-05-10.count"): today_topusers_value,
            ctx.get_abspath("workdir/stats/2020-05-10.citycount"): today_usercount_value,
        }
        file_system.set_files(files)
        ctx.set_file_system(file_system)

        cron.update_stats_topusers(ctx, "2020-05-10")

        # No .csv, no .topusers or .usercount.
        self.assertFalse(today_topusers_value.tell())
        self.assertFalse(today_usercount_value.tell())


if __name__ == '__main__':
    unittest.main()
