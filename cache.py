#!/usr/bin/env python3
#
# Copyright 2021 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cache module accelerates some functions of the areas module."""

import os

import yattag

from i18n import translate as _
import areas
import config
import util


def is_missing_housenumbers_html_cached(relation: areas.Relation) -> bool:
    """Decides if we have an up to date cache entry or not."""
    cache_path = relation.get_files().get_housenumbers_htmlcache_path()
    if not os.path.exists(cache_path):
        return False

    cache_mtime = os.path.getmtime(cache_path)
    osm_streets_path = relation.get_files().get_osm_streets_path()
    osm_streets_mtime = os.path.getmtime(osm_streets_path)
    if osm_streets_mtime > cache_mtime:
        return False

    osm_housenumbers_path = relation.get_files().get_osm_housenumbers_path()
    osm_housenumbers_mtime = os.path.getmtime(osm_housenumbers_path)
    if osm_housenumbers_mtime > cache_mtime:
        return False

    ref_housenumbers_path = relation.get_files().get_ref_housenumbers_path()
    ref_housenumbers_mtime = os.path.getmtime(ref_housenumbers_path)
    if ref_housenumbers_mtime > cache_mtime:
        return False

    datadir = config.get_abspath("data")
    relation_path = os.path.join(datadir, "relation-%s.yaml" % relation.get_name())
    if os.path.exists(relation_path) and os.path.getmtime(relation_path) > cache_mtime:
        return False

    return True


def get_missing_housenumbers_html(relation: areas.Relation) -> yattag.doc.Doc:
    """Gets the cached HTML of the missing housenumbers for a relation."""
    doc = yattag.doc.Doc()
    if is_missing_housenumbers_html_cached(relation):
        with relation.get_files().get_housenumbers_htmlcache_stream("r") as stream:
            doc.asis(stream.read())
        return doc

    ret = relation.write_missing_housenumbers()
    todo_street_count, todo_count, done_count, percent, table = ret

    with doc.tag("p"):
        prefix = config.Config.get_uri_prefix()
        relation_name = relation.get_name()
        doc.text(_("OpenStreetMap is possibly missing the below {0} house numbers for {1} streets.")
                 .format(str(todo_count), str(todo_street_count)))
        doc.text(_(" (existing: {0}, ready: {1}).").format(str(done_count), util.format_percent(str(percent))))
        doc.stag("br")
        with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
            doc.text(_("Filter incorrect information"))
        doc.text(".")
        doc.stag("br")
        with doc.tag("a", href=prefix + "/missing-housenumbers/{}/view-turbo".format(relation_name)):
            doc.text(_("Overpass turbo query for the below streets"))
        doc.stag("br")
        with doc.tag("a", href=prefix + "/missing-housenumbers/{}/view-result.txt".format(relation_name)):
            doc.text(_("Plain text format"))
        doc.stag("br")
        with doc.tag("a", href=prefix + "/missing-housenumbers/{}/view-result.chkl".format(relation_name)):
            doc.text(_("Checklist format"))

    doc.asis(util.html_table_from_list(table).getvalue())
    doc.asis(util.invalid_refstreets_to_html(areas.get_invalid_refstreets(relation)).getvalue())
    doc.asis(util.invalid_filter_keys_to_html(areas.get_invalid_filter_keys(relation)).getvalue())

    with relation.get_files().get_housenumbers_htmlcache_stream("w") as stream:
        stream.write(doc.getvalue())

    return doc

# vim:set shiftwidth=4 softtabstop=4 expandtab:
