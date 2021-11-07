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

import rust


def application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: rust.PyContext
) -> Tuple[str, List[Tuple[str, str]], bytes]:
    """The entry point of this WSGI app."""
    return rust.py_application(request_headers, request_data, ctx)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
