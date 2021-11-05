#!/usr/bin/env bash
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

#
# Dumps a dependency graph of python modules.
#

echo "digraph {" > deps.dot
for module_file in *.py
do
    module=$(basename $module_file .py)
    for dependency in $(grep ^import $module_file|sed 's/import //'; grep ^from $module_file |sed 's/from \(.*\) import.*/\1/g'|sort -u)
    do
        # Silence stubs for rust modules.
        case $dependency in
            api)
                continue
            ;;
            cache)
                continue
            ;;
            context)
                continue
            ;;
            util)
                continue
            ;;
            areas)
                continue
            ;;
            wsgi)
                continue
            ;;
        esac

        if [ ! -e "$dependency.py" ]; then
            continue
        fi
        echo "$module -> $dependency;" >> deps.dot
    done
done
echo "}" >> deps.dot
dot -Tpng -o deps.png deps.dot
xdg-open deps.png

# vim:set shiftwidth=4 softtabstop=4 expandtab:
