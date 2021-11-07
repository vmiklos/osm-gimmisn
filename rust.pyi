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
from typing import Set
from typing import Tuple
from typing import TypeVar
from typing import cast
import api


class PyDoc:
    """Generates xml/html documents."""
    def __init__(self) -> None:
        ...

    def get_value(self) -> str:
        """Gets the escaped value."""
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

    def get_uri_prefix(self) -> str:
        """Gets the global URI prefix."""
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

def py_get_content(path: str) -> bytes:
    """Gets the content of a file in workdir."""
    ...

class PyRelationFiles:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, workdir: str, name: str):
        ...

    def get_ref_streets_path(self) -> str:
        """Build the file name of the reference street list of a relation."""
        ...

    def get_osm_streets_path(self) -> str:
        """Build the file name of the OSM street list of a relation."""
        ...

    def get_osm_housenumbers_path(self) -> str:
        """Build the file name of the OSM house number list of a relation."""
        ...

    def get_ref_housenumbers_path(self) -> str:
        """Build the file name of the reference house number list of a relation."""
        ...

    def get_housenumbers_percent_path(self) -> str:
        """Builds the file name of the house number percent file of a relation."""
        ...

    def get_housenumbers_htmlcache_path(self) -> str:
        """Builds the file name of the house number HTML cache file of a relation."""
        ...

    def get_streets_percent_path(self) -> str:
        """Builds the file name of the street percent file of a relation."""
        ...

    def get_streets_additional_count_path(self) -> str:
        """Builds the file name of the street additional count file of a relation."""
        ...

    def get_housenumbers_additional_count_path(self) -> str:
        """Builds the file name of the housenumber additional count file of a relation."""
        ...

    def get_ref_streets_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the reference street list of a relation for reading."""
        ...

    def get_osm_streets_read_stream(self, ctx: PyContext) -> BinaryIO:
        """Opens the OSM street list of a relation for reading."""
        ...

class PyRelationConfig:
    """A relation configuration comes directly from static data, not a result of some external query."""
    def is_active(self) -> bool:
        """Gets if the relation is active."""
        ...

class PyRelation:
    """A relation is a closed polygon on the map."""
    def get_name(self) -> str:
        """Gets the name of the relation."""
        ...

    def get_files(self) -> PyRelationFiles:
        """Gets access to the file interface."""
        ...

    def get_config(self) -> PyRelationConfig:
        """Gets access to the config interface."""
        ...

    def set_config(self, config: PyRelationConfig) -> None:
        """Sets the config interface."""
        ...

class PyRelations:
    """A relations object is a container of named relation objects."""
    def __init__(self, ctx: PyContext) -> None:
        ...

    def get_relation(self, name: str) -> PyRelation:
        """Gets the relation that has the specified name."""
        ...

    def get_names(self) -> List[str]:
        """Gets a sorted list of relation names."""
        ...

    def get_relations(self) -> List[PyRelation]:
        """Gets a list of relations."""
        ...

def py_application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: PyContext
) -> Tuple[str, List[Tuple[str, str]], bytes]:
    """The entry point of this WSGI app."""
    ...

def py_get_topcities(ctx: PyContext, src_root: str) -> List[Tuple[str, int]]:
    """
    Generates a list of cities, sorted by how many new hours numbers they got recently.
    """
    ...

# vim:set shiftwidth=4 softtabstop=4 expandtab:
