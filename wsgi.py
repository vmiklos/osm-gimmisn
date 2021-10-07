#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

from typing import Dict
from typing import List
from typing import Tuple
import traceback

import yattag

import rust
import webframe


def handle_main_housenr_additional_count(ctx: rust.PyContext, relation: rust.PyRelation) -> yattag.Doc:
    """Handles the housenumber additional count part of the main page."""
    return rust.py_handle_main_housenr_additional_count(ctx, relation)


def our_application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: rust.PyContext
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Dispatches the request based on its URI."""
    return rust.py_our_application(request_headers, request_data, ctx)


def application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: rust.PyContext
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """The entry point of this WSGI app."""
    try:
        return our_application(request_headers, request_data, ctx)
    # pylint: disable=broad-except
    except Exception:  # pragma: no cover
        return webframe.handle_exception(request_headers, traceback.format_exc())


# vim:set shiftwidth=4 softtabstop=4 expandtab:
