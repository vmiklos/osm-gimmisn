#!/usr/bin/env bash
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

#
# This script runs an overpass query, then generates stats.json based on up to date and historic
# data. It doesn't necessarily run on the same machine as cron.py.
#

statedir="$(dirname "$0")/workdir/stats/"

date="$(date +%Y-%m-%d)"
cut -d $'\t' -f 5 "${statedir}/${date}.csv" |sort |uniq -c |wc -l > "${statedir}/${date}.usercount"

# vim:set shiftwidth=4 softtabstop=4 expandtab:
