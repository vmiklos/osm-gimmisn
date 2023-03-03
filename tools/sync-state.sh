#!/usr/bin/env bash
#
# Copyright 2020 Miklos Vajna
#
# SPDX-License-Identifier: MIT
#

#
# This script synchronizes state from a public instance to a local dev one. This helps not running
# all the expensive overpasss queries locally, rather take the existing results via rsync.
#

rsync -avP --delete-after osm-gimmisn@osm-gimmisn:git/osm-gimmisn/{refdir,workdir} .

# vim:set shiftwidth=4 softtabstop=4 expandtab:
