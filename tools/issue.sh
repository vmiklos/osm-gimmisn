#!/usr/bin/env bash
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

#
# This script allows offline handling of issues.
#

if [ "$1" == "pull" ]; then
    git config remote.git-bug.url >/dev/null
    if [ $? != 0 ]; then
        git config remote.git-bug.url git://github.com/vmiklos/osm-gimmisn
    fi
    git bug pull git-bug
elif [ "$1" == "open" -a -n "$2" ]; then
    git bug webui -q 'metadata:github-url:"'$2'"'
else
    echo "usage: tools/issue.sh [ pull | open URL ]"
fi

# vim:set shiftwidth=4 softtabstop=4 expandtab:
