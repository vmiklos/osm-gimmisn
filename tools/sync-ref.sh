#!/bin/bash -e
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

#
# This script synchronizes reference data between a public instance and a local dev instance.
#

if [ -z "$1" ]; then
    echo "usage:"
    echo "tools/sync-ref.sh https://www.example.com/osm/data/"
    exit 1
fi

refs=$(grep -o '[a-z0-9_]\+.tsv' data/wsgi.ini.template)
mkdir refdir.new
cd refdir.new
for i in $refs
do
    wget $1/$i
done
cd ..
rm -rf refdir
mv refdir.new refdir
cat data/wsgi.ini.template > wsgi.ini

# vim:set shiftwidth=4 softtabstop=4 expandtab:
