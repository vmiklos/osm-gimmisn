#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Parses the Apache access log of osm-gimmisn for 1 month."""

from typing import List
from typing import BinaryIO

import rust


def main(argv: List[str], stdout: BinaryIO, ctx: rust.PyContext) -> None:
    """Commandline interface."""
    rust.py_main(argv, stdout, ctx)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
