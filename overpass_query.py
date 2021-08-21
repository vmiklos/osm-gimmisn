#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The overpass_query module allows getting data out of the OSM DB without a full download."""

import rust


overpass_query = rust.py_overpass_query
overpass_query_need_sleep = rust.py_overpass_query_need_sleep


# vim:set shiftwidth=4 softtabstop=4 expandtab:
