#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains free functions shared between other modules."""

from typing import List

import helpers


def format_even_odd(only_in_ref: List[str], html: bool) -> List[str]:
    """Separate even and odd numbers, this helps survey in most cases."""
    key = helpers.split_house_number
    even = sorted([i for i in only_in_ref if int(helpers.split_house_number(i)[0]) % 2 == 0], key=key)
    if html:
        even = [color_house_number(i) for i in even]
    even_string = ", ".join(even)
    odd = sorted([i for i in only_in_ref if int(helpers.split_house_number(i)[0]) % 2 == 1], key=key)
    if html:
        odd = [color_house_number(i) for i in odd]
    odd_string = ", ".join(odd)
    elements = []
    if odd_string:
        elements.append(odd_string)
    if even_string:
        elements.append(even_string)
    return elements


def color_house_number(fro: str) -> str:
    """Colors a house number according to its suffix."""
    if not fro.endswith("*"):
        return fro
    return '<span style="color: blue;">' + fro[:-1] + '</span>'

# vim:set shiftwidth=4 softtabstop=4 expandtab:
