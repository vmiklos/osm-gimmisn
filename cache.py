#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cache module accelerates some functions of the areas module."""

import rust


def is_missing_housenumbers_html_cached(ctx: rust.PyContext, relation: rust.PyRelation) -> bool:
    """Decides if we have an up to date HTML cache entry or not."""
    return rust.py_is_missing_housenumbers_html_cached(ctx, relation)


def get_missing_housenumbers_html(ctx: rust.PyContext, relation: rust.PyRelation) -> rust.PyDoc:
    """Gets the cached HTML of the missing housenumbers for a relation."""
    return rust.py_get_missing_housenumbers_html(ctx, relation)


def get_additional_housenumbers_html(ctx: rust.PyContext, relation: rust.PyRelation) -> rust.PyDoc:
    """Gets the cached HTML of the additional housenumbers for a relation."""
    return rust.py_get_additional_housenumbers_html(ctx, relation)


def is_missing_housenumbers_txt_cached(ctx: rust.PyContext, relation: rust.PyRelation) -> bool:
    """Decides if we have an up to date plain text cache entry or not."""
    return rust.py_is_missing_housenumbers_txt_cached(ctx, relation)


def get_missing_housenumbers_txt(ctx: rust.PyContext, relation: rust.PyRelation) -> str:
    """Gets the cached plain text of the missing housenumbers for a relation."""
    return rust.py_get_missing_housenumbers_txt(ctx, relation)

# vim:set shiftwidth=4 softtabstop=4 expandtab:
