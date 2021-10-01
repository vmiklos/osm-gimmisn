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


def our_application_json(
        environ: Dict[str, str],
        ctx: rust.PyContext,
        relations: rust.PyRelations,
        request_uri: str
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Dispatches json requests based on their URIs."""
    return rust.py_our_application_json(environ, ctx, relations, request_uri)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
