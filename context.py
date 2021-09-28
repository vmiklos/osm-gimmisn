#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The config module contains functionality related to configuration handling.
It intentionally doesn't import any other 'own' modules, so it can be used anywhere.
"""

import rust


def make_context(prefix: str) -> rust.PyContext:
    """Factory for rust.PyContext."""
    return rust.PyContext(prefix)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
