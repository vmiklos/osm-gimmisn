#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi_additional module contains functionality for additional streets."""

from typing import Tuple

import rust


def additional_streets_view_txt(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str,
    chkl: bool
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt."""
    return rust.py_additional_streets_view_txt(ctx, relations, request_uri, chkl)


def additional_streets_view_result(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str
) -> rust.PyDoc:
    """Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result."""
    return rust.py_additional_streets_view_result(ctx, relations, request_uri)


def additional_housenumbers_view_result(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str
) -> rust.PyDoc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/budapest_11/view-result."""
    return rust.py_additional_housenumbers_view_result(ctx, relations, request_uri)


def additional_streets_view_turbo(relations: rust.PyRelations, request_uri: str) -> rust.PyDoc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo."""
    return rust.py_additional_streets_view_turbo(relations, request_uri)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
