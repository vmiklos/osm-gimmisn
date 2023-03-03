#!/usr/bin/env bash
#
# Copyright 2021 Miklos Vajna
#
# SPDX-License-Identifier: MIT
#

#
# Dumps a dependency graph of rust modules.
#

echo "digraph {" > deps.dot
for module_file in src/*.rs
do
    module=$(basename $module_file .rs)
    for dependency in $(grep "^use crate::" $module_file |sed 's/use crate::\([^:]*\).*;/\1/' |sort -u)
    do
        if [ ! -e "src/$dependency.rs" ]; then
            continue
        fi
        echo "$module -> $dependency;" >> deps.dot
    done
done
echo "}" >> deps.dot
dot -Tpng -o deps.png deps.dot
xdg-open deps.png

# vim:set shiftwidth=4 softtabstop=4 expandtab:
