#!/usr/bin/env bash
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

# Allows running the tests in a read-only source root (at least the tests/ subfolder).

if [ "$1" == "on" ]; then
    mv tests tests.orig
    mkdir tests
    sudo mount --bind -o ro tests.orig tests
elif [ "$1" == "off" ]; then
    sudo umount tests
    rmdir tests
    mv tests.orig tests
fi

# vim:set shiftwidth=4 softtabstop=4 expandtab:
