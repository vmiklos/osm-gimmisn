#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Shared type hints.
"""

from typing import Dict
from typing import List


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

# vim:set shiftwidth=4 softtabstop=4 expandtab:
