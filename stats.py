#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The stats module creates statistics about missing / non-missing house numbers."""

from typing import List
from typing import Tuple

import rust


def get_topcities(ctx: rust.PyContext, src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    return rust.py_get_topcities(ctx, src_root)


def generate_json(ctx: rust.PyContext, state_dir: str, json_path: str) -> None:
    """Generates the stats json and writes it to `stream`."""
    rust.py_generate_json(ctx, state_dir, json_path)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
