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
from typing import TextIO
from typing import Tuple
import json
import os
import sys

import yaml

import rust

if sys.platform.startswith("win"):
    import _locale


def validate_filters(errors: List[str], parent: str, filters: Dict[str, Any]) -> List[str]:
    """Validates a filter list."""
    return rust.py_validate_filters(errors, parent, json.dumps(filters))


def validate_refstreets(errors: List[str], parent: str, refstreets: Dict[str, Any]) -> List[str]:
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
    return errors


def validate_street_filters(errors: List[str], parent: str, street_filters: List[Any]) -> List[str]:
    """Validates a street filter list."""
    context = parent
    for index, street_filter in enumerate(street_filters):
        if not isinstance(street_filter, str):
            errors.append("expected value type for '%s[%s]' is str" % (context, index))
    return errors


def validate_relation_alias(errors: List[str], parent: str, alias: List[Any]) -> List[str]:
    """Validates an 'alias' list."""
    context = parent
    for index, alias_data in enumerate(alias):
        if not isinstance(alias_data, str):
            errors.append("expected value type for '%s[%s]' is str" % (context, index))
    return errors


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
        "additional-housenumbers": (bool, None),
        "alias": (list, validate_relation_alias),
    }

    for key, value in relation.items():
        if key in handlers.keys():
            value_type, handler = handlers[key]
            if not isinstance(value, value_type):
                errors.append("expected value type for '%s%s' is %s" % (context, key, value_type))
            if handler:
                errors += handler([], context + key, value)
        else:
            errors.append("unexpected key '%s%s'" % (context, key))


def validate_relations(errors: List[str], relations: Dict[str, Any]) -> None:
    """Validates a relation list."""
    for key, value in relations.items():
        validate_relation(errors, key, value)


def main(argv: List[str], stdout: TextIO) -> int:
    """Commandline interface to this module."""
    if sys.platform.startswith("win"):
        # pylint: disable=protected-access
        _locale._getdefaultlocale = (lambda *args: ['en_US', 'utf8'])

    yaml_path = argv[1]
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
                stdout.write("failed to validate %s: %s\n" % (yaml_path, error))
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv, sys.stdout))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
