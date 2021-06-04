#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_cron module covers the cron module."""

from typing import Any
from typing import BinaryIO
from typing import Optional
import datetime
import io
import os
import time
import unittest
import unittest.mock
import urllib.error

import test_config

import areas
import config
import cron
import util


def mock_make_config() -> config.Config2:
    """Creates a Config instance that has its root as /tests."""
    return config.Config2("tests")


def mock_urlopen_raise_error(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
    """Mock urlopen(), always throwing an error."""
    raise urllib.error.HTTPError(url="", code=0, msg="", hdrs={}, fp=io.BytesIO())


class TestOverpassSleep(unittest.TestCase):
    """Tests overpass_sleep()."""
    def test_no_sleep(self) -> None:
        """Tests the case when no sleep is needed."""
        def mock_overpass_query_need_sleep() -> int:
            return 0
        mock_sleep_called = False

        def mock_sleep(_seconds: float) -> None:
            nonlocal mock_sleep_called
            mock_sleep_called = True
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', mock_overpass_query_need_sleep):
            with unittest.mock.patch('time.sleep', mock_sleep):
                cron.overpass_sleep()
                self.assertFalse(mock_sleep_called)

    def test_need_sleep(self) -> None:
        """Tests the case when sleep is needed."""
        sleep_for = 42

        def mock_overpass_query_need_sleep() -> int:
            nonlocal sleep_for
            if sleep_for > 0:
                sleep_for = 0
                return 42
            return sleep_for
        captured_seconds = 0.0

        def mock_sleep(seconds: float) -> None:
            nonlocal captured_seconds
            captured_seconds = seconds
        with unittest.mock.patch('overpass_query.overpass_query_need_sleep', mock_overpass_query_need_sleep):
            with unittest.mock.patch('time.sleep', mock_sleep):
                cron.overpass_sleep()
                self.assertEqual(captured_seconds, 42.0)


class TestUpdateRefHousenumbers(test_config.TestCase):
    """Tests update_ref_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        for relation_name in relations.get_active_names():
            if relation_name not in ("gazdagret", "ujbuda"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "street-housenumbers-reference-gazdagret.lst")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_ref_housenumbers(conf, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_ref_housenumbers(conf, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure housenumber ref is not created for the streets=only case.
        ujbuda_path = os.path.join(relations.get_workdir(), "street-housenumbers-reference-ujbuda.lst")
        self.assertFalse(os.path.exists(ujbuda_path))


class TestUpdateRefStreets(test_config.TestCase):
    """Tests update_ref_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        for relation_name in relations.get_active_names():
            # gellerthegy is streets=no
            if relation_name not in ("gazdagret", "gellerthegy"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "streets-reference-gazdagret.lst")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_ref_streets(conf, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_ref_streets(conf, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure street ref is not created for the streets=no case.
        ujbuda_path = os.path.join(relations.get_workdir(), "streets-reference-gellerthegy.lst")
        self.assertFalse(os.path.exists(ujbuda_path))


class TestUpdateMissingHousenumbers(test_config.TestCase):
    """Tests update_missing_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        for relation_name in relations.get_active_names():
            # ujbuda is streets=only
            if relation_name not in ("gazdagret", "ujbuda"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "gazdagret.percent")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_missing_housenumbers(conf, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_missing_housenumbers(conf, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure housenumber stat is not created for the streets=only case.
        self.assertFalse(os.path.exists(os.path.join(relations.get_workdir(), "ujbuda.percent")))


class TestUpdateMissingStreets(test_config.TestCase):
    """Tests update_missing_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        for relation_name in relations.get_active_names():
            # gellerthegy is streets=no
            if relation_name not in ("gazdagret", "gellerthegy"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "gazdagret-streets.percent")
        expected = util.get_content(path)
        os.unlink(path)
        cron.update_missing_streets(conf, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_missing_streets(conf, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path)
        self.assertEqual(actual, expected)
        # Make sure street stat is not created for the streets=no case.
        self.assertFalse(os.path.exists(os.path.join(relations.get_workdir(), "gellerthegy-streets.percent")))


class TestUpdateAdditionalStreets(test_config.TestCase):
    """Tests update_additional_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        for relation_name in relations.get_active_names():
            # gellerthegy is streets=no
            if relation_name not in ("gazdagret", "gellerthegy"):
                relations.get_relation(relation_name).get_config().set_active(False)
        path = os.path.join(relations.get_workdir(), "gazdagret-additional-streets.count")
        expected = "1"
        if os.path.exists(path):
            util.get_content(path)
            os.unlink(path)
        cron.update_additional_streets(conf, relations, update=True)
        mtime = os.path.getmtime(path)
        cron.update_additional_streets(conf, relations, update=False)
        self.assertEqual(os.path.getmtime(path), mtime)
        actual = util.get_content(path).decode("utf-8")
        self.assertEqual(actual, expected)
        # Make sure street stat is not created for the streets=no case.
        self.assertFalse(os.path.exists(os.path.join(relations.get_workdir(), "gellerthegy-additional-streets.count")))


class TestUpdateOsmHousenumbers(test_config.TestCase):
    """Tests update_osm_housenumbers()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        result_from_overpass = "@id\taddr:street\taddr:housenumber\taddr:postcode\taddr:housename\t"
        result_from_overpass += "addr:conscriptionnumber\taddr:flats\taddr:floor\taddr:door\taddr:unit\tname\t@type\n\n"
        result_from_overpass += "1\tTörökugrató utca\t1\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tTörökugrató utca\t2\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tTűzkő utca\t9\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tTűzkő utca\t10\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tOSM Name 1\t1\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tOSM Name 1\t2\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tOnly In OSM utca\t1\t\t\t\t\t\t\t\t\tnode\n"
        result_from_overpass += "1\tSecond Only In OSM utca\t1\t\t\t\t\t\t\t\t\tnode\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf

        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
                for relation_name in relations.get_active_names():
                    if relation_name != "gazdagret":
                        relations.get_relation(relation_name).get_config().set_active(False)
                path = os.path.join(relations.get_workdir(), "street-housenumbers-gazdagret.csv")
                expected = util.get_content(path)
                os.unlink(path)
                cron.update_osm_housenumbers(conf, relations, update=True)
                mtime = os.path.getmtime(path)
                cron.update_osm_housenumbers(conf, relations, update=False)
                self.assertEqual(os.path.getmtime(path), mtime)
                self.assertTrue(mock_overpass_sleep_called)
                actual = util.get_content(path)
                self.assertEqual(actual, expected)

    def test_http_error(self) -> None:
        """Tests the case when we keep getting HTTP errors."""
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('urllib.request.urlopen', mock_urlopen_raise_error):
                for relation_name in relations.get_active_names():
                    if relation_name != "gazdagret":
                        relations.get_relation(relation_name).get_config().set_active(False)
                expected = util.get_content(relations.get_workdir(), "street-housenumbers-gazdagret.csv")
                cron.update_osm_housenumbers(conf, relations, update=True)
                self.assertTrue(mock_overpass_sleep_called)
                # Make sure that in case we keep getting errors we give up at some stage and
                # leave the last state unchanged.
                actual = util.get_content(relations.get_workdir(), "street-housenumbers-gazdagret.csv")
                self.assertEqual(actual, expected)


class TestUpdateOsmStreets(test_config.TestCase):
    """Tests update_osm_streets()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        result_from_overpass = "@id\tname\n1\tTűzkő utca\n2\tTörökugrató utca\n3\tOSM Name 1\n4\tHamzsabégi út\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf

        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
                for relation_name in relations.get_active_names():
                    if relation_name != "gazdagret":
                        relations.get_relation(relation_name).get_config().set_active(False)
                expected = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
                path = os.path.join(relations.get_workdir(), "streets-gazdagret.csv")
                os.unlink(path)
                cron.update_osm_streets(conf, relations, update=True)
                mtime = os.path.getmtime(path)
                cron.update_osm_streets(conf, relations, update=False)
                self.assertEqual(os.path.getmtime(path), mtime)
                self.assertTrue(mock_overpass_sleep_called)
                actual = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
                self.assertEqual(actual, expected)

    def test_http_error(self) -> None:
        """Tests the case when we keep getting HTTP errors."""
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('urllib.request.urlopen', mock_urlopen_raise_error):
                for relation_name in relations.get_active_names():
                    if relation_name != "gazdagret":
                        relations.get_relation(relation_name).get_config().set_active(False)
                expected = util.get_content(relations.get_workdir(), "streets-gazdagret.csv")
                cron.update_osm_streets(conf, relations, update=True)
                self.assertTrue(mock_overpass_sleep_called)
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


class MockDate(datetime.date):
    """Mock datetime.date."""
    @classmethod
    def today(cls) -> 'MockDate':
        """Returns today's date."""
        return cls(2020, 5, 10)


class TestUpdateStats(test_config.TestCase):
    """Tests update_stats()."""
    def test_happy(self) -> None:
        """Tests the happy path."""

        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        result_from_overpass = "@id\taddr:postcode\naddr:city\taddr:street\taddr:housenumber\t@user\n"
        result_from_overpass += "7677\tOrfű\tDollár utca\t1\tvasony\n"
        result_from_overpass += "7677\tOrfű\tDollár utca\t2\tvasony\n"

        def mock_urlopen(_url: str, _data: Optional[bytes] = None) -> BinaryIO:
            buf = io.BytesIO()
            buf.write(result_from_overpass.encode('utf-8'))
            buf.seek(0)
            return buf

        # Create a CSV that is definitely old enough to be removed.
        old_path = config.get_abspath("workdir/stats/old.csv")
        create_old_file(old_path)

        today = time.strftime("%Y-%m-%d")
        path = config.get_abspath("workdir/stats/%s.csv" % today)
        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('urllib.request.urlopen', mock_urlopen):
                with unittest.mock.patch('datetime.date', MockDate):
                    cron.update_stats(overpass=True)
        actual = util.get_content(path).decode("utf-8")
        self.assertEqual(actual, result_from_overpass)

        # Make sure that the old CSV is removed.
        self.assertFalse(os.path.exists(old_path))

        self.assertTrue(mock_overpass_sleep_called)

        with open(config.get_abspath("workdir/stats/ref.count"), "r") as stream:
            num_ref = int(stream.read().strip())
        self.assertEqual(num_ref, 300)

    def test_http_error(self) -> None:
        """Tests the case when we keep getting HTTP errors."""
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('urllib.request.urlopen', mock_urlopen_raise_error):
                with unittest.mock.patch('datetime.date', MockDate):
                    cron.update_stats(overpass=True)
        self.assertTrue(mock_overpass_sleep_called)

    def test_no_overpass(self) -> None:
        """Tests the case when we don't call overpass."""
        mock_overpass_sleep_called = False

        def mock_overpass_sleep() -> None:
            nonlocal mock_overpass_sleep_called
            mock_overpass_sleep_called = True

        with unittest.mock.patch("cron.overpass_sleep", mock_overpass_sleep):
            with unittest.mock.patch('datetime.date', MockDate):
                cron.update_stats(overpass=False)
        self.assertFalse(mock_overpass_sleep_called)


class TestOurMain(test_config.TestCase):
    """Tests our_main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        calls = 0

        def count_calls(_conf: config.Config2, _relations: areas.Relation, _update: bool) -> None:
            nonlocal calls
            calls += 1

        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        with unittest.mock.patch("cron.update_osm_streets", count_calls):
            with unittest.mock.patch("cron.update_osm_housenumbers", count_calls):
                with unittest.mock.patch("cron.update_ref_streets", count_calls):
                    with unittest.mock.patch("cron.update_ref_housenumbers", count_calls):
                        with unittest.mock.patch("cron.update_missing_streets", count_calls):
                            with unittest.mock.patch("cron.update_missing_housenumbers", count_calls):
                                with unittest.mock.patch("cron.update_additional_streets", count_calls):
                                    cron.our_main(conf, relations, mode="relations", update=True, overpass=True)

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

        def count_calls(_overpass: bool) -> None:
            nonlocal calls
            calls += 1

        conf = mock_make_config()
        relations = areas.Relations(conf.get_workdir())
        with unittest.mock.patch("cron.update_stats", count_calls):
            cron.our_main(conf, relations, mode="stats", update=False, overpass=True)

        self.assertEqual(calls, 1)


class TestMain(test_config.TestCase):
    """Tests main()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        mock_main_called = False

        def mock_main(
            _conf: config.Config2,
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

        with unittest.mock.patch("cron.our_main", mock_main):
            with unittest.mock.patch("logging.info", mock_info):
                argv = [""]
                with unittest.mock.patch('sys.argv', argv):
                    with unittest.mock.patch("config.make_config", mock_make_config):
                        cron.main()

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

        with unittest.mock.patch("cron.our_main", mock_our_main):
            with unittest.mock.patch("logging.info", mock_info):
                with unittest.mock.patch("logging.error", mock_error):
                    argv = [""]
                    with unittest.mock.patch('sys.argv', argv):
                        with unittest.mock.patch("config.make_config", mock_make_config):
                            cron.main()

        self.assertTrue(mock_error_called)


if __name__ == '__main__':
    unittest.main()
