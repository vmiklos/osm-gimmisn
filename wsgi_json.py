#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the json part of the web interface."""

from typing import Dict
from typing import List
from typing import Tuple

import rust
import util
import webframe


def streets_update_result_json(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/streets/ormezo/update-result.json."""
    return rust.py_streets_update_result_json(ctx, relations, request_uri)


def street_housenumbers_update_result_json(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/update-result.json."""
    return rust.py_street_housenumbers_update_result_json(ctx, relations, request_uri)


def missing_housenumbers_update_result_json(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/update-result.json."""
    return rust.py_missing_housenumbers_update_result_json(ctx, relations, request_uri)


def missing_streets_update_result_json(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/missing-streets/ormezo/update-result.json."""
    return rust.py_missing_streets_update_result_json(ctx, relations, request_uri)


def our_application_json(
        environ: Dict[str, str],
        ctx: rust.PyContext,
        relations: rust.PyRelations,
        request_uri: str
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Dispatches json requests based on their URIs."""
    content_type = "application/json"
    headers: List[Tuple[str, str]] = []
    prefix = ctx.get_ini().get_uri_prefix()
    if request_uri.startswith(prefix + "/streets/"):
        output = streets_update_result_json(ctx, relations, request_uri)
    elif request_uri.startswith(prefix + "/street-housenumbers/"):
        output = street_housenumbers_update_result_json(ctx, relations, request_uri)
    elif request_uri.startswith(prefix + "/missing-housenumbers/"):
        output = missing_housenumbers_update_result_json(ctx, relations, request_uri)
    else:
        # Assume that request_uri starts with prefix + "/missing-streets/".
        output = missing_streets_update_result_json(ctx, relations, request_uri)
    output_bytes = util.to_bytes(output)
    response = webframe.make_response(content_type, "200 OK", output_bytes, headers)
    return webframe.send_response(environ, response)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
