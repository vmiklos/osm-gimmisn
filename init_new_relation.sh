#!/bin/bash -ex
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Example: ./init_new_relation.sh madarhegy 2713839

name="$1"
value="$2"

cd workdir

sed "s/2714372/$value/g" street-housenumbers-sasad.txt > street-housenumbers-$name.txt
../overpass_query.py street-housenumbers-$name.txt > street-housenumbers-$name.csv

# Don't talk to overpass again without an interrupt.
sleep 10s

sed "s/2714372/$value/g" streets-sasad.txt > streets-$name.txt
../overpass_query.py streets-$name.txt > streets-$name.csv

# vim:set shiftwidth=4 softtabstop=4 expandtab:
