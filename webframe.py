#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The webframe module provides the header, toolbar and footer code."""

from typing import Any
from typing import Dict
from typing import List
from typing import Optional
from typing import Tuple
import json
import os
import urllib

import yattag

from rust import py_translate as tr
import areas
import context
import rust
import util


def get_footer(last_updated: str) -> yattag.Doc:
    """Produces the end of the page."""
    return rust.py_get_footer(last_updated)


def fill_missing_header_items(
    ctx: context.Context,
    streets: str,
    additional_housenumbers: bool,
    relation_name: str,
    items: List[yattag.Doc]
) -> List[yattag.Doc]:
    """Generates the 'missing house numbers/streets' part of the header."""
    return rust.py_fill_missing_header_items(ctx, streets, additional_housenumbers, relation_name, items)


def get_toolbar(
        ctx: context.Context,
        relations: Optional[areas.Relations],
        function: str,
        relation_name: str,
        relation_osmid: int
) -> yattag.Doc:
    """Produces the start of the page. Note that the content depends on the function and the
    relation, but not on the action to keep a balance between too generic and too specific
    content."""
    return rust.py_get_toolbar(ctx, relations, function, relation_name, relation_osmid)


def handle_static(ctx: context.Context, request_uri: str) -> Tuple[bytes, str, List[Tuple[str, str]]]:
    """Handles serving static content."""
    return rust.py_handle_static(ctx, request_uri)


Response = rust.PyResponse


def send_response(
        environ: Dict[str, str],
        response: Response
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


def handle_stats(ctx: context.Context, relations: areas.Relations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/housenumber-stats/hungary/."""
    return rust.py_handle_stats(ctx, relations, request_uri)


def get_request_uri(environ: Dict[str, str], ctx: context.Context, relations: areas.Relations) -> str:
    """Finds out the request URI."""
    return rust.py_get_request_uri(environ, ctx, relations)


def check_existing_relation(ctx: context.Context, relations: areas.Relations, request_uri: str) -> yattag.Doc:
    """Prevents serving outdated data from a relation that has been renamed."""
    return rust.py_check_existing_relation(ctx, relations, request_uri)


def handle_no_osm_streets(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-osm-streets error on a page using JS."""
    return rust.py_handle_no_osm_streets(prefix, relation_name)


def handle_no_osm_housenumbers(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-osm-housenumbers error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/street-housenumbers/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-osm-housenumbers")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No existing house numbers: call Overpass to create..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-overpass-wait", tr("No existing house numbers: waiting for Overpass...")),
            ("str-overpass-error", tr("Error from Overpass: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_no_ref_housenumbers(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-ref-housenumbers error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/missing-housenumbers/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-ref-housenumbers")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No reference house numbers: create from reference..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-reference-wait", tr("No reference house numbers: creating from reference...")),
            ("str-reference-error", tr("Error from reference: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_no_ref_streets(prefix: str, relation_name: str) -> yattag.Doc:
    """Handles the no-ref-streets error on a page using JS."""
    doc = yattag.Doc()
    link = prefix + "/missing-streets/" + relation_name + "/update-result"
    with doc.tag("div", [("id", "no-ref-streets")]):
        with doc.tag("a", [("href", link)]):
            doc.text(tr("No street list: create from reference..."))
    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-reference-wait", tr("No reference streets: creating from reference...")),
            ("str-reference-error", tr("Error from reference: ")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def handle_github_webhook(environ: Dict[str, Any], ctx: context.Context) -> yattag.Doc:
    """Handles a GitHub style webhook."""

    body = urllib.parse.parse_qs(util.from_bytes(environ["wsgi.input"].read()))
    payload = body["payload"][0]
    root = json.loads(payload)
    if root["ref"] == "refs/heads/master":
        my_env: Dict[str, str] = {}
        my_env["PATH"] = "osm-gimmisn-env/bin:" + os.environ["PATH"]
        ctx.get_subprocess().run(["make", "-C", ctx.get_abspath(""), "deploy"], env=my_env)

    return yattag.Doc.from_text("")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
