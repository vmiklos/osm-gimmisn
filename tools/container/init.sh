#!/usr/bin/env bash
#
# Copyright 2023 Miklos Vajna
#
# SPDX-License-Identifier: MIT
#

cd /opt/osm-gimmisn

# TODO doc: podman exec -t -i osm-gimmisn bash -c 'cd /opt/osm-gimmisn && target/release/osm-gimmisn sync-ref --mode download --url https://www.example.com/osm/data/'
# TODO doc: podman exec -t -i osm-gimmisn bash -c 'cd /opt/osm-gimmisn && git pull -r && make data/yamls.cache'

target/release/osm-gimmisn rouille --host 0.0.0.0

# vim:set shiftwidth=4 softtabstop=4 expandtab:
