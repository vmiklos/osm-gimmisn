#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The config module contains functionality related to configuration handling.
It intentionally doesn't import any other 'own' modules, so it can be used anywhere.
"""

import os

import api
import rust


Ini = rust.PyIni
StdFileSystem = rust.PyStdFileSystem


class Context:
    """Context owns global state which is set up once and then read everywhere."""
    def __init__(self, prefix: str) -> None:
        self.__rust = rust.PyContext(prefix)
        root_dir = os.path.abspath(os.path.dirname(__file__))
        self.root = os.path.join(root_dir, prefix)
        self.__file_system: api.FileSystem = StdFileSystem()

    def get_abspath(self, rel_path: str) -> str:
        """Make a path absolute, taking the repo root as a base dir."""
        return self.__rust.get_abspath(rel_path)

    def set_file_system(self, file_system: api.FileSystem) -> None:
        """Sets the file system implementation."""
        self.__file_system = file_system

    def get_file_system(self) -> api.FileSystem:
        """Gets the file system implementation."""
        return self.__file_system

    def set_network(self, network: api.Network) -> None:
        """Sets the network implementation."""
        return self.__rust.set_network(network)

    def get_network(self) -> api.Network:
        """Gets the network implementation."""
        return self.__rust.get_network()

    def set_time(self, time: api.Time) -> None:
        """Sets the time implementation."""
        self.__rust.set_time(time)

    def get_time(self) -> api.Time:
        """Gets the time implementation."""
        return self.__rust.get_time()

    def set_subprocess(self, subprocess: api.Subprocess) -> None:
        """Sets the subprocess implementation."""
        self.__rust.set_subprocess(subprocess)

    def get_subprocess(self) -> api.Subprocess:
        """Gets the subprocess implementation."""
        return self.__rust.get_subprocess()

    def get_ini(self) -> Ini:
        """Gets the ini file."""
        return self.__rust.get_ini()

    def set_unit(self, unit: api.Unit) -> None:
        """Sets the testing interface."""
        self.__rust.set_unit(unit)

    def get_unit(self) -> api.Unit:
        """Gets the testing interface."""
        return self.__rust.get_unit()


# vim:set shiftwidth=4 softtabstop=4 expandtab:
