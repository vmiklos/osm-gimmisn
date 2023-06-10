#!/usr/bin/env bash
#
# Copyright 2023 Miklos Vajna
#
# SPDX-License-Identifier: MIT
#

mkdir -p workdir
podman run \
    --hostname osm-gimmisn \
    --interactive \
    --name osm-gimmisn \
    --publish 8000:8000 \
    --rm \
    --tty \
    --volume $PWD/workdir:/opt/osm-gimmisn/workdir \
    osm-gimmisn:latest

# vim:set shiftwidth=4 softtabstop=4 expandtab:
