#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cache module accelerates some functions of the areas module."""

from typing import List
import os

import yattag

from i18n import translate as tr
import areas
import context
import util


def is_cache_outdated(ctx: context.Context, cache_path: str, dependencies: List[str]) -> bool:
    """Decides if we have an up to date cache entry or not."""
    path_exists = ctx.get_file_system().path_exists
    getmtime = ctx.get_file_system().getmtime
    if not path_exists(cache_path):
        return False

    cache_mtime = getmtime(cache_path)

    for dependency in dependencies:
        if path_exists(dependency) and getmtime(dependency) > cache_mtime:
            return False

    return True


def is_missing_housenumbers_html_cached(ctx: context.Context, relation: areas.Relation) -> bool:
    """Decides if we have an up to date HTML cache entry or not."""
    cache_path = relation.get_files().get_housenumbers_htmlcache_path()
    datadir = ctx.get_abspath("data")
    relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())
    dependencies = [
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path
    ]
    return is_cache_outdated(ctx, cache_path, dependencies)


def is_additional_housenumbers_html_cached(ctx: context.Context, relation: areas.Relation) -> bool:
    """Decides if we have an up to date HTML cache entry for additional house numbers or not."""
    cache_path = relation.get_files().get_additional_housenumbers_htmlcache_path()
    datadir = ctx.get_abspath("data")
    relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())
    dependencies = [
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path
    ]
    return is_cache_outdated(ctx, cache_path, dependencies)


def get_missing_housenumbers_html(ctx: context.Context, relation: areas.Relation) -> yattag.doc.Doc:
    """Gets the cached HTML of the missing housenumbers for a relation."""
    doc = yattag.doc.Doc()
    if is_missing_housenumbers_html_cached(ctx, relation):
        with relation.get_files().get_housenumbers_htmlcache_read_stream(ctx) as stream:
            doc.asis(util.from_bytes(stream.read()))
        return doc

    ret = relation.write_missing_housenumbers()
    todo_street_count, todo_count, done_count, percent, table = ret
 
    with doc.tag("p"):
        prefix = ctx.get_ini().get_uri_prefix()
        prefix2 = "https://osm-gimmisn.vmiklos.hu/osm"

        relation_name = relation.get_name()
        doc.text(tr("OpenStreetMap is possibly missing the below {0} house numbers for {1} streets.")
                 .format(str(todo_count), str(todo_street_count)))
        doc.text(tr(" (existing: {0}, ready: {1}).").format(str(done_count), util.format_percent(str(percent))))
        doc.stag("br")
        with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
            doc.text(tr("Filter incorrect information"))
        doc.text(".")
        doc.stag("br")
        with doc.tag("a", href=prefix + "/missing-housenumbers/{}/view-turbo".format(relation_name)):
            doc.text(tr("Overpass turbo query for the below streets"))
        doc.stag("br")
        with doc.tag("a", href=prefix + "/missing-housenumbers/{}/view-result.txt".format(relation_name)):
            doc.text(tr("Plain text format"))
        doc.stag("br")
        with doc.tag("a", href=prefix + "/missing-housenumbers/{}/view-result.chkl".format(relation_name)):
            doc.text(tr("Checklist format"))
        doc.stag("br")
        with doc.tag("a", href=prefix2 + "/missing-housenumbers/" + relation_name + "/view-lints", target="_blank"):
            doc.text(tr("View lints (vmiklos.hu)"))

    doc.asis(util.html_table_from_list(table).getvalue())
    doc.asis(util.invalid_refstreets_to_html(relation.get_invalid_refstreets()).getvalue())
    doc.asis(util.invalid_filter_keys_to_html(relation.get_invalid_filter_keys()).getvalue())

    with relation.get_files().get_housenumbers_htmlcache_write_stream(ctx) as stream:
        stream.write(util.to_bytes(doc.getvalue()))

    return doc


def get_additional_housenumbers_html(ctx: context.Context, relation: areas.Relation) -> yattag.doc.Doc:
    """Gets the cached HTML of the additional housenumbers for a relation."""
    doc = yattag.doc.Doc()
    if is_additional_housenumbers_html_cached(ctx, relation):
        with relation.get_files().get_additional_housenumbers_htmlcache_read_stream(ctx) as stream:
            doc.asis(util.from_bytes(stream.read()))
        return doc

    ret = relation.write_additional_housenumbers()
    todo_street_count, todo_count, table = ret

    with doc.tag("p"):
        doc.text(tr("OpenStreetMap additionally has the below {0} house numbers for {1} streets.")
                 .format(str(todo_count), str(todo_street_count)))
        doc.stag("br")
        with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
            doc.text(tr("Filter incorrect information"))

    doc.asis(util.html_table_from_list(table).getvalue())
    doc.asis(util.invalid_refstreets_to_html(relation.get_invalid_refstreets()).getvalue())
    doc.asis(util.invalid_filter_keys_to_html(relation.get_invalid_filter_keys()).getvalue())

    with relation.get_files().get_additional_housenumbers_htmlcache_write_stream(ctx) as stream:
        stream.write(util.to_bytes(doc.getvalue()))

    return doc


def is_missing_housenumbers_txt_cached(ctx: context.Context, relation: areas.Relation) -> bool:
    """Decides if we have an up to date plain text cache entry or not."""
    cache_path = relation.get_files().get_housenumbers_txtcache_path()
    datadir = ctx.get_abspath("data")
    relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())
    dependencies = [
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path
    ]
    return is_cache_outdated(ctx, cache_path, dependencies)


def get_missing_housenumbers_txt(ctx: context.Context, relation: areas.Relation) -> str:
    """Gets the cached plain text of the missing housenumbers for a relation."""
    output = ""
    if is_missing_housenumbers_txt_cached(ctx, relation):
        with relation.get_files().get_housenumbers_txtcache_read_stream(ctx) as stream:
            output = util.from_bytes(stream.read())
        return output

    ongoing_streets, _ignore = relation.get_missing_housenumbers()
    table = []
    for result in ongoing_streets:
        range_list = util.get_housenumber_ranges(result[1])
        range_strings = [i.get_number() for i in range_list]
        # Street name, only_in_reference items.
        if not relation.get_config().get_street_is_even_odd(result[0].get_osm_name()):
            result_sorted = sorted(range_strings, key=util.split_house_number)
            row = result[0].get_osm_name() + "\t[" + ", ".join(result_sorted) + "]"
        else:
            elements = util.format_even_odd(range_list, doc=None)
            row = result[0].get_osm_name() + "\t[" + "], [".join(elements) + "]"
        table.append(row)
    table.sort(key=util.get_lexical_sort_key())
    output += "\n".join(table)

    with relation.get_files().get_housenumbers_txtcache_write_stream(ctx) as stream:
        stream.write(util.to_bytes(output))
    return output

# vim:set shiftwidth=4 softtabstop=4 expandtab:
