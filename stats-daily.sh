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

cd "$(dirname "$0")" || exit
date="$(date +%Y-%m-%d)"
./overpass_query.py data/street-housenumbers-hungary.txt > "${date}.csv"
# Ignore 5th field, which is the user who touched the object last.
sed '1d' "${date}.csv" |cut -d $'\t' -f 1-4 |sort -u|wc -l > "${date}.count"
cut -d $'\t' -f 5 "${date}.csv" |sort |uniq -c |sort -k1,1n |tail -n 20 |tac > "${date}.topusers"

# Clean up older (than 7 days), large .csv files.
find . -type f -name "*.csv" -mtime +7 -exec rm -f {} \;

./stats.py > stats.json

prod="workdir/stats-htdocs/"
mkdir -p "${prod}"
cp -- *.html *.js *.json "${prod}/"

# vim:set shiftwidth=4 softtabstop=4 expandtab:
