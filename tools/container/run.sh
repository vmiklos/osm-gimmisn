#!/usr/bin/env bash
#
# Copyright 2023 Miklos Vajna
#
# SPDX-License-Identifier: MIT
#

REF_URL="$1"

mkdir -p workdir
podman run \
    --env "REF_URL=$REF_URL" \
    --hostname osm-gimmisn \
    --interactive \
    --name osm-gimmisn \
    --publish 8000:8000 \
    --rm \
    --tty \
    --volume $PWD/workdir:/opt/osm-gimmisn/workdir \
    localhost/osm-gimmisn:latest

# vim:set shiftwidth=4 softtabstop=4 expandtab:
