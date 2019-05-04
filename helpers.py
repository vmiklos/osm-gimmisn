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
