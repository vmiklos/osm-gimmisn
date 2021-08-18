#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Type hints for rust.so.
"""

from typing import Any
from typing import BinaryIO
from typing import Dict
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

class PyStdFileSystem(api.FileSystem):
    """File system implementation, backed by the Rust stdlib."""
    def __init__(self) -> None:
        ...

    def path_exists(self, path: str) -> bool:
        ...

    def getmtime(self, path: str) -> float:
        ...

    def open_read(self, path: str) -> BinaryIO:
        ...

    def open_write(self, path: str) -> BinaryIO:
        ...

class PyIni:
    """Configuration file reader."""
    def __init__(self, config_path: str, root: str) -> None:
        ...

    def get_workdir(self) -> str:
        """Gets the directory which is writable."""
        ...

    def get_reference_housenumber_paths(self) -> List[str]:
        """Gets the abs paths of ref housenumbers."""
        ...

    def get_reference_street_path(self) -> str:
        """Gets the abs path of ref streets."""
        ...

    def get_reference_citycounts_path(self) -> str:
        """Gets the abs path of ref citycounts."""
        ...

    def get_uri_prefix(self) -> str:
        """Gets the global URI prefix."""
        ...

    def get_tcp_port(self) -> int:
        """Gets the TCP port to be used."""
        ...

    def get_overpass_uri(self) -> str:
        """Gets the URI of the overpass instance to be used."""
        ...

    def get_cron_update_inactive(self) -> bool:
        """Should cron.py update inactive relations?"""
        ...

class PyContext:
    """Context owns global state which is set up once and then read everywhere."""
    def __init__(self, prefix: str) -> None:
        ...

    def get_abspath(self, rel_path: str) -> str:
        """Make a path absolute, taking the repo root as a base dir."""
        ...

    def get_ini(self) -> PyIni:
        """Gets the ini file."""
        ...

    def set_network(self, network: api.Network) -> None:
        """Sets the network implementation."""
        ...

    def get_network(self) -> api.Network:
        """Gets the network implementation."""
        ...

    def set_time(self, time: api.Time) -> None:
        """Sets the time implementation."""
        ...

    def get_time(self) -> api.Time:
        """Gets the time implementation."""
        ...

    def set_subprocess(self, subprocess: api.Subprocess) -> None:
        """Sets the subprocess implementation."""
        ...

    def get_subprocess(self) -> api.Subprocess:
        """Gets the subprocess implementation."""
        ...

    def set_unit(self, unit: api.Unit) -> None:
        """Sets the unit implementation."""
        ...

    def get_unit(self) -> api.Unit:
        """Gets the unit implementation."""
        ...

    def set_file_system(self, file_system: api.FileSystem) -> None:
        """Sets the file system implementation."""
        ...

    def get_file_system(self) -> api.FileSystem:
        """Gets the file system implementation."""
        ...


# vim:set shiftwidth=4 softtabstop=4 expandtab:
