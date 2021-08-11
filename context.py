#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The config module contains functionality related to configuration handling.
It intentionally doesn't import any other 'own' modules, so it can be used anywhere.
"""

from typing import BinaryIO
import os

import api
import rust


class FileSystem:
    """File system interface."""
    def path_exists(self, path: str) -> bool:  # pragma: no cover
        """Test whether a path exists."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...

    def getmtime(self, path: str) -> float:  # pragma: no cover
        """Return the last modification time of a file."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...

    def open_read(self, path: str) -> BinaryIO:  # pragma: no cover
        """Opens a file for reading in binary mode."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...

    def open_write(self, path: str) -> BinaryIO:  # pragma: no cover
        """Opens a file for writing in binary mode."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class StdFileSystem(FileSystem):
    """File system implementation, backed by the Python stdlib."""
    def __init__(self) -> None:
        self.rust = rust.PyStdFileSystem()

    def path_exists(self, path: str) -> bool:
        return self.rust.path_exists(path)

    def getmtime(self, path: str) -> float:
        return self.rust.getmtime(path)

    def open_read(self, path: str) -> BinaryIO:
        # The caller will do this:
        # pylint: disable=consider-using-with
        return open(path, "rb")

    def open_write(self, path: str) -> BinaryIO:
        # The caller will do this:
        # pylint: disable=consider-using-with
        return open(path, "wb")


StdNetwork = rust.PyStdNetwork
StdTime = rust.PyStdTime
StdSubprocess = rust.PyStdSubprocess
StdUnit = rust.PyStdUnit
Ini = rust.PyIni


class Context:
    """Context owns global state which is set up once and then read everywhere."""
    def __init__(self, prefix: str) -> None:
        self.__rust = rust.PyContext(prefix)
        root_dir = os.path.abspath(os.path.dirname(__file__))
        self.root = os.path.join(root_dir, prefix)
        self.__file_system: FileSystem = StdFileSystem()
        self.__network: api.Network = StdNetwork()
        self.__time: api.Time = StdTime()
        self.__subprocess: api.Subprocess = StdSubprocess()
        self.__unit: api.Unit = StdUnit()

    def get_abspath(self, rel_path: str) -> str:
        """Make a path absolute, taking the repo root as a base dir."""
        return self.__rust.get_abspath(rel_path)

    def set_file_system(self, file_system: FileSystem) -> None:
        """Sets the file system implementation."""
        self.__file_system = file_system

    def get_file_system(self) -> FileSystem:
        """Gets the file system implementation."""
        return self.__file_system

    def set_network(self, network: api.Network) -> None:
        """Sets the network implementation."""
        self.__network = network

    def get_network(self) -> api.Network:
        """Gets the network implementation."""
        return self.__network

    def set_time(self, time_impl: api.Time) -> None:
        """Sets the time implementation."""
        self.__time = time_impl

    def get_time(self) -> api.Time:
        """Gets the time implementation."""
        return self.__time

    def set_subprocess(self, subprocess: api.Subprocess) -> None:
        """Sets the subprocess implementation."""
        self.__subprocess = subprocess

    def get_subprocess(self) -> api.Subprocess:
        """Gets the subprocess implementation."""
        return self.__subprocess

    def get_ini(self) -> Ini:
        """Gets the ini file."""
        return self.__rust.get_ini()

    def set_unit(self, unit: api.Unit) -> None:
        """Sets the testing interface."""
        self.__unit = unit

    def get_unit(self) -> api.Unit:
        """Gets the testing interface."""
        return self.__unit


# vim:set shiftwidth=4 softtabstop=4 expandtab:
