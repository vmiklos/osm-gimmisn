#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

from typing import Dict
from typing import List
from typing import Tuple

import api
import rust


def split_house_number(house_number: str) -> Tuple[int, str]:
    """Splits house_number into a numerical and a remainder part."""
    return rust.py_split_house_number(house_number)


def build_street_reference_cache(local_streets: str) -> Dict[str, Dict[str, List[str]]]:
    """Builds an in-memory cache from the reference on-disk TSV (street version)."""
    return rust.py_build_street_reference_cache(local_streets)


def build_reference_cache(local: str, refcounty: str) -> Dict[str, Dict[str, Dict[str, List[api.HouseNumberWithComment]]]]:
    """Builds an in-memory cache from the reference on-disk TSV (house number version)."""
    return rust.py_build_reference_cache(local, refcounty)


def get_housenumber_ranges(house_numbers: List[rust.PyHouseNumber]) -> List[rust.PyHouseNumberRange]:
    """Gets a reference range list for a house number list by looking at what range provided a givne
    house number."""
    return rust.py_get_housenumber_ranges(house_numbers)


def get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
    return rust.py_get_content(path)


def to_bytes(string: str) -> bytes:
    """Encodes the string to UTF-8."""
    return string.encode("utf-8")


def from_bytes(array: bytes) -> str:
    """Decodes the string from UTF-8."""
    return array.decode("utf-8")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
