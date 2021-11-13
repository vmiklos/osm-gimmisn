#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""Parses the Apache access log of osm-gimmisn for 1 month."""

from typing import List
from typing import Set
from typing import BinaryIO
import sys

import context
import rust


def is_complete_relation(relations: rust.PyRelations, relation_name: str) -> bool:
    """Does this relation have 100% house number coverage?"""
    return rust.py_is_complete_relation(relations, relation_name)


def check_top_edited_relations(ctx: rust.PyContext, frequent_relations: Set[str]) -> Set[str]:
    """
    Update frequent_relations based on get_topcities():
    1) The top 5 edited cities count as frequent, even if they have ~no visitors.
    2) If a relation got <5 house numbers in the last 30 days, then they are not frequent, even with
    lots of visitors.
    """
    return rust.py_check_top_edited_relations(ctx, frequent_relations)


def main(argv: List[str], stdout: BinaryIO, ctx: rust.PyContext) -> None:
    """Commandline interface."""
    rust.py_main(argv, stdout, ctx)


if __name__ == '__main__':
    main(sys.argv, sys.stdout.buffer, context.make_context(""))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
