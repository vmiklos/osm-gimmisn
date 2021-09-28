#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Shared type hints.
"""

from typing import BinaryIO
from typing import Dict
from typing import List


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


class Network:
    """Network interface."""
    def urlopen(self, url: str, data: str) -> str:  # pragma: no cover
        """Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class Time:
    """Time interface."""
    def now(self) -> float:  # pragma: no cover
        """Calculates the current Unix timestamp from GMT."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...

    def sleep(self, seconds: float) -> None:  # pragma: no cover
        """Delay execution for a given number of seconds."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class Subprocess:
    """Subprocess interface."""
    def run(self, args: List[str], env: Dict[str, str]) -> str:  # pragma: no cover
        """Runs a commmand, capturing its output."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...

    def exit(self, code: int) -> None:  # pragma: no cover
        """Terminates the current process with the specified exit code."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


class Unit:
    """Unit testing interface."""
    def make_error(self) -> str:  # pragma: no cover
        """Injects a fake error."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


# Two strings: first is a range, second is an optional comment.
HouseNumberWithComment = List[str]

# vim:set shiftwidth=4 softtabstop=4 expandtab:
