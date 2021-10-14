#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The stats module creates statistics about missing / non-missing house numbers."""

from typing import Any
from typing import Dict
from typing import List
from typing import Tuple
import json

import rust


def get_topcities(ctx: rust.PyContext, src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    return rust.py_get_topcities(ctx, src_root)


def handle_user_total(ctx: rust.PyContext, src_root: str, j: Dict[str, Any], day_range: int) -> Any:
    """Shows # of total users / day."""
    return json.loads(rust.py_handle_user_total(ctx, src_root, json.dumps(j), day_range))


def get_previous_month(today: int, months: int) -> int:
    """Returns a date that was today N months ago."""
    return rust.py_get_previous_month(today, months)


def handle_monthly_new(ctx: rust.PyContext, src_root: str, j: Dict[str, Any], month_range: int) -> Any:
    """Shows # of new housenumbers / month."""
    return json.loads(rust.py_handle_monthly_new(ctx, src_root, json.dumps(j), month_range))


def handle_daily_total(ctx: rust.PyContext, src_root: str, j: Dict[str, Any], day_range: int) -> Any:
    """Shows # of total housenumbers / day."""
    return json.loads(rust.py_handle_daily_total(ctx, src_root, json.dumps(j), day_range))


def handle_monthly_total(ctx: rust.PyContext, src_root: str, j: Dict[str, Any], month_range: int) -> Any:
    """Shows # of total housenumbers / month."""
    return json.loads(rust.py_handle_monthly_total(ctx, src_root, json.dumps(j), month_range))


def generate_json(ctx: rust.PyContext, state_dir: str, json_path: str) -> None:
    """Generates the stats json and writes it to `stream`."""
    rust.py_generate_json(ctx, state_dir, json_path)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
