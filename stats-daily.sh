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

srcdir="$(dirname "$0")"
statedir="$(dirname "$0")/workdir/stats/"
mkdir -p "${statedir}"

date="$(date +%Y-%m-%d)"
# Ignore 5th field, which is the user who touched the object last.
sed '1d' "${statedir}/${date}.csv" |cut -d $'\t' -f 1-4 |sort -u|wc -l > "${statedir}/${date}.count"
cut -d $'\t' -f 5 "${statedir}/${date}.csv" |sort |uniq -c |sort -k1,1rn |head -n 20 > "${statedir}/${date}.topusers"

# Clean up older (than 7 days), large .csv files.
find "${statedir}" -type f -name "*.csv" -mtime +7 -exec rm -f {} \;

# vim:set shiftwidth=4 softtabstop=4 expandtab:
