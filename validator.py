#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The validator module validates yaml files under data/."""

from typing import BinaryIO
from typing import List

import rust


def main(argv: List[str], stdout: BinaryIO) -> int:
    """Commandline interface to this module."""
    return rust.py_validator_main(argv, stdout)


# vim:set shiftwidth=4 softtabstop=4 expandtab:
