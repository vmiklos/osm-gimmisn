#!/usr/bin/env python3
#
# Copyright (c) 2019 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""The util module contains functionality shared between other modules."""

import rust


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
