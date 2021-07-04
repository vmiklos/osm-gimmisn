#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The config module contains functionality related to configuration handling.
It intentionally doesn't import any other 'own' modules, so it can be used anywhere.
"""

from typing import Dict
from typing import List
from typing import Optional
from typing import Tuple
from typing import cast
import calendar
import configparser
import os
import time
import urllib.request
import subprocess


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

    def get_relation_allowlist(self) -> List[str]:  # pragma: no cover
        """Returns the list of relations which are not in relations.yaml, but are to be accepted."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class StdFileSystem(FileSystem):
    """File system implementation, backed by the Python stdlib."""
    def path_exists(self, path: str) -> bool:
        return os.path.exists(path)

    def getmtime(self, path: str) -> float:
        return os.path.getmtime(path)

    def get_relation_allowlist(self) -> List[str]:
        return []


class Network:
    """Network interface."""
    def urlopen(self, url: str, data: Optional[bytes] = None) -> Tuple[bytes, str]:  # pragma: no cover
        """Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class StdNetwork(Network):
    """Network implementation, backed by the Python stdlib."""
    def urlopen(self, url: str, data: Optional[bytes] = None) -> Tuple[bytes, str]:  # pragma: no cover
        try:
            with urllib.request.urlopen(url, data) as stream:
                buf = stream.read()
            return (cast(bytes, buf), str())
        except urllib.error.HTTPError as http_error:
            return (bytes(), str(http_error))


class Time:
    """Time interface."""
    def now(self) -> float:  # pragma: no cover
        """Calculates the current Unix timestamp from GMT."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class StdTime(Time):
    """Time implementation, backed by the Python stdlib, i.e. intentionally not tested."""
    def now(self) -> float:  # pragma: no cover
        # time.time() would use the current TZ, not GMT.
        return calendar.timegm(time.localtime())


class Subprocess:
    """Subprocess interface."""
    def run(self, args: List[str], env: Dict[str, str]) -> bytes:  # pragma: no cover
        """Runs a commmand, capturing its output."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class StdSubprocess(Subprocess):
    """Subprocess implementation, backed by the Python stdlib, i.e. intentionally not tested."""
    def run(self, args: List[str], env: Dict[str, str]) -> bytes:  # pragma: no cover
        full_env = os.environ
        full_env.update(env)
        process = subprocess.run(args, stdout=subprocess.PIPE, check=True, env=full_env)
        return process.stdout


class Config:
    """Config replacement without static state."""
    def __init__(self, prefix: str) -> None:
        root_dir = os.path.abspath(os.path.dirname(__file__))
        self.root = os.path.join(root_dir, prefix)
        self.__config = configparser.ConfigParser()
        config_path = self.get_abspath("wsgi.ini")
        self.__config.read(config_path)
        self.__file_system: FileSystem = StdFileSystem()
        self.__network: Network = StdNetwork()
        self.__time: Time = StdTime()
        self.__subprocess: Subprocess = StdSubprocess()

    def get_abspath(self, rel_path: str) -> str:
        """Make a path absolute, taking the repo root as a base dir."""
        return os.path.join(self.root, rel_path)

    def get_workdir(self) -> str:
        """Gets the directory which is writable."""
        return self.get_abspath(self.__config.get('wsgi', 'workdir').strip())

    def get_reference_housenumber_paths(self) -> List[str]:
        """Gets the abs paths of ref housenumbers."""
        relpaths = self.__config.get("wsgi", "reference_housenumbers").strip().split(' ')
        return [self.get_abspath(relpath) for relpath in relpaths]

    def get_reference_street_path(self) -> str:
        """Gets the abs path of ref streets."""
        relpath = self.__config.get("wsgi", "reference_street").strip()
        return self.get_abspath(relpath)

    def get_reference_citycounts_path(self) -> str:
        """Gets the abs path of ref citycounts."""
        relpath = self.__config.get("wsgi", "reference_citycounts").strip()
        return self.get_abspath(relpath)

    def get_uri_prefix(self) -> str:
        """Gets the global URI prefix."""
        return self.__config.get("wsgi", "uri_prefix").strip()

    def get_tcp_port(self) -> int:
        """Gets the TCP port to be used."""
        return int(self.__config.get("wsgi", "tcp_port", fallback="8000").strip())

    def get_overpass_uri(self) -> str:
        """Gets the URI of the overpass instance to be used."""
        return self.__config.get("wsgi", "overpass_uri", fallback="https://overpass-api.de").strip()

    def get_cron_update_inactive(self) -> bool:
        """Should cron.py update inactive relations?"""
        return self.__config.get("wsgi", "cron_update_inactive", fallback="False").strip() == "True"

    def set_file_system(self, file_system: FileSystem) -> None:
        """Sets the file system implementation."""
        self.__file_system = file_system

    def get_file_system(self) -> FileSystem:
        """Gets the file system implementation."""
        return self.__file_system

    def set_network(self, network: Network) -> None:
        """Sets the network implementation."""
        self.__network = network

    def get_network(self) -> Network:
        """Gets the network implementation."""
        return self.__network

    def set_time(self, time_impl: Time) -> None:
        """Sets the time implementation."""
        self.__time = time_impl

    def get_time(self) -> Time:
        """Gets the time implementation."""
        return self.__time

    def set_subprocess(self, subprocess_impl: Subprocess) -> None:
        """Sets the subprocess implementation."""
        self.__subprocess = subprocess_impl

    def get_subprocess(self) -> Subprocess:
        """Gets the subprocess implementation."""
        return self.__subprocess


# vim:set shiftwidth=4 softtabstop=4 expandtab:
