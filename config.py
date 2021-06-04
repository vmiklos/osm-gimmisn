#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The config module contains functionality related to configuration handling.
It intentionally doesn't import any other 'own' modules, so it can be used anywhere.
"""

from typing import List
from typing import Optional
import configparser
import os
import subprocess


class Config:
    """Exposes config key values from wsgi.ini."""
    __config: Optional[configparser.ConfigParser] = None

    @staticmethod
    def __get() -> configparser.ConfigParser:
        """Gives direct access to the read config key values."""
        if Config.__config is None:
            Config.__config = configparser.ConfigParser()
            config_path = get_abspath("wsgi.ini")
            Config.__config.read(config_path)

        return Config.__config

    @staticmethod
    def get_value(key: str) -> str:
        """Gets the value of key."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", key)

    @staticmethod
    def set_value(key: str, value: str) -> None:
        """Sets key to value in the in-memory config."""
        Config.__get()
        assert Config.__config is not None
        if value:
            Config.__config.read_dict({"wsgi": {key: value}})
        else:
            Config.__config.remove_option("wsgi", key)

    @staticmethod
    def get_tcp_port() -> int:
        """Gets the TCP port to be used."""
        Config.__get()
        assert Config.__config is not None
        return int(Config.__config.get("wsgi", "tcp_port", fallback="8000").strip())

    @staticmethod
    def get_overpass_uri() -> str:
        """Gets the URI of the overpass instance to be used."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "overpass_uri", fallback="https://overpass-api.de").strip()

    @staticmethod
    def get_cron_update_inactive() -> bool:
        """Should cron.py update inactive relations?"""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "cron_update_inactive", fallback="False").strip() == "True"


def get_abspath(path: str) -> str:
    """Make a path absolute, taking the repo root as a base dir."""
    if os.path.isabs(path):
        return path

    return os.path.join(os.path.dirname(__file__), path)


class Config2:
    """Config replacement without static state."""
    def __init__(self, prefix: str) -> None:
        with subprocess.Popen(['git', 'rev-parse', '--show-toplevel'], stdout=subprocess.PIPE) as process:
            root_dir = process.communicate()[0].rstrip().decode('utf-8')
        self.root = os.path.join(root_dir, prefix)
        self.__config = configparser.ConfigParser()
        config_path = self.get_abspath("wsgi.ini")
        self.__config.read(config_path)

    def has_value(self, key: str) -> bool:
        """Determines if key is set in the config."""
        return self.__config.has_option("wsgi", key)

    def set_value(self, key: str, value: str) -> None:
        """Sets key to value in the in-memory config."""
        self.__config.read_dict({"wsgi": {key: value}})

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

    def get_locale(self) -> str:
        """Gets the locale."""
        return self.__config.get("wsgi", "locale").strip()

    def get_timezone(self) -> str:
        """Gets the timezone."""
        return self.__config.get("wsgi", "timezone").strip()

    def get_uri_prefix(self) -> str:
        """Gets the global URI prefix."""
        return self.__config.get("wsgi", "uri_prefix").strip()


def make_config() -> Config2:
    """Factory for Config2."""
    return Config2("")


# vim:set shiftwidth=4 softtabstop=4 expandtab:
