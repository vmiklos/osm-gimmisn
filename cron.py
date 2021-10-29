#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cron module allows doing nightly tasks."""

import rust


def update_stats_topusers(ctx: rust.PyContext, today: str) -> None:
    """Counts the top housenumber editors as of today."""
    rust.py_update_stats_topusers(ctx, today)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
