#!/bin/bash -ex
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

#
# This script runs all the tests for CI purposes.
#

if [ -n "${GITHUB_WORKFLOW}" ]; then
    sudo apt-get install tzdata locales
    sudo locale-gen hu_HU.UTF-8

    sudo apt-get install gettext
fi
pip install -r requirements.txt
make -j$(getconf _NPROCESSORS_ONLN) check RSDEBUG=1

# vim:set shiftwidth=4 softtabstop=4 expandtab:
