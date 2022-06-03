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

    # Build from source: cargo install --version 0.4.4 cargo-llvm-cov
    # Binary install:
    wget https://github.com/ryankurte/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-gnu.tgz
    tar -xvf cargo-binstall-x86_64-unknown-linux-gnu.tgz
    ./cargo-binstall --no-confirm --version 0.4.4 cargo-llvm-cov
    rustup component add llvm-tools-preview --toolchain stable-x86_64-unknown-linux-gnu
fi
make -j$(getconf _NPROCESSORS_ONLN) check RSDEBUG=1

# vim:set shiftwidth=4 softtabstop=4 expandtab:
