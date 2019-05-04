#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

import re


def sort_numerically(strings):
    return sorted(strings, key=split_house_number)


def split_house_number(house_number):
    match = re.search(r"^([0-9]*)([^0-9].*|)$", house_number)
    number = 0
    try:
        number = int(match.group(1))
    except ValueError:
        pass
    return (number, match.group(2))


def simplify(s, spaceDecode=False):
    """ Handles normalization of a street name."""
    s = s.replace('Á', 'A').replace('á', 'a')
    s = s.replace('É', 'E').replace('é', 'e')
    s = s.replace('Í', 'I').replace('í', 'i')
    s = s.replace('Ó', 'O').replace('ó', 'o')
    s = s.replace('Ö', 'O').replace('ö', 'o')
    s = s.replace('Ő', 'O').replace('ő', 'o')
    s = s.replace('Ú', 'U').replace('ú', 'u')
    s = s.replace('Ü', 'U').replace('ü', 'u')
    s = s.replace('Ű', 'U').replace('ű', 'u')
    if spaceDecode:
        s = s.replace(' ', '%20')
    else:
        s = s.replace(' ', '_')
    s = s.lower()
    return s
