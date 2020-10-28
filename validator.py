#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The validator module validates yaml files under data/."""

from typing import Any
from typing import Dict
from typing import List
from typing import Tuple
import os
import re
import sys

import yaml

if sys.platform.startswith("win"):
    import _locale


def validate_range_missing_keys(
        errors: List[str],
        parent: str,
        range_data: Dict[str, Any],
        filter_data: Dict[str, Any]
) -> None:
    """Validates a range description: check for missing keys."""
    if "start" not in range_data.keys():
        errors.append("unexpected missing key 'start' for '%s'" % parent)

    if "end" not in range_data.keys():
        errors.append("unexpected missing key 'end' for '%s'" % parent)

    if "start" not in range_data.keys() or "end" not in range_data.keys():
        return

    start = int(range_data["start"])
    end = int(range_data["end"])
    if int(start > end):
        errors.append("expected end >= start for '%s'" % parent)

    if "interpolation" not in filter_data.keys():
        if start % 2 != end % 2:
            errors.append("expected start % 2 == end % 2 for '" + parent + "'")


def validate_range(errors: List[str], parent: str, range_data: Dict[str, Any], filter_data: Dict[str, Any]) -> None:
    """Validates a range description."""
    context = parent + "."
    for key, value in range_data.items():
        if key == "start":
            if not isinstance(value, str):
                errors.append("expected value type for '%s%s' is str" % (context, key))
        elif key == "end":
            if not isinstance(value, str):
                errors.append("expected value type for '%s%s' is str" % (context, key))
        elif key == "refsettlement":
            if not isinstance(value, str):
                errors.append("expected value type for '%s%s' is str" % (context, key))
        else:
            errors.append("unexpected key '%s%s'" % (context, key))
    validate_range_missing_keys(errors, parent, range_data, filter_data)


def validate_ranges(errors: List[str], parent: str, ranges: List[Any], filter_data: Dict[str, Any]) -> None:
    """Validates a range list."""
    context = parent
    for index, range_data in enumerate(ranges):
        validate_range(errors, "%s[%s]" % (context, index), range_data, filter_data)


def validate_filter_invalid(errors: List[str], parent: str, invalid: List[Any]) -> None:
    """Validates an 'invalid' list."""
    context = parent
    for index, invalid_data in enumerate(invalid):
        if not isinstance(invalid_data, str):
            errors.append("expected value type for '%s[%s]' is str" % (context, index))
            continue
        if re.match(r"^[0-9]+$", invalid_data):
            continue
        if re.match(r"^[0-9]+[a-z]$", invalid_data):
            continue
        if re.match(r"^[0-9]+/[0-9]$", invalid_data):
            continue
        errors.append("expected format for '%s[%s]' is '42', '42a' or '42/1'" % (context, index))


def validate_filter(errors: List[str], parent: str, filter_data: Dict[str, Any]) -> None:
    """Validates a filter dictionary."""
    context = parent + "."
    for key, value in filter_data.items():
        if key == "ranges":
            if not isinstance(value, list):
                errors.append("expected value type for '%s%s' is list" % (context, key))
                continue
            validate_ranges(errors, context + "ranges", value, filter_data)
        elif key == "invalid":
            if not isinstance(value, list):
                errors.append("expected value type for '%s%s' is list" % (context, key))
                continue
            validate_filter_invalid(errors, context + "invalid", value)
        elif key == "refsettlement":
            if not isinstance(value, str):
                errors.append("expected value type for '%s%s' is str" % (context, key))
        elif key == "interpolation":
            if not isinstance(value, str):
                errors.append("expected value type for '%s%s' is str" % (context, key))
        elif key == "show-refstreet":
            if not isinstance(value, bool):
                errors.append("expected value type for '%s%s' is bool" % (context, key))
        else:
            errors.append("unexpected key '%s%s'" % (context, key))


def validate_filters(errors: List[str], parent: str, filters: Dict[str, Any]) -> None:
    """Validates a filter list."""
    context = parent + "."
    for key, value in filters.items():
        validate_filter(errors, context + key, value)


def validate_refstreets(errors: List[str], parent: str, refstreets: Dict[str, Any]) -> None:
    """Validates a reference streets list."""
    context = parent + "."
    for key, value in refstreets.items():
        if not isinstance(value, str):
            errors.append("expected value type for '%s%s' is str" % (context, key))
            continue
        if "'" in key or "\"" in key:
            errors.append("expected no quotes in '%s%s'" % (context, key))
        if "'" in value or "\"" in value:
            errors.append("expected no quotes in value of '%s%s'" % (context, key))
    reverse = {v: k for k, v in refstreets.items()}
    if len(refstreets) != len(reverse):
        errors.append("osm and ref streets are not a 1:1 mapping in '%s'" % context)


def validate_street_filters(errors: List[str], parent: str, street_filters: List[Any]) -> None:
    """Validates a street filter list."""
    context = parent
    for index, street_filter in enumerate(street_filters):
        if not isinstance(street_filter, str):
            errors.append("expected value type for '%s[%s]' is str" % (context, index))


def validate_relation_alias(errors: List[str], parent: str, alias: List[Any]) -> None:
    """Validates an 'alias' list."""
    context = parent
    for index, alias_data in enumerate(alias):
        if not isinstance(alias_data, str):
            errors.append("expected value type for '%s[%s]' is str" % (context, index))


def validate_relation(errors: List[str], parent: str, relation: Dict[str, Any]) -> None:
    """Validates a toplevel or a nested relation."""
    context = ""
    if parent:
        context = parent + "."

        # Just to be consistent, we require these keys in relations.yaml for now, even if code would
        # handle having them there on in relation-foo.yaml as well.
        for key in ("osmrelation", "refcounty", "refsettlement"):
            if key not in relation.keys():
                errors.append("missing key '%s%s'" % (context, key))

    handlers: Dict[str, Tuple[Any, Any]] = {
        "osmrelation": (int, None),
        "refcounty": (str, None),
        "refsettlement": (str, None),
        "source": (str, None),
        "filters": (dict, validate_filters),
        "refstreets": (dict, validate_refstreets),
        "missing-streets": (str, None),
        "street-filters": (list, validate_street_filters),
        "osm-street-filters": (list, validate_street_filters),
        "inactive": (bool, None),
        "housenumber-letters": (bool, None),
        "alias": (list, validate_relation_alias),
    }

    for key, value in relation.items():
        if key in handlers.keys():
            value_type, handler = handlers[key]
            if not isinstance(value, value_type):
                errors.append("expected value type for '%s%s' is %s" % (context, key, value_type))
            if handler:
                handler(errors, context + key, value)
        else:
            errors.append("unexpected key '%s%s'" % (context, key))


def validate_relations(errors: List[str], relations: Dict[str, Any]) -> None:
    """Validates a relation list."""
    for key, value in relations.items():
        validate_relation(errors, key, value)


def main() -> None:
    """Commandline interface to this module."""
    if sys.platform.startswith("win"):
        # pylint: disable=protected-access
        _locale._getdefaultlocale = (lambda *args: ['en_US', 'utf8'])

    yaml_path = sys.argv[1]
    _, basename = os.path.split(yaml_path)
    with open(yaml_path) as stream:
        yaml_data = yaml.safe_load(stream)
        errors: List[str] = []
        if basename == "relations.yaml":
            validate_relations(errors, yaml_data)
        else:
            parent = ""
            validate_relation(errors, parent, yaml_data)
        if errors:
            for error in errors:
                print("failed to validate %s: %s" % (yaml_path, error))
            sys.exit(1)


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
