#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The webframe module provides the header, toolbar and footer code."""

from typing import Dict
from typing import List
from typing import Tuple

import rust
import yattag


def fill_missing_header_items(
    ctx: rust.PyContext,
    streets: str,
    additional_housenumbers: bool,
    relation_name: str,
    items: List[yattag.Doc]
) -> List[yattag.Doc]:
    """Generates the 'missing house numbers/streets' part of the header."""
    return rust.py_fill_missing_header_items(ctx, streets, additional_housenumbers, relation_name, items)


def handle_static(ctx: rust.PyContext, request_uri: str) -> Tuple[bytes, str, List[Tuple[str, str]]]:
    """Handles serving static content."""
    return rust.py_handle_static(ctx, request_uri)


def handle_exception(
        environ: Dict[str, str],
        error: str
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Displays an unhandled exception on the page."""
    return rust.py_handle_exception(environ, error)


def get_request_uri(environ: Dict[str, str], ctx: rust.PyContext, relations: rust.PyRelations) -> str:
    """Finds out the request URI."""
    return rust.py_get_request_uri(environ, ctx, relations)


def handle_github_webhook(data: bytes, ctx: rust.PyContext) -> yattag.Doc:
    """Handles a GitHub style webhook."""
    return rust.py_handle_github_webhook(data, ctx)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
