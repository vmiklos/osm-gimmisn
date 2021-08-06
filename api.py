#!/usr/bin/env python3
#
# Copyright (c) 2021 Miklos Vajna and contributors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Shared type hints.
"""

from typing import Tuple


class Network:
    """Network interface."""
    def urlopen(self, url: str, data: bytes) -> Tuple[bytes, str]:  # pragma: no cover
        """Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST."""
        # pylint: disable=no-self-use
        # pylint: disable=unused-argument
        ...


# vim:set shiftwidth=4 softtabstop=4 expandtab:
