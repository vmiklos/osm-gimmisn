#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

import json
import locale
import os
import subprocess
import sys
import urllib.parse
from typing import Any
from typing import Callable
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
import wsgiref.simple_server

import yattag

from i18n import translate as _
import areas
import config
import overpass_query
import util
import webframe

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse

if sys.platform.startswith("win"):
    import _locale


def handle_streets(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/streets/ormezo/view-query."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.doc.Doc()
    doc.asis(webframe.get_toolbar(relations, "streets", relation_name, osmrelation).getvalue())

    prefix = config.Config.get_uri_prefix()
    if action == "view-query":
        with doc.tag("pre"):
            doc.text(relation.get_osm_streets_query())
    elif action == "update-result":
        query = relation.get_osm_streets_query()
        try:
            relation.get_files().write_osm_streets(overpass_query.overpass_query(query))
            streets = relation.get_config().should_check_missing_streets()
            if streets != "only":
                doc.text(_("Update successful: "))
                link = prefix + "/missing-housenumbers/" + relation_name + "/view-result"
                doc.asis(util.gen_link(link, _("View missing house numbers")).getvalue())
            else:
                doc.text(_("Update successful."))
        except urllib.error.HTTPError as http_error:
            doc.asis(util.handle_overpass_error(http_error).getvalue())
    else:
        # assume view-result
        with relation.get_files().get_osm_streets_csv_stream() as sock:
            table = util.tsv_to_list(sock)
            doc.asis(util.html_table_from_list(table).getvalue())

    doc.asis(webframe.get_footer(get_streets_last_modified(relation)).getvalue())
    return doc


def handle_street_housenumbers(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.doc.Doc()
    doc.asis(webframe.get_toolbar(relations, "street-housenumbers", relation_name, osmrelation).getvalue())

    prefix = config.Config.get_uri_prefix()
    if action == "view-query":
        with doc.tag("pre"):
            doc.text(relation.get_osm_housenumbers_query())
    elif action == "update-result":
        query = relation.get_osm_housenumbers_query()
        try:
            relation.get_files().write_osm_housenumbers(overpass_query.overpass_query(query))
            doc.text(_("Update successful: "))
            link = prefix + "/missing-housenumbers/" + relation_name + "/view-result"
            doc.asis(util.gen_link(link, _("View missing house numbers")).getvalue())
        except urllib.error.HTTPError as http_error:
            doc.asis(util.handle_overpass_error(http_error).getvalue())
    else:
        # assume view-result
        if not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
            with doc.tag("div", id="no-osm-housenumbers"):
                doc.text(_("No existing house numbers"))
        else:
            with relation.get_files().get_osm_housenumbers_csv_stream() as sock:
                table = util.tsv_to_list(sock)
                doc.asis(util.html_table_from_list(table).getvalue())

    date = get_housenumbers_last_modified(relation)
    doc.asis(webframe.get_footer(date).getvalue())
    return doc


def missing_housenumbers_view_turbo(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-turbo."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.doc.Doc()
    relation = relations.get_relation(relation_name)
    ongoing_streets, _ = relation.get_missing_housenumbers()
    streets: List[str] = []
    for result in ongoing_streets:
        # Street name, # of only_in_reference items.
        streets.append(result[0].get_osm_name())
    query = areas.make_turbo_query_for_streets(relation, streets)

    with doc.tag("pre"):
        doc.text(query)
    return doc


def missing_housenumbers_view_res(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.doc.Doc()
    relation = relations.get_relation(relation_name)
    prefix = config.Config.get_uri_prefix()
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        with doc.tag("div", id="no-osm-streets"):
            doc.text(_("No existing streets: "))
            link = prefix + "/streets/" + relation_name + "/update-result"
            doc.asis(util.gen_link(link, _("Call Overpass to create")).getvalue())
    elif not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
        with doc.tag("div", id="no-osm-housenumbers"):
            doc.text(_("No existing house numbers: "))
            link = prefix + "/street-housenumbers/" + relation_name + "/update-result"
            doc.asis(util.gen_link(link, _("Call Overpass to create")).getvalue())
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        with doc.tag("div", id="no-ref-housenumbers"):
            doc.text(_("No missing house numbers: "))
            link = prefix + "/missing-housenumbers/" + relation_name + "/update-result"
            doc.asis(util.gen_link(link, _("Create from reference")).getvalue())
    else:
        ret = relation.write_missing_housenumbers()
        todo_street_count, todo_count, done_count, percent, table = ret

        with doc.tag("p"):
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
            doc.text(".")

        doc.asis(util.html_table_from_list(table).getvalue())
    return doc


def missing_streets_view_result(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/budapest_11/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    doc = yattag.doc.Doc()
    prefix = config.Config.get_uri_prefix()
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        with doc.tag("div", id="no-osm-streets"):
            doc.text(_("No existing streets: "))
            with doc.tag("a", href=prefix + "/streets/" + relation_name + "/update-result"):
                doc.text(_("Call Overpass to create"))
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        with doc.tag("div", id="no-ref-streets"):
            doc.text(_("No street list: "))
            with doc.tag("a", href=prefix + "/missing-streets/" + relation_name + "/update-result"):
                doc.text(_("Create from reference"))
    else:
        ret = relation.write_missing_streets()
        todo_count, done_count, percent, streets = ret
        streets.sort(key=locale.strxfrm)
        table = [[util.html_escape(_("Street name"))]]
        for street in streets:
            table.append([util.html_escape(street)])

        with doc.tag("p"):
            doc.text(_("OpenStreetMap is possibly missing the below {0} streets.").format(str(todo_count)))
            doc.text(_(" (existing: {0}, ready: {1}).").format(str(done_count), util.format_percent(str(percent))))
            doc.stag("br")
            with doc.tag("a", href=prefix + "/missing-streets/{}/view-turbo".format(relation_name)):
                doc.text(_("Overpass turbo query for streets with questionable names"))
            doc.text(".")

        doc.asis(util.html_table_from_list(table).getvalue())
    return doc


def missing_housenumbers_view_txt(relations: areas.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)
    relation.get_config().set_letter_suffix_style(util.LetterSuffixStyle.LOWER)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets")
    elif not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
        output += _("No existing house numbers")
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        output += _("No reference house numbers")
    else:
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
        table.sort(key=locale.strxfrm)
        output += "\n".join(table)
    return output


def get_chkl_split_limit() -> int:
    """Decides when to split a too long line in the chkl output."""
    return 20


def missing_housenumbers_view_chkl(relations: areas.Relations, request_uri: str) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.chkl."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)
    relation.get_config().set_letter_suffix_style(util.LetterSuffixStyle.LOWER)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets")
    elif not os.path.exists(relation.get_files().get_osm_housenumbers_path()):
        output += _("No existing house numbers")
    elif not os.path.exists(relation.get_files().get_ref_housenumbers_path()):
        output += _("No reference house numbers")
    else:
        ongoing_streets, _ignore = relation.get_missing_housenumbers()

        table = []
        for result in ongoing_streets:
            range_list = util.get_housenumber_ranges(result[1])
            # Street name, only_in_reference items.
            row = "[ ] "
            if not relation.get_config().get_street_is_even_odd(result[0].get_osm_name()):
                result_sorted = sorted([i.get_number() for i in range_list], key=util.split_house_number)
                row += result[0].get_osm_name() + " [" + ", ".join(result_sorted) + "]"
                table.append(row)
            else:
                elements = util.format_even_odd(range_list, doc=None)
                if len(elements) > 1 and len(range_list) > get_chkl_split_limit():
                    for element in elements:
                        row = "[ ] " + result[0].get_osm_name() + " [" + element + "]"
                        table.append(row)
                else:
                    row += result[0].get_osm_name() + " [" + "], [".join(elements) + "]"
                    table.append(row)
        table.sort(key=locale.strxfrm)
        output += "\n".join(table)
    return output, relation_name


def missing_streets_view_txt(relations: areas.Relations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-result.txt."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    output = ""
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        output += _("No existing streets")
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        output += _("No reference streets")
    else:
        todo_streets, _ignore = relation.get_missing_streets()
        todo_streets.sort(key=locale.strxfrm)
        output += "\n".join(todo_streets)
    return output


def missing_housenumbers_update(relations: areas.Relations, relation_name: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/update-result."""
    references = config.Config.get_reference_housenumber_paths()
    relation = relations.get_relation(relation_name)
    relation.write_ref_housenumbers(references)
    doc = yattag.doc.Doc()
    doc.text(_("Update successful: "))
    prefix = config.Config.get_uri_prefix()
    link = prefix + "/missing-housenumbers/" + relation_name + "/view-result"
    doc.asis(util.gen_link(link, _("View missing house numbers")).getvalue())
    return doc


def missing_streets_update(relations: areas.Relations, relation_name: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/update-result."""
    reference = config.Config.get_reference_street_path()
    relation = relations.get_relation(relation_name)
    relation.write_ref_streets(reference)
    doc = yattag.doc.Doc()
    with doc.tag("div", id="update-success"):
        doc.text(_("Update successful."))
    return doc


def handle_missing_housenumbers(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]
    date = None

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()
    doc = yattag.doc.Doc()
    doc.asis(webframe.get_toolbar(relations, "missing-housenumbers", relation_name, osmrelation).getvalue())

    if action == "view-turbo":
        doc.asis(missing_housenumbers_view_turbo(relations, request_uri).getvalue())
    elif action == "view-query":
        with doc.tag("pre"):
            with relation.get_files().get_ref_housenumbers_stream("r") as sock:
                doc.text(sock.read())
        date = get_last_modified(relation.get_files().get_ref_housenumbers_path())
    elif action == "update-result":
        doc.asis(missing_housenumbers_update(relations, relation_name).getvalue())
    else:
        # assume view-result
        doc.asis(missing_housenumbers_view_res(relations, request_uri).getvalue())

    if not date:
        date = ref_housenumbers_last_modified(relations, relation_name)
    doc.asis(webframe.get_footer(date).getvalue())
    return doc


def missing_streets_view_turbo(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ormezo/view-turbo."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.doc.Doc()
    relation = relations.get_relation(relation_name)
    refstreets = relation.get_config().get_refstreets()
    streets: List[str] = []
    for key, _value in refstreets.items():
        if relation.should_show_ref_street(key):
            streets.append(key)
    query = areas.make_turbo_query_for_streets(relation, streets)

    with doc.tag("pre"):
        doc.text(query)
    return doc


def handle_missing_streets(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.doc.Doc()
    doc.asis(webframe.get_toolbar(relations, "missing-streets", relation_name, osmrelation).getvalue())

    if action == "view-turbo":
        doc.asis(missing_streets_view_turbo(relations, request_uri).getvalue())
    elif action == "view-query":
        with doc.tag("pre"):
            with relation.get_files().get_ref_streets_stream("r") as sock:
                doc.text(sock.read())
    elif action == "update-result":
        doc.asis(missing_streets_update(relations, relation_name).getvalue())
    else:
        # assume view-result
        doc.asis(missing_streets_view_result(relations, request_uri).getvalue())

    date = ref_streets_last_modified(relation)
    doc.asis(webframe.get_footer(date).getvalue())
    return doc


def handle_additional_streets(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    # tokens[-1] would be the action

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.doc.Doc()
    doc.asis(webframe.get_toolbar(relations, "additional-streets", relation_name, osmrelation).getvalue())

    # assume view-result
    doc.asis(additional_streets_view_result(relations, request_uri).getvalue())

    doc.asis(webframe.get_footer().getvalue())
    return doc


def additional_streets_view_result(relations: areas.Relations, request_uri: str) -> yattag.doc.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    relation = relations.get_relation(relation_name)

    doc = yattag.doc.Doc()
    prefix = config.Config.get_uri_prefix()
    if not os.path.exists(relation.get_files().get_osm_streets_path()):
        with doc.tag("div", id="no-osm-streets"):
            doc.text(_("No existing streets: "))
            with doc.tag("a", href=prefix + "/streets/" + relation_name + "/update-result"):
                doc.text(_("Call Overpass to create"))
    elif not os.path.exists(relation.get_files().get_ref_streets_path()):
        with doc.tag("div", id="no-ref-streets"):
            doc.text(_("No street list: "))
            with doc.tag("a", href=prefix + "/missing-streets/" + relation_name + "/update-result"):
                doc.text(_("Create from reference"))
    else:
        # Get "only in OSM" streets.
        streets, _ignore = relation.get_additional_streets()
        count = len(streets)
        streets.sort(key=locale.strxfrm)
        table = [[util.html_escape(_("Street name"))]]
        for street in streets:
            table.append([util.html_escape(street)])

        with doc.tag("p"):
            doc.text(_("OpenStreetMap additionally has the below {0} streets.").format(str(count)))

        doc.asis(util.html_table_from_list(table).getvalue())
    return doc


def get_last_modified(path: str) -> str:
    """Gets the update date string of a file."""
    return webframe.format_timestamp(get_timestamp(path))


def get_timestamp(path: str) -> float:
    """Gets the timestamp of a file if it exists, 0 otherwise."""
    try:
        return os.path.getmtime(path)
    except FileNotFoundError:
        return 0


def ref_housenumbers_last_modified(relations: areas.Relations, name: str) -> str:
    """Gets the update date for missing house numbers."""
    relation = relations.get_relation(name)
    t_ref = get_timestamp(relation.get_files().get_ref_housenumbers_path())
    t_housenumbers = get_timestamp(relation.get_files().get_osm_housenumbers_path())
    return webframe.format_timestamp(max(t_ref, t_housenumbers))


def ref_streets_last_modified(relation: areas.Relation) -> str:
    """Gets the update date for missing streets."""
    t_ref = get_timestamp(relation.get_files().get_ref_streets_path())
    t_osm = get_timestamp(relation.get_files().get_osm_streets_path())
    return webframe.format_timestamp(max(t_ref, t_osm))


def get_housenumbers_last_modified(relation: areas.Relation) -> str:
    """Gets the update date of house numbers for a relation."""
    return get_last_modified(relation.get_files().get_osm_housenumbers_path())


def get_streets_last_modified(relation: areas.Relation) -> str:
    """Gets the update date of streets for a relation."""
    return get_last_modified(relation.get_files().get_osm_streets_path())


def handle_main_housenr_percent(relation: areas.Relation) -> Tuple[yattag.doc.Doc, str]:
    """Handles the house number percent part of the main page."""
    prefix = config.Config.get_uri_prefix()
    url = prefix + "/missing-housenumbers/" + relation.get_name() + "/view-result"
    percent = "N/A"
    if os.path.exists(relation.get_files().get_housenumbers_percent_path()):
        percent = util.get_content(relation.get_files().get_housenumbers_percent_path())

    doc = yattag.doc.Doc()
    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_housenumbers_percent_path())
        with doc.tag("strong"):
            with doc.tag("a", href=url, title=_("updated") + " " + date):
                doc.text(util.format_percent(percent))
        return doc, percent

    with doc.tag("strong"):
        with doc.tag("a", href=url):
            doc.text(_("missing house numbers"))
    return doc, "0"


def handle_main_street_percent(relation: areas.Relation) -> Tuple[yattag.doc.Doc, str]:
    """Handles the street percent part of the main page."""
    prefix = config.Config.get_uri_prefix()
    url = prefix + "/missing-streets/" + relation.get_name() + "/view-result"
    percent = "N/A"
    if os.path.exists(relation.get_files().get_streets_percent_path()):
        percent = util.get_content(relation.get_files().get_streets_percent_path())

    doc = yattag.doc.Doc()
    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_streets_percent_path())
        with doc.tag("strong"):
            with doc.tag("a", href=url, title=_("updated") + " " + date):
                doc.text(util.format_percent(percent))
        return doc, percent

    with doc.tag("strong"):
        with doc.tag("a", href=url):
            doc.text(_("missing streets"))
    return doc, "0"


def filter_for_everything(_complete: bool, _relation: areas.Relation) -> bool:
    """Does not filter out anything."""
    return True


def filter_for_incomplete(complete: bool, _relation: areas.Relation) -> bool:
    """Filters out complete items."""
    return not complete


def create_filter_for_refcounty(refcounty_filter: str) -> Callable[[bool, areas.Relation], bool]:
    """Creates a function that filters for a single refcounty."""
    return lambda _complete, relation: relation.get_config().get_refcounty() == refcounty_filter


def create_filter_for_refcounty_refsettlement(
        refcounty_filter: str,
        refsettlement_filter: str
) -> Callable[[bool, areas.Relation], bool]:
    """Creates a function that filters for a single refsettlement in a refcounty."""
    def filter_for(_complete: bool, relation: areas.Relation) -> bool:
        r_config = relation.get_config()
        return r_config.get_refcounty() == refcounty_filter and r_config.get_refsettlement() == refsettlement_filter
    return filter_for


def handle_main_filters_refcounty(relations: areas.Relations, refcounty_id: str, refcounty: str) -> yattag.doc.Doc:
    """Handles one refcounty in the filter part of the main wsgi page."""
    doc = yattag.doc.Doc()
    name = relations.refcounty_get_name(refcounty)
    if not name:
        return doc

    prefix = config.Config.get_uri_prefix()
    with doc.tag("a", href=prefix + "/filter-for/refcounty/" + refcounty):
        doc.text(name)
    if refcounty_id and refcounty == refcounty_id:
        refsettlement_ids = relations.refcounty_get_refsettlement_ids(refcounty_id)
        if refsettlement_ids:
            names: List[yattag.doc.Doc] = []
            for refsettlement_id in refsettlement_ids:
                name = relations.refsettlement_get_name(refcounty_id, refsettlement_id)
                name_doc = yattag.doc.Doc()
                href_format = prefix + "/filter-for/refcounty/{}/refsettlement/{}"
                with name_doc.tag("a", href=href_format.format(refcounty, refsettlement_id)):
                    name_doc.text(name)
                names.append(name_doc)
            doc.text(" (")
            for index, item in enumerate(names):
                if index:
                    doc.text(", ")
                doc.asis(item.getvalue())
            doc.text(")")
    return doc


def handle_main_filters(relations: areas.Relations, refcounty_id: str) -> yattag.doc.Doc:
    """Handlers the filter part of the main wsgi page."""
    items: List[yattag.doc.Doc] = []
    doc = yattag.doc.Doc()
    prefix = config.Config.get_uri_prefix()
    with doc.tag("a", href=prefix + "/filter-for/incomplete"):
        doc.text(_("Hide complete areas"))
    items.append(doc)
    # Sorted set of refcounty values of all relations.
    for refcounty in sorted({relation.get_config().get_refcounty() for relation in relations.get_relations()}):
        items.append(handle_main_filters_refcounty(relations, refcounty_id, refcounty))
    doc = yattag.doc.Doc()
    with doc.tag("h1"):
        doc.text(_("Where to map?"))
    with doc.tag("p"):
        doc.text(_("Filters:") + " ")
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.asis(item.getvalue())
    return doc


def setup_main_filter_for(request_uri: str) -> Tuple[Callable[[bool, areas.Relation], bool], str]:
    """Sets up a filter-for function from request uri: only certain areas are shown then."""
    tokens = request_uri.split("/")
    filter_for: Callable[[bool, areas.Relation], bool] = filter_for_everything
    filters = util.parse_filters(tokens)
    refcounty = ""
    if "incomplete" in filters:
        # /osm/filter-for/incomplete
        filter_for = filter_for_incomplete
    elif "refcounty" in filters and "refsettlement" in filters:
        # /osm/filter-for/refcounty/<value>/refsettlement/<value>.
        refcounty = filters["refcounty"]
        filter_for = create_filter_for_refcounty_refsettlement(filters["refcounty"], filters["refsettlement"])
    elif "refcounty" in filters:
        # /osm/filter-for/refcounty/<value>.
        refcounty = filters["refcounty"]
        filter_for = create_filter_for_refcounty(refcounty)
    return filter_for, refcounty


def handle_main_relation(
        relations: areas.Relations,
        filter_for: Callable[[bool, areas.Relation], bool],
        relation_name: str
) -> List[yattag.doc.Doc]:
    """Handles one relation (one table row) on the main page."""
    relation = relations.get_relation(relation_name)
    # If checking both streets and house numbers, then "is complete" refers to the street coverage
    # for "hide complete" purposes.
    complete = True

    streets = relation.get_config().should_check_missing_streets()

    row = []  # List[yattag.doc.Doc]
    row.append(util.html_escape(relation_name))

    prefix = config.Config.get_uri_prefix()
    if streets != "only":
        cell, percent = handle_main_housenr_percent(relation)
        doc = yattag.doc.Doc()
        doc.asis(cell.getvalue())
        row.append(doc)
        complete = float(percent) >= 100.0

        date = get_housenumbers_last_modified(relation)
        doc = yattag.doc.Doc()
        href = prefix + "/street-housenumbers/" + relation_name + "/view-result"
        with doc.tag("a", href=href, title=_("updated") + " " + date):
            doc.text(_("existing house numbers"))
        row.append(doc)
    else:
        row.append(yattag.doc.Doc())

        row.append(yattag.doc.Doc())

    if streets != "no":
        cell, percent = handle_main_street_percent(relation)
        row.append(cell)
        complete = float(percent) >= 100.0
    else:
        row.append(yattag.doc.Doc())

    date = get_streets_last_modified(relation)
    doc = yattag.doc.Doc()
    with doc.tag("a", href=prefix + "/streets/" + relation_name + "/view-result", title=_("updated") + " " + date):
        doc.text(_("existing streets"))
    row.append(doc)

    doc = yattag.doc.Doc()
    with doc.tag("a", href="https://www.openstreetmap.org/relation/" + str(relation.get_config().get_osmrelation())):
        doc.text(_("area boundary"))
    row.append(doc)

    if not filter_for(complete, relation):
        row.clear()

    return row


def handle_main(request_uri: str, relations: areas.Relations) -> yattag.doc.Doc:
    """Handles the main wsgi page.

    Also handles /osm/filter-for/* which filters for a condition."""
    filter_for, refcounty = setup_main_filter_for(request_uri)

    doc = yattag.doc.Doc()
    doc.asis(webframe.get_toolbar(relations).getvalue())

    doc.asis(handle_main_filters(relations, refcounty).getvalue())
    table = []
    table.append([util.html_escape(_("Area")),
                  util.html_escape(_("House number coverage")),
                  util.html_escape(_("Existing house numbers")),
                  util.html_escape(_("Street coverage")),
                  util.html_escape(_("Existing streets")),
                  util.html_escape(_("Area boundary"))])
    for relation_name in relations.get_names():
        row = handle_main_relation(relations, filter_for, relation_name)
        if row:
            table.append(row)
    doc.asis(util.html_table_from_list(table).getvalue())
    with doc.tag("p"):
        with doc.tag("a", href="https://github.com/vmiklos/osm-gimmisn/tree/master/doc"):
            doc.text(_("Add new area"))

    doc.asis(webframe.get_footer().getvalue())
    return doc


def get_html_title(request_uri: str) -> str:
    """Determines the HTML title for a given function and relation name."""
    tokens = request_uri.split("/")
    function = ""
    relation_name = ""
    if len(tokens) > 3:
        function = tokens[2]
        relation_name = tokens[3]
    title = ""
    if function == "missing-housenumbers":
        title = " - " + _("{0} missing house numbers").format(relation_name)
    elif function == "missing-streets":
        title = " - " + relation_name + " " + _("missing streets")
    elif function == "street-housenumbers":
        title = " - " + relation_name + " " + _("existing house numbers")
    elif function == "streets":
        title = " - " + relation_name + " " + _("existing streets")
    return title


def write_html_head(doc: yattag.doc.Doc, title: str) -> None:
    """Produces the <head> tag and its contents."""
    prefix = config.Config.get_uri_prefix()
    with doc.tag("head"):
        with doc.tag("title"):
            doc.text(_("Where to map?") + title)
        doc.stag("meta", charset="UTF-8")
        doc.stag("link", rel="stylesheet", type="text/css", href=prefix + "/static/osm.css")
        with doc.tag("script", src=prefix + "/static/sorttable.js"):
            pass
        doc.stag("meta", name="viewport", content="width=device-width, initial-scale=1")


def handle_github_webhook(environ: Dict[str, Any]) -> yattag.doc.Doc:
    """Handles a GitHub style webhook."""

    body = urllib.parse.parse_qs(environ["wsgi.input"].read().decode('utf-8'))
    payload = body["payload"][0]
    root = json.loads(payload)
    if root["ref"] == "refs/heads/master":
        my_env = os.environ
        my_env["PATH"] = "osm-gimmisn-env/bin:" + my_env["PATH"]
        subprocess.run(["make", "-C", config.get_abspath(""), "deploy"], check=True, env=my_env)

    return util.html_escape("")


def our_application_txt(
        start_response: 'StartResponse',
        relations: areas.Relations,
        request_uri: str
) -> Iterable[bytes]:
    """Dispatches plain text requests based on their URIs."""
    content_type = "text/plain"
    extra_headers: List[Tuple[str, str]] = []
    prefix = config.Config.get_uri_prefix()
    if request_uri.startswith(prefix + "/missing-streets/"):
        output = missing_streets_view_txt(relations, request_uri)
    else:
        # assume prefix + "/missing-housenumbers/"
        _, _, ext = request_uri.partition('.')
        if ext == "chkl":
            output, relation_name = missing_housenumbers_view_chkl(relations, request_uri)
            content_type = "application/octet-stream"
            extra_headers.append(("Content-Disposition", 'attachment;filename="' + relation_name + '.txt"'))
        elif request_uri.endswith("robots.txt"):
            output = util.get_content(config.get_abspath("data"), "robots.txt")
        else:
            # assume txt
            output = missing_housenumbers_view_txt(relations, request_uri)
    return webframe.send_response(start_response, content_type, "200 OK", output, extra_headers)


HANDLERS = {
    "/streets/": handle_streets,
    "/missing-streets/": handle_missing_streets,
    "/additional-streets/": handle_additional_streets,
    "/street-housenumbers/": handle_street_housenumbers,
    "/missing-housenumbers/": handle_missing_housenumbers,
    "/housenumber-stats/": webframe.handle_stats,
}


def get_handler(request_uri: str) -> Optional[Callable[[areas.Relations, str], yattag.doc.Doc]]:
    """Decides request_uri matches what handler."""
    prefix = config.Config.get_uri_prefix()
    for key, value in HANDLERS.items():
        if request_uri.startswith(prefix + key):
            return value
    return None


def our_application(
        environ: Dict[str, Any],
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """Dispatches the request based on its URI."""
    util.set_locale()

    language = util.setup_localization(environ)

    relations = areas.Relations(config.Config.get_workdir())

    request_uri = webframe.get_request_uri(environ, relations)
    _, _, ext = request_uri.partition('.')

    if ext in ("txt", "chkl"):
        return our_application_txt(start_response, relations, request_uri)

    prefix = config.Config.get_uri_prefix()
    if request_uri.startswith(prefix + "/static/"):
        output, content_type = webframe.handle_static(request_uri)
        return webframe.send_response(start_response, content_type, "200 OK", output, [])

    doc = yattag.doc.Doc()
    util.write_html_header(doc)
    with doc.tag("html", lang=language):
        write_html_head(doc, get_html_title(request_uri))

        with doc.tag("body"):
            no_such_relation = webframe.check_existing_relation(relations, request_uri)
            handler = get_handler(request_uri)
            if no_such_relation.getvalue():
                doc.asis(no_such_relation.getvalue())
            elif handler:
                doc.asis(handler(relations, request_uri).getvalue())
            elif request_uri.startswith(prefix + "/webhooks/github"):
                doc.asis(handle_github_webhook(environ).getvalue())
            else:
                doc.asis(handle_main(request_uri, relations).getvalue())

    return webframe.send_response(start_response, "text/html", "200 OK", doc.getvalue(), [])


def application(
        environ: Dict[str, Any],
        start_response: 'StartResponse'
) -> Iterable[bytes]:
    """The entry point of this WSGI app."""
    try:
        return our_application(environ, start_response)

    # pylint: disable=broad-except
    except Exception:
        return webframe.handle_exception(environ, start_response)


def main() -> None:
    """Commandline interface to this module."""
    if sys.platform.startswith("win"):
        # pylint: disable=protected-access
        _locale._getdefaultlocale = (lambda *args: ['en_US', 'utf8'])

    port = config.Config.get_tcp_port()
    prefix = config.Config.get_uri_prefix()
    httpd = wsgiref.simple_server.make_server('', port, application)
    print("Open <http://localhost:" + str(port) + prefix + "/> in your browser.")
    httpd.serve_forever()


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
