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
import configparser
import os


def get_abspath(path: str) -> str:
    """Make a path absolute, taking the repo root as a base dir."""
    if os.path.isabs(path):
        return path

    return os.path.join(os.path.dirname(__file__), path)


class Config:
    """Config replacement without static state."""
    def __init__(self, prefix: str) -> None:
        root_dir = os.path.abspath(os.path.dirname(__file__))
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

    def get_tcp_port(self) -> int:
        """Gets the TCP port to be used."""
        return int(self.__config.get("wsgi", "tcp_port", fallback="8000").strip())

    def get_overpass_uri(self) -> str:
        """Gets the URI of the overpass instance to be used."""
        return self.__config.get("wsgi", "overpass_uri", fallback="https://overpass-api.de").strip()

    def get_cron_update_inactive(self) -> bool:
        """Should cron.py update inactive relations?"""
        return self.__config.get("wsgi", "cron_update_inactive", fallback="False").strip() == "True"


# vim:set shiftwidth=4 softtabstop=4 expandtab:
