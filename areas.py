#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The areas module contains the Relations class and associated functionality."""

import rust


def make_relations(ctx: rust.PyContext) -> rust.PyRelations:
    """Factory for rust.PyRelations."""
    return rust.PyRelations(ctx)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
