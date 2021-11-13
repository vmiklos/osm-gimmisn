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

    def set_file_system(self, file_system: api.FileSystem) -> None:
        """Sets the file system implementation."""
        ...

class PyRelationFiles:
    """A relation's file interface provides access to files associated with a relation."""
    def __init__(self, workdir: str, name: str):
        ...

    def get_housenumbers_percent_path(self) -> str:
        """Builds the file name of the house number percent file of a relation."""
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

def py_is_complete_relation(relations: PyRelations, relation_name: str) -> bool:
    """Does this relation have 100% house number coverage?"""
    ...

def py_check_top_edited_relations(ctx: PyContext, frequent_relations: Set[str]) -> Set[str]:
    """
    Update frequent_relations based on get_topcities():
    1) The top 5 edited cities count as frequent, even if they have ~no visitors.
    2) If a relation got <5 house numbers in the last 30 days, then they are not frequent, even with
    lots of visitors.
    """
    ...

def py_main(argv: List[str], stdout: BinaryIO, ctx: PyContext) -> None:
    """Commandline interface."""
    ...

# vim:set shiftwidth=4 softtabstop=4 expandtab:
