#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The webframe module provides the header, toolbar and footer code."""

from typing import BinaryIO
from typing import Dict
from typing import List
from typing import Optional
from typing import Tuple

import rust
import yattag


def get_footer(last_updated: str) -> yattag.Doc:
    """Produces the end of the page."""
    return rust.py_get_footer(last_updated)


def fill_missing_header_items(
    ctx: rust.PyContext,
    streets: str,
    additional_housenumbers: bool,
    relation_name: str,
    items: List[yattag.Doc]
) -> List[yattag.Doc]:
    """Generates the 'missing house numbers/streets' part of the header."""
    return rust.py_fill_missing_header_items(ctx, streets, additional_housenumbers, relation_name, items)


def get_toolbar(
        ctx: rust.PyContext,
        relations: Optional[rust.PyRelations],
        function: str,
        relation_name: str,
        relation_osmid: int
) -> yattag.Doc:
    """Produces the start of the page. Note that the content depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    return rust.py_get_toolbar(ctx, relations, function, relation_name, relation_osmid)


def handle_static(ctx: rust.PyContext, request_uri: str) -> Tuple[bytes, str, List[Tuple[str, str]]]:
    """Handles serving static content."""
    return rust.py_handle_static(ctx, request_uri)


def make_response(content_type: str, status: str, output_bytes: bytes, headers: List[Tuple[str, str]]) -> rust.PyResponse:
    """Factory for rust.PyResponse."""
    return rust.PyResponse(content_type, status, output_bytes, headers)


def send_response(
        environ: Dict[str, str],
        response: rust.PyResponse
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Turns an output string into a byte array and sends it."""
    return rust.py_send_response(environ, response)


def handle_exception(
        environ: Dict[str, str],
        error: str
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Displays an unhandled exception on the page."""
    return rust.py_handle_exception(environ, error)


def handle_404() -> yattag.Doc:
    """Displays a not-found page."""
    return rust.py_handle_404()


def format_timestamp(timestamp: float) -> str:
    """Formats timestamp as UI date-time."""
    return rust.py_format_timestamp(timestamp)


def handle_stats(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/."""
    return rust.py_handle_stats(ctx, relations, request_uri)


def get_request_uri(environ: Dict[str, str], ctx: rust.PyContext, relations: rust.PyRelations) -> str:
    """Finds out the request URI."""
    return rust.py_get_request_uri(environ, ctx, relations)


def check_existing_relation(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Prevents serving outdated data from a relation that has been renamed."""
    return rust.py_check_existing_relation(ctx, relations, request_uri)


def handle_no_osm_streets(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-osm-streets error on a page using JS."""
    return rust.py_handle_no_osm_streets(prefix, relation_name)


def handle_no_osm_housenumbers(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    return rust.py_handle_no_osm_housenumbers(prefix, relation_name)


def handle_no_ref_housenumbers(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    return rust.py_handle_no_ref_housenumbers(prefix, relation_name)


def handle_no_ref_streets(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-ref-streets error on a page using JS."""
    return rust.py_handle_no_ref_streets(prefix, relation_name)


def handle_github_webhook(stream: BinaryIO, ctx: rust.PyContext) -> yattag.Doc:
    """Handles a GitHub style webhook."""
    return rust.py_handle_github_webhook(stream, ctx)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
