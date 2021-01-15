#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi_additional module covers the wsgi_additional module."""

from typing import List
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import os
import unittest
import unittest.mock

import test_config

import areas
import config
import wsgi

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def get_relations() -> areas.Relations:
    """Returns a Relations object that uses the test data and workdir."""
    workdir = os.path.join(os.path.dirname(__file__), "workdir")
    return areas.Relations(workdir)


class TestWsgi(test_config.TestCase):
    """Base class for wsgi tests."""
    def get_txt_for_path(self, path: str) -> str:
        """Generates a string for a given wsgi path."""
        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            # Make sure the built-in exception catcher is not kicking in.
            self.assertEqual(status, "200 OK")
            header_dict = dict(response_headers)
            if path.endswith(".chkl"):
                self.assertEqual(header_dict["Content-type"], "application/octet-stream")
            else:
                self.assertEqual(header_dict["Content-type"], "text/plain; charset=utf-8")

        prefix = config.Config.get_uri_prefix()
        environ = {
            "PATH_INFO": prefix + path
        }
        callback = cast('StartResponse', start_response)
        output_iterable = wsgi.application(environ, callback)
        output_list = cast(List[bytes], output_iterable)
        self.assertTrue(output_list)
        output = output_list[0].decode('utf-8')
        return output


class TestStreets(TestWsgi):
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
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_osm_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "No existing streets")

    def test_view_result_txt_no_ref_streets(self) -> None:
        """Tests the txt output, no ref streets case."""
        relations = get_relations()
        relation = relations.get_relation("gazdagret")
        hide_path = relation.get_files().get_ref_streets_path()
        real_exists = os.path.exists

        def mock_exists(path: str) -> bool:
            if path == hide_path:
                return False
            return real_exists(path)
        with unittest.mock.patch('os.path.exists', mock_exists):
            result = self.get_txt_for_path("/additional-streets/gazdagret/view-result.txt")
            self.assertEqual(result, "No reference streets")


if __name__ == '__main__':
    unittest.main()
