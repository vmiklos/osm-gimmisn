#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_webframe module covers the webframe module."""

from typing import List
from typing import TYPE_CHECKING
from typing import Tuple
from typing import cast
import traceback
import unittest

import webframe

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse  # noqa: F401


class TestHandleException(unittest.TestCase):
    """Tests handle_exception()."""
    def test_happy(self) -> None:
        """Tests the happy path."""
        environ = {
            "PATH_INFO": "/"
        }

        def start_response(status: str, response_headers: List[Tuple[str, str]]) -> None:
            self.assertTrue(status.startswith("500"))
            header_dict = dict(response_headers)
            self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")

        try:
            int("a")
        # pylint: disable=broad-except
        except Exception:
            callback = cast('StartResponse', start_response)
            status, headers, data = webframe.handle_exception(environ, traceback.format_exc())
            callback(status, headers)
            self.assertTrue(data)
            output = data.decode('utf-8')
            self.assertIn("ValueError", output)
            return
        self.fail()


if __name__ == '__main__':
    unittest.main()
