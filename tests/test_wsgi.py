#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The test_wsgi module covers the wsgi module."""

from typing import Any
from typing import Dict
import io
import unittest
import xml.etree.ElementTree as ET
import xmlrpc.client

import test_context

import wsgi


class TestWsgi(unittest.TestCase):
    """Base class for wsgi tests."""
    def __init__(self, method_name: str) -> None:
        unittest.TestCase.__init__(self, method_name)
        self.gzip_compress = False
        self.ctx = test_context.make_test_context()
        self.environ: Dict[str, Any] = {}
        self.bytes = bytes()

    def get_dom_for_path(self, path: str, absolute: bool = False, expected_status: str = "") -> ET.Element:
        """Generates an XML DOM for a given wsgi path."""
        if not expected_status:
            expected_status = "200 OK"

        prefix = self.ctx.get_ini().get_uri_prefix()
        if not absolute:
            path = prefix + path
        self.environ["PATH_INFO"] = path
        if self.gzip_compress:
            self.environ["HTTP_ACCEPT_ENCODING"] = "gzip, deflate"
        status, response_headers, data = wsgi.application(self.environ, self.bytes, self.ctx)
        # Make sure the built-in exception catcher is not kicking in.
        self.assertEqual(status, expected_status)
        header_dict = dict(response_headers)
        self.assertEqual(header_dict["Content-type"], "text/html; charset=utf-8")
        self.assertTrue(data)
        if self.gzip_compress:
            output_bytes = xmlrpc.client.gzip_decode(data)
        else:
            output_bytes = data
        output = output_bytes.decode('utf-8')
        stream = io.StringIO(output)
        tree = ET.parse(stream)
        return tree.getroot()

    def get_txt_for_path(self, path: str) -> str:
        """Generates a string for a given wsgi path."""
        prefix = self.ctx.get_ini().get_uri_prefix()
        environ = {
            "PATH_INFO": prefix + path
        }
        status, response_headers, data = wsgi.application(environ, bytes(), self.ctx)
        # Make sure the built-in exception catcher is not kicking in.
        self.assertEqual(status, "200 OK")
        header_dict = dict(response_headers)
        if path.endswith(".chkl"):
            self.assertEqual(header_dict["Content-type"], "application/octet-stream")
        else:
            self.assertEqual(header_dict["Content-type"], "text/plain; charset=utf-8")
        self.assertTrue(data)
        output = data.decode('utf-8')
        return output
