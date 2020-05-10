#!/usr/bin/env python3
#
# Copyright (c) 2020 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
The config module contains functionality related to configuration handling.
It intentionally doesn't import any other 'own' modules, so it can be used anywhere.
"""

from typing import Any
from typing import List
from typing import Optional
import configparser
import os


class Config:
    """Exposes config key values from wsgi.ini."""
    __config: Optional[configparser.ConfigParser] = None

    @staticmethod
    def __get() -> configparser.ConfigParser:
        """Gives direct access to the read config key values."""
        if Config.__config is None:
            Config.__config = configparser.ConfigParser()
            for relpath in ("wsgi.ini", "wsgi.ini.local"):
                config_path = get_abspath(relpath)
                if os.path.exists(config_path):
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
    def has_value(key: str) -> bool:
        """Determines if key is set in the config."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.has_option("wsgi", key)

    @staticmethod
    def get_workdir() -> str:
        """Gets the directory which is writable."""
        Config.__get()
        assert Config.__config is not None
        return get_abspath(Config.__config.get('wsgi', 'workdir').strip())

    @staticmethod
    def get_reference_housenumber_paths() -> List[str]:
        """Gets the abs paths of ref housenumbers."""
        Config.__get()
        assert Config.__config is not None
        relpaths = Config.__config.get("wsgi", "reference_housenumbers").strip().split(' ')
        return [get_abspath(relpath) for relpath in relpaths]

    @staticmethod
    def get_reference_street_path() -> str:
        """Gets the abs path of ref streets."""
        Config.__get()
        assert Config.__config is not None
        relpath = Config.__config.get("wsgi", "reference_street").strip()
        return get_abspath(relpath)

    @staticmethod
    def get_locale() -> str:
        """Gets the locale."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "locale").strip()

    @staticmethod
    def get_timezone() -> str:
        """Gets the timezone."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "timezone").strip()

    @staticmethod
    def get_uri_prefix() -> str:
        """Gets the global URI prefix."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "uri_prefix").strip()

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

    @staticmethod
    def has_matomo() -> bool:
        """Checks if both matomo_url and matomo_site_id are set."""
        Config.__get()
        assert Config.__config is not None
        return Config.has_value("matomo_url") and Config.has_value("matomo_site_id")

    @staticmethod
    def get_matomo_url() -> str:
        """Gets the Matomo URL."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "matomo_url").strip()

    @staticmethod
    def get_matomo_site_id() -> str:
        """Gets the Matomo site ID."""
        Config.__get()
        assert Config.__config is not None
        return Config.__config.get("wsgi", "matomo_site_id").strip()


class ConfigContext:
    """Context manager for Config."""
    def __init__(self, key: str, value: str) -> None:
        """Remembers what should be the new value."""
        self.key = key
        self.value = value
        self.old_value = ""
        if Config.has_value(key):
            self.old_value = Config.get_value(key)

    def __enter__(self) -> 'ConfigContext':
        """Switches to the new value."""
        Config.set_value(self.key, self.value)
        return self

    def __exit__(self, _exc_type: Any, _exc_value: Any, _exc_traceback: Any) -> bool:
        """Switches back to the old value."""
        Config.set_value(self.key, self.old_value)
        return True


def get_abspath(path: str) -> str:
    """Make a path absolute, taking the repo root as a base dir."""
    if os.path.isabs(path):
        return path

    return os.path.join(os.path.dirname(__file__), path)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
