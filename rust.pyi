#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Type hints for rust.so.
"""

from typing import Any
from typing import List
from typing import Optional
from typing import Tuple
from typing import cast
import api


class PyRange:
    """A range object represents an odd or even range of integer numbers."""
    def __init__(self, start: int, end: int, interpolation: str) -> None:
        ...

    def get_start(self) -> int:
        """The smallest integer."""
        ...

    def get_end(self) -> int:
        """The largest integer."""
        ...

    def is_odd(self) -> Optional[bool]:
        """None for all house numbers on one side, bool otherwise."""
        ...

    def __contains__(self, item: int) -> bool:
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        ...


class PyRanges:
    """A Ranges object contains an item if any of its Range objects contains it."""
    def __init__(self, items: List[PyRange]) -> None:
        ...

    def get_items(self) -> List[PyRange]:
        """The list of contained Range objects."""
        ...

    def __contains__(self, item: int) -> bool:
        ...

    def __repr__(self) -> str:
        ...

    def __eq__(self, other: object) -> bool:
        ...


class PyDoc:
    """Generates xml/html documents."""
    def __init__(self) -> None:
        ...

    def get_value(self) -> str:
        """Gets the escaped value."""
        ...

    def append_value(self, value: str) -> None:
        """Appends escaped content to the value."""
        ...

    def tag(self, name: str, attrs: List[Tuple[str, str]]) -> 'PyTag':
        """Starts a new tag."""
        ...

    def stag(self, name: str, attrs: List[Tuple[str, str]]) -> None:
        """Starts a new tag and closes it as well."""
        ...

    def text(self, text: str) -> None:
        """Appends unescaped content to the document."""
        ...


class PyTag:
    """Starts a tag, which is closed automatically."""
    def __init__(self, doc: PyDoc, name: str, attrs: List[Tuple[str, str]]) -> None:
        ...

    def __enter__(self) -> None:
        ...

    def __exit__(self, tpe: Any, value: Any, traceback: Any) -> None:
        ...

def py_parse(raw_languages: str) -> List[str]:
    """
    Parse a RFC 2616 Accept-Language string.
    https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html#sec14

    :param accept_language_str: A string in RFC 2616 format.
    """
    ...

def py_get_version() -> str:
    """Gets the git version."""
    ...

class PyStdFileSystem:
    """File system implementation, backed by the Rust stdlib."""
    def __init__(self) -> None:
        ...

    def path_exists(self, path: str) -> bool:
        ...

    def getmtime(self, path: str) -> float:
        ...

class PyStdNetwork(api.Network):
    """Network implementation, backed by the Rust stdlib."""
    def urlopen(self, url: str, data: str) -> Tuple[str, str]:  # pragma: no cover
        ...

class PyStdTime(api.Time):
    """Time implementation, backed by the Python stdlib, i.e. intentionally not tested."""
    def now(self) -> float:
        ...

    def sleep(self, seconds: float) -> None:
        ...



# vim:set shiftwidth=4 softtabstop=4 expandtab:
