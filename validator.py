#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The validator module validates yaml files under data/."""

import os
import sys
from typing import Any
from typing import Dict
from typing import List
from typing import Tuple
import yaml


def validate_range_missing_keys(parent: str, range_data: Dict[str, Any], filter_data: Dict[str, Any]) -> str:
    """Validates a range description: check for missing keys."""
    if "start" not in range_data.keys():
        return "unexpected missing key 'start' for '%s'" % parent

    if "end" not in range_data.keys():
        return "unexpected missing key 'end' for '%s'" % parent

    start = int(range_data["start"])
    end = int(range_data["end"])
    if int(start > end):
        return "expected end >= start for '%s'" % parent

    if "interpolation" not in filter_data.keys():
        if start % 2 != end % 2:
            return "expected start % 2 == end % 2 for '" + parent + "'"

    return ""


def validate_range(parent: str, range_data: Dict[str, Any], filter_data: Dict[str, Any]) -> str:
    """Validates a range description."""
    context = parent + "."
    for key, value in range_data.items():
        if key == "start":
            if not isinstance(value, str):
                return "expected value type for '%s%s' is str" % (context, key)
        elif key == "end":
            if not isinstance(value, str):
                return "expected value type for '%s%s' is str" % (context, key)
        elif key == "reftelepules":
            if not isinstance(value, str):
                return "expected value type for '%s%s' is str" % (context, key)
        else:
            return "unexpected key '%s%s'" % (context, key)
    return validate_range_missing_keys(parent, range_data, filter_data)


def validate_ranges(parent: str, ranges: List[Any], filter_data: Dict[str, Any]) -> str:
    """Validates a range list."""
    context = parent
    for index, range_data in enumerate(ranges):
        ret = validate_range("%s[%s]" % (context, index), range_data, filter_data)
        if ret:
            return ret
    return ""


def validate_filter(parent: str, filter_data: Dict[str, Any]) -> str:
    """Validates a filter dictionary."""
    context = parent + "."
    for key, value in filter_data.items():
        if key == "ranges":
            if not isinstance(value, list):
                return "expected value type for '%s%s' is list" % (context, key)
            ret = validate_ranges(context + "ranges", value, filter_data)
            if ret:
                return ret
        elif key == "reftelepules":
            if not isinstance(value, str):
                return "expected value type for '%s%s' is str" % (context, key)
        elif key == "interpolation":
            if not isinstance(value, str):
                return "expected value type for '%s%s' is str" % (context, key)
        else:
            return "unexpected key '%s%s'" % (context, key)
    return ""


def validate_filters(parent: str, filters: Dict[str, Any]) -> str:
    """Validates a filter list."""
    context = parent + "."
    for key, value in filters.items():
        ret = validate_filter(context + key, value)
        if ret:
            return ret
    return ""


def validate_refstreets(parent: str, refstreets: Dict[str, Any]) -> str:
    """Validates a reference streets list."""
    context = parent + "."
    for key, value in refstreets.items():
        if not isinstance(value, str):
            return "expected value type for '%s%s' is str" % (context, key)
    return ""


def validate_street_filters(parent: str, street_filters: List[Any]) -> str:
    """Validates a street filter list."""
    context = parent
    for index, street_filter in enumerate(street_filters):
        if not isinstance(street_filter, str):
            return "expected value type for '%s[%s]' is str" % (context, index)
    return ""


def validate_relation(parent: str, relation: Dict[str, Any]) -> str:
    """Validates a toplevel or a nested relation."""
    context = ""
    if parent:
        context = parent + "."

        # Just to be consistent, we require these keys in relations.yaml for now, even if code would
        # handle having them there on in relation-foo.yaml as well.
        for key in ("osmrelation", "refmegye", "reftelepules"):
            if key not in relation.keys():
                return "missing key '%s%s'" % (context, key)

    ret = ""

    handlers = {
        "osmrelation": (int, None),
        "refmegye": (str, None),
        "reftelepules": (str, None),
        "source": (str, None),
        "filters": (dict, validate_filters),
        "refstreets": (dict, validate_refstreets),
        "suspicious-relations": (str, None),
        "street-filters": (list, validate_street_filters),
    }  # type: Dict[str, Tuple[Any, Any]]

    for key, value in relation.items():
        if key in handlers.keys():
            value_type, handler = handlers[key]
            if not isinstance(value, value_type):
                ret = "expected value type for '%s%s' is %s" % (context, key, value_type)
                break
            if handler:
                ret = handler(context + key, value)
                if ret:
                    break
        else:
            ret = "unexpected key '%s%s'" % (context, key)
            break
    return ret


def validate_relations(relations: Dict[str, Any]) -> str:
    """Validates a relation list."""
    for key, value in relations.items():
        ret = validate_relation(key, value)
        if ret:
            return ret
    return ""


def main() -> None:
    """Commandline interface to this module."""

    yaml_path = sys.argv[1]
    _, basename = os.path.split(yaml_path)
    with open(yaml_path) as stream:
        yaml_data = yaml.load(stream)
        ret = ""
        if basename == "relations.yaml":
            ret = validate_relations(yaml_data)
        else:
            parent = ""
            ret = validate_relation(parent, yaml_data)
        if ret:
            print("failed to validate %s: %s" % (yaml_path, ret))
            sys.exit(1)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
