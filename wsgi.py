#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

from typing import Any
from typing import Callable
from typing import Dict
from typing import Iterable
from typing import List
from typing import Optional
from typing import TYPE_CHECKING
from typing import Tuple
import os
import traceback

import yattag

from rust import py_translate as tr
import areas
import rust
import util
import webframe
import wsgi_additional
import wsgi_json

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def handle_streets(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/streets/ormezo/view-query."""
    return rust.py_handle_streets(ctx, relations, request_uri)


def handle_street_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    return rust.py_handle_street_housenumbers(ctx, relations, request_uri)


def missing_streets_view_result(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/budapest_11/view-result."""
    return rust.py_missing_streets_view_result(ctx, relations, request_uri)


def missing_housenumbers_view_txt(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> str:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.txt."""
    return rust.py_missing_housenumbers_view_txt(ctx, relations, request_uri)


def missing_housenumbers_view_chkl(
        ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.chkl."""
    return rust.py_missing_housenumbers_view_chkl(ctx, relations, request_uri)


def missing_streets_view_txt(
    ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str, chkl: bool
) -> Tuple[str, str]:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-result.txt."""
    return rust.py_missing_streets_view_txt(ctx, relations, request_uri, chkl)


def missing_streets_update(ctx: rust.PyContext, relations: rust.PyRelations, relation_name: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/update-result."""
    return rust.py_missing_streets_update(ctx, relations, relation_name)


def handle_missing_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-[result|query]."""
    return rust.py_handle_missing_housenumbers(ctx, relations, request_uri)


def missing_streets_view_turbo(relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ormezo/view-turbo."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]

    doc = yattag.Doc()
    relation = relations.get_relation(relation_name)
    refstreets = relation.get_config().get_refstreets()
    streets: List[str] = []
    for key, _value in refstreets.items():
        if relation.should_show_ref_street(key):
            streets.append(key)
    query = areas.make_turbo_query_for_streets(relation, streets)

    with doc.tag("pre", []):
        doc.text(query)
    return doc


def handle_missing_streets(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.Doc()
    doc.append_value(webframe.get_toolbar(ctx, relations, "missing-streets", relation_name, osmrelation).get_value())

    if action == "view-turbo":
        doc.append_value(missing_streets_view_turbo(relations, request_uri).get_value())
    elif action == "view-query":
        with doc.tag("pre", []):
            with relation.get_files().get_ref_streets_read_stream(ctx) as sock:
                doc.text(util.from_bytes(sock.read()))
    elif action == "update-result":
        doc.append_value(missing_streets_update(ctx, relations, relation_name).get_value())
    else:
        # assume view-result
        doc.append_value(missing_streets_view_result(ctx, relations, request_uri).get_value())

    date = streets_diff_last_modified(relation)
    doc.append_value(webframe.get_footer(date).get_value())
    return doc


def handle_additional_streets(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    action = tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.Doc()
    doc.append_value(webframe.get_toolbar(ctx, relations, "additional-streets", relation_name, osmrelation).get_value())

    if action == "view-turbo":
        doc.append_value(wsgi_additional.additional_streets_view_turbo(relations, request_uri).get_value())
    else:
        # assume view-result
        doc.append_value(wsgi_additional.additional_streets_view_result(ctx, relations, request_uri).get_value())

    date = streets_diff_last_modified(relation)
    doc.append_value(webframe.get_footer(date).get_value())
    return doc


def handle_additional_housenumbers(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str
) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ujbuda/view-[result|query]."""
    tokens = request_uri.split("/")
    relation_name = tokens[-2]
    # action would be tokens[-1]

    relation = relations.get_relation(relation_name)
    osmrelation = relation.get_config().get_osmrelation()

    doc = yattag.Doc()
    doc.append_value(webframe.get_toolbar(ctx, relations, "additional-housenumbers", relation_name, osmrelation).get_value())

    # assume action is view-result
    doc.append_value(wsgi_additional.additional_housenumbers_view_result(ctx, relations, request_uri).get_value())

    date = housenumbers_diff_last_modified(relation)
    doc.append_value(webframe.get_footer(date).get_value())
    return doc


def get_last_modified(path: str) -> str:
    """Gets the update date string of a file."""
    return webframe.format_timestamp(util.get_timestamp(path))


def streets_diff_last_modified(relation: rust.PyRelation) -> str:
    """Gets the update date for missing/additional streets."""
    t_ref = util.get_timestamp(relation.get_files().get_ref_streets_path())
    t_osm = util.get_timestamp(relation.get_files().get_osm_streets_path())
    return webframe.format_timestamp(max(t_ref, t_osm))


def housenumbers_diff_last_modified(relation: rust.PyRelation) -> str:
    """Gets the update date for missing/additional housenumbers."""
    t_ref = util.get_timestamp(relation.get_files().get_ref_housenumbers_path())
    t_osm = util.get_timestamp(relation.get_files().get_osm_housenumbers_path())
    return webframe.format_timestamp(max(t_ref, t_osm))


def handle_main_housenr_percent(ctx: rust.PyContext, relation: rust.PyRelation) -> Tuple[yattag.Doc, str]:
    """Handles the house number percent part of the main page."""
    prefix = ctx.get_ini().get_uri_prefix()
    url = prefix + "/missing-housenumbers/" + relation.get_name() + "/view-result"
    percent = "N/A"
    if ctx.get_file_system().path_exists(relation.get_files().get_housenumbers_percent_path()):
        with relation.get_files().get_housenumbers_percent_read_stream(ctx) as stream:
            percent = util.from_bytes(stream.read())

    doc = yattag.Doc()
    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_housenumbers_percent_path())
        with doc.tag("strong", []):
            with doc.tag("a", [("href", url), ("title", tr("updated") + " " + date)]):
                doc.text(util.format_percent(percent))
        return doc, percent

    with doc.tag("strong", []):
        with doc.tag("a", [("href", url)]):
            doc.text(tr("missing house numbers"))
    return doc, "0"


def handle_main_street_percent(ctx: rust.PyContext, relation: rust.PyRelation) -> Tuple[yattag.Doc, str]:
    """Handles the street percent part of the main page."""
    prefix = ctx.get_ini().get_uri_prefix()
    url = prefix + "/missing-streets/" + relation.get_name() + "/view-result"
    percent = "N/A"
    if ctx.get_file_system().path_exists(relation.get_files().get_streets_percent_path()):
        with relation.get_files().get_streets_percent_read_stream(ctx) as stream:
            percent = util.from_bytes(stream.read())

    doc = yattag.Doc()
    if percent != "N/A":
        date = get_last_modified(relation.get_files().get_streets_percent_path())
        with doc.tag("strong", []):
            with doc.tag("a", [("href", url), ("title", tr("updated") + " " + date)]):
                doc.text(util.format_percent(percent))
        return doc, percent

    with doc.tag("strong", []):
        with doc.tag("a", [("href", url)]):
            doc.text(tr("missing streets"))
    return doc, "0"


def handle_main_street_additional_count(ctx: rust.PyContext, relation: rust.PyRelation) -> yattag.Doc:
    """Handles the street additional count part of the main page."""
    prefix = ctx.get_ini().get_uri_prefix()
    url = prefix + "/additional-streets/" + relation.get_name() + "/view-result"
    additional_count = ""
    if ctx.get_file_system().path_exists(relation.get_files().get_streets_additional_count_path()):
        with relation.get_files().get_streets_additional_count_read_stream(ctx) as stream:
            additional_count = util.from_bytes(stream.read())

    doc = yattag.Doc()
    if additional_count:
        date = get_last_modified(relation.get_files().get_streets_additional_count_path())
        with doc.tag("strong", []):
            with doc.tag("a", [("href", url), ("title", tr("updated") + " " + date)]):
                doc.text(tr("{} streets").format(additional_count))
        return doc

    with doc.tag("strong", []):
        with doc.tag("a", [("href", url)]):
            doc.text(tr("additional streets"))
    return doc


def handle_main_housenr_additional_count(ctx: rust.PyContext, relation: rust.PyRelation) -> yattag.Doc:
    """Handles the housenumber additional count part of the main page."""
    if not relation.get_config().should_check_additional_housenumbers():
        return yattag.Doc()

    prefix = ctx.get_ini().get_uri_prefix()
    url = prefix + "/additional-housenumbers/" + relation.get_name() + "/view-result"
    additional_count = ""
    if ctx.get_file_system().path_exists(relation.get_files().get_housenumbers_additional_count_path()):
        with relation.get_files().get_housenumbers_additional_count_read_stream(ctx) as stream:
            additional_count = util.from_bytes(stream.read()).strip()

    doc = yattag.Doc()
    if additional_count:
        date = get_last_modified(relation.get_files().get_housenumbers_additional_count_path())
        with doc.tag("strong", []):
            with doc.tag("a", [("href", url), ("title", tr("updated") + " " + date)]):
                doc.text(tr("{} house numbers").format(additional_count))
        return doc

    with doc.tag("strong", []):
        with doc.tag("a", [("href", url)]):
            doc.text(tr("additional house numbers"))
    return doc


def filter_for_everything(_complete: bool, _relation: rust.PyRelation) -> bool:
    """Does not filter out anything."""
    return True


def filter_for_incomplete(complete: bool, _relation: rust.PyRelation) -> bool:
    """Filters out complete items."""
    return not complete


def create_filter_for_refcounty(refcounty_filter: str) -> Callable[[bool, rust.PyRelation], bool]:
    """Creates a function that filters for a single refcounty."""
    return lambda _complete, relation: relation.get_config().get_refcounty() == refcounty_filter


def create_filter_for_relations(relation_filter: str) -> Callable[[bool, rust.PyRelation], bool]:
    """Creates a function that filters for the specified relations."""
    relations: List[int] = []
    if relation_filter:
        relations = [int(i) for i in relation_filter.split(",")]
    return lambda _complete, relation: relation.get_config().get_osmrelation() in relations


def create_filter_for_refcounty_refsettlement(
        refcounty_filter: str,
        refsettlement_filter: str
) -> Callable[[bool, rust.PyRelation], bool]:
    """Creates a function that filters for a single refsettlement in a refcounty."""
    def filter_for(_complete: bool, relation: rust.PyRelation) -> bool:
        r_config = relation.get_config()
        return r_config.get_refcounty() == refcounty_filter and r_config.get_refsettlement() == refsettlement_filter
    return filter_for


def handle_main_filters_refcounty(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    refcounty_id: str,
    refcounty: str
) -> yattag.Doc:
    """Handles one refcounty in the filter part of the main wsgi page."""
    doc = yattag.Doc()
    name = relations.refcounty_get_name(refcounty)
    if not name:
        return doc

    prefix = ctx.get_ini().get_uri_prefix()
    with doc.tag("a", [("href", prefix + "/filter-for/refcounty/" + refcounty + "/whole-county")]):
        doc.text(name)
    if refcounty_id and refcounty == refcounty_id:
        refsettlement_ids = relations.refcounty_get_refsettlement_ids(refcounty_id)
        if refsettlement_ids:
            names: List[yattag.Doc] = []
            for refsettlement_id in refsettlement_ids:
                name = relations.refsettlement_get_name(refcounty_id, refsettlement_id)
                name_doc = yattag.Doc()
                href_format = prefix + "/filter-for/refcounty/{}/refsettlement/{}"
                with name_doc.tag("a", [("href", href_format.format(refcounty, refsettlement_id))]):
                    name_doc.text(name)
                names.append(name_doc)
            doc.text(" (")
            for index, item in enumerate(names):
                if index:
                    doc.text(", ")
                doc.append_value(item.get_value())
            doc.text(")")
    return doc


def handle_main_filters(ctx: rust.PyContext, relations: rust.PyRelations, refcounty_id: str) -> yattag.Doc:
    """Handlers the filter part of the main wsgi page."""
    items: List[yattag.Doc] = []

    doc = yattag.Doc()
    with doc.tag("span", [("id", "filter-based-on-position")]):
        with doc.tag("a", [("href", "#")]):
            doc.text(tr("Based on position"))
    items.append(doc)

    doc = yattag.Doc()
    prefix = ctx.get_ini().get_uri_prefix()
    with doc.tag("a", [("href", prefix + "/filter-for/everything")]):
        doc.text(tr("Show complete areas"))
    items.append(doc)

    # Sorted set of refcounty values of all relations.
    for refcounty in sorted({relation.get_config().get_refcounty() for relation in relations.get_relations()}):
        items.append(handle_main_filters_refcounty(ctx, relations, refcounty_id, refcounty))
    doc = yattag.Doc()
    with doc.tag("h1", []):
        doc.text(tr("Where to map?"))
    with doc.tag("p", []):
        doc.text(tr("Filters:") + " ")
        for index, item in enumerate(items):
            if index:
                doc.text(" ¦ ")
            doc.append_value(item.get_value())

    # Emit localized strings for JS purposes.
    with doc.tag("div", [("style", "display: none;")]):
        string_pairs = [
            ("str-gps-wait", tr("Waiting for GPS...")),
            ("str-gps-error", tr("Error from GPS: ")),
            ("str-overpass-wait", tr("Waiting for Overpass...")),
            ("str-overpass-error", tr("Error from Overpass: ")),
            ("str-relations-wait", tr("Waiting for relations...")),
            ("str-relations-error", tr("Error from relations: ")),
            ("str-redirect-wait", tr("Waiting for redirect...")),
        ]
        for key, value in string_pairs:
            with doc.tag("div", [("id", key), ("data-value", value)]):
                pass
    return doc


def setup_main_filter_for(request_uri: str) -> Tuple[Callable[[bool, rust.PyRelation], bool], str]:
    """Sets up a filter-for function from request uri: only certain areas are shown then."""
    tokens = request_uri.split("/")
    filter_for: Callable[[bool, rust.PyRelation], bool] = filter_for_incomplete
    filters = util.parse_filters(tokens)
    refcounty = ""
    if "incomplete" in filters:
        # /osm/filter-for/incomplete
        filter_for = filter_for_incomplete
    elif "everything" in filters:
        # /osm/filter-for/everything
        filter_for = filter_for_everything
    elif "refcounty" in filters and "refsettlement" in filters:
        # /osm/filter-for/refcounty/<value>/refsettlement/<value>
        refcounty = filters["refcounty"]
        filter_for = create_filter_for_refcounty_refsettlement(filters["refcounty"], filters["refsettlement"])
    elif "refcounty" in filters:
        # /osm/filter-for/refcounty/<value>/whole-county
        refcounty = filters["refcounty"]
        filter_for = create_filter_for_refcounty(refcounty)
    elif "relations" in filters:
        # /osm/filter-for/relations/<id1>,<id2>
        relations = filters["relations"]
        filter_for = create_filter_for_relations(relations)
    return filter_for, refcounty


def handle_main_relation(
        ctx: rust.PyContext,
        relations: rust.PyRelations,
        filter_for: Callable[[bool, rust.PyRelation], bool],
        relation_name: str
) -> List[yattag.Doc]:
    """Handles one relation (one table row) on the main page."""
    relation = relations.get_relation(relation_name)
    # If checking both streets and house numbers, then "is complete" refers to both street and
    # housenr coverage for "hide complete" purposes.
    complete = True

    streets = relation.get_config().should_check_missing_streets()

    row = []  # List[yattag.Doc]
    row.append(yattag.Doc.from_text(relation_name))

    if streets != "only":
        cell, percent = handle_main_housenr_percent(ctx, relation)
        doc = yattag.Doc()
        doc.append_value(cell.get_value())
        row.append(doc)
        complete &= float(percent) >= 100.0

        row.append(handle_main_housenr_additional_count(ctx, relation))
    else:
        row.append(yattag.Doc())
        row.append(yattag.Doc())

    if streets != "no":
        cell, percent = handle_main_street_percent(ctx, relation)
        row.append(cell)
        complete &= float(percent) >= 100.0
    else:
        row.append(yattag.Doc())

    if streets != "no":
        row.append(handle_main_street_additional_count(ctx, relation))
    else:
        row.append(yattag.Doc())

    doc = yattag.Doc()
    with doc.tag("a", [("href", "https://www.openstreetmap.org/relation/" + str(relation.get_config().get_osmrelation()))]):
        doc.text(tr("area boundary"))
    row.append(doc)

    if not filter_for(complete, relation):
        row.clear()

    return row


def handle_main(request_uri: str, ctx: rust.PyContext, relations: rust.PyRelations) -> yattag.Doc:
    """Handles the main wsgi page.

    Also handles /osm/filter-for/* which filters for a condition."""
    filter_for, refcounty = setup_main_filter_for(request_uri)

    doc = yattag.Doc()
    doc.append_value(webframe.get_toolbar(ctx, relations, function=str(), relation_name=str(), relation_osmid=0).get_value())

    doc.append_value(handle_main_filters(ctx, relations, refcounty).get_value())
    table = []
    table.append([yattag.Doc.from_text(tr("Area")),
                  yattag.Doc.from_text(tr("House number coverage")),
                  yattag.Doc.from_text(tr("Additional house numbers")),
                  yattag.Doc.from_text(tr("Street coverage")),
                  yattag.Doc.from_text(tr("Additional streets")),
                  yattag.Doc.from_text(tr("Area boundary"))])
    for relation_name in relations.get_names():
        row = handle_main_relation(ctx, relations, filter_for, relation_name)
        if row:
            table.append(row)
    doc.append_value(util.html_table_from_list(table).get_value())
    with doc.tag("p", []):
        with doc.tag("a", [("href", "https://github.com/vmiklos/osm-gimmisn/tree/master/doc")]):
            doc.text(tr("Add new area"))

    doc.append_value(webframe.get_footer(last_updated=str()).get_value())
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
        title = " - " + tr("{0} missing house numbers").format(relation_name)
    elif function == "missing-streets":
        title = " - " + relation_name + " " + tr("missing streets")
    elif function == "street-housenumbers":
        title = " - " + relation_name + " " + tr("existing house numbers")
    elif function == "streets":
        title = " - " + relation_name + " " + tr("existing streets")
    return title


def write_html_head(ctx: rust.PyContext, doc: yattag.Doc, title: str) -> None:
    """Produces the <head> tag and its contents."""
    prefix = ctx.get_ini().get_uri_prefix()
    with doc.tag("head", []):
        doc.stag("meta", [("charset", "UTF-8")])
        doc.stag("meta", [("name", "viewport"), ("content", "width=device-width, initial-scale=1")])
        with doc.tag("title", []):
            doc.text(tr("Where to map?") + title)
        doc.stag("link", [("rel", "icon"), ("type", "image/vnd.microsoft.icon"), ("sizes", "16x12"), ("href", prefix + "/favicon.ico")])
        doc.stag("link", [("rel", "icon"), ("type", "image/svg+xml"), ("sizes", "any"), ("href", prefix + "/favicon.svg")])

        css_path = os.path.join(ctx.get_ini().get_workdir(), "osm.min.css")
        with open(css_path, "r") as stream:
            with doc.tag("style", []):
                doc.text(stream.read())

        with doc.tag("noscript", []):
            with doc.tag("style", [("type", "text/css")]):
                doc.text(".no-js { display: block; }")
                doc.text(".js { display: none; }")

        with doc.tag("script", [("defer", ""), ("src", prefix + "/static/bundle.js")]):
            pass


def our_application_txt(
        environ: Dict[str, Any],
        start_response: 'StartResponse',
        ctx: rust.PyContext,
        relations: rust.PyRelations,
        request_uri: str
) -> Iterable[bytes]:
    """Dispatches plain text requests based on their URIs."""
    content_type = "text/plain"
    headers: List[Tuple[str, str]] = []
    prefix = ctx.get_ini().get_uri_prefix()
    _, _, ext = request_uri.partition('.')
    chkl = ext == "chkl"
    if request_uri.startswith(prefix + "/missing-streets/"):
        output, relation_name = missing_streets_view_txt(ctx, relations, request_uri, chkl)
        if chkl:
            content_type = "application/octet-stream"
            headers.append(("Content-Disposition", 'attachment;filename="' + relation_name + '.txt"'))
    elif request_uri.startswith(prefix + "/additional-streets/"):
        output, relation_name = wsgi_additional.additional_streets_view_txt(ctx, relations, request_uri, chkl)
        if chkl:
            content_type = "application/octet-stream"
            headers.append(("Content-Disposition", 'attachment;filename="' + relation_name + '.txt"'))
    else:  # assume prefix + "/missing-housenumbers/"
        if chkl:
            output, relation_name = missing_housenumbers_view_chkl(ctx, relations, request_uri)
            content_type = "application/octet-stream"
            headers.append(("Content-Disposition", 'attachment;filename="' + relation_name + '.txt"'))
        elif request_uri.endswith("robots.txt"):
            output = util.from_bytes(util.get_content(ctx.get_abspath("data/robots.txt")))
        else:  # assume txt
            output = missing_housenumbers_view_txt(ctx, relations, request_uri)
    output_bytes = util.to_bytes(output)
    response_properties = webframe.make_response(content_type, "200 OK", output_bytes, headers)
    filtered_environ = {k: v for k, v in environ.items() if k == "HTTP_ACCEPT_ENCODING"}
    status, headers, output_byte_list = webframe.send_response(filtered_environ, response_properties)
    start_response(status, headers)
    return output_byte_list


HANDLERS = {
    "/streets/": handle_streets,
    "/missing-streets/": handle_missing_streets,
    "/additional-streets/": handle_additional_streets,
    "/additional-housenumbers/": handle_additional_housenumbers,
    "/street-housenumbers/": handle_street_housenumbers,
    "/missing-housenumbers/": handle_missing_housenumbers,
    "/housenumber-stats/": webframe.handle_stats,
}


def get_handler(
    ctx: rust.PyContext,
    request_uri: str
) -> Optional[Callable[[rust.PyContext, rust.PyRelations, str], yattag.Doc]]:
    """Decides request_uri matches what handler."""
    prefix = ctx.get_ini().get_uri_prefix()
    for key, value in HANDLERS.items():
        if request_uri.startswith(prefix + key):
            return value
    return None


def our_application(
        environ: Dict[str, Any],
        start_response: 'StartResponse',
        ctx: rust.PyContext
) -> Tuple[Iterable[bytes], str]:
    """Dispatches the request based on its URI."""
    try:
        language = util.setup_localization([(k, v) for k, v in environ.items() if isinstance(v, str)])

        relations = areas.make_relations(ctx)

        filtered_environ = {k: v for k, v in environ.items() if k == "PATH_INFO"}
        request_uri = webframe.get_request_uri(filtered_environ, ctx, relations)
        _, _, ext = request_uri.partition('.')

        if ext in ("txt", "chkl"):
            return our_application_txt(environ, start_response, ctx, relations, request_uri), str()

        if not (request_uri == "/" or request_uri.startswith(ctx.get_ini().get_uri_prefix())):
            doc = webframe.handle_404()
            response = webframe.make_response("text/html", "404 Not Found", util.to_bytes(doc.get_value()), [])
            filtered_environ = {k: v for k, v in environ.items() if k == "HTTP_ACCEPT_ENCODING"}
            status, headers, output_byte_list = webframe.send_response(filtered_environ, response)
            start_response(status, headers)
            return output_byte_list, str()

        if request_uri.startswith(ctx.get_ini().get_uri_prefix() + "/static/") or \
                request_uri.endswith("favicon.ico") or request_uri.endswith("favicon.svg"):
            output, content_type, headers = webframe.handle_static(ctx, request_uri)
            filtered_environ = {k: v for k, v in environ.items() if k == "HTTP_ACCEPT_ENCODING"}
            status, headers, output_byte_list = webframe.send_response(filtered_environ,
                                                                       webframe.make_response(content_type, "200 OK", output, headers))
            start_response(status, headers)
            return output_byte_list, str()

        if ext == "json":
            filtered_environ = {k: v for k, v in environ.items() if k == "HTTP_ACCEPT_ENCODING"}
            status, headers, output_byte_list = wsgi_json.our_application_json(filtered_environ, ctx, relations, request_uri)
            start_response(status, headers)
            return output_byte_list, str()

        doc = yattag.Doc()
        util.write_html_header(doc)
        with doc.tag("html", [("lang", language)]):
            write_html_head(ctx, doc, get_html_title(request_uri))

            with doc.tag("body", []):
                no_such_relation = webframe.check_existing_relation(ctx, relations, request_uri)
                handler = get_handler(ctx, request_uri)
                if no_such_relation.get_value():
                    doc.append_value(no_such_relation.get_value())
                elif handler:
                    doc.append_value(handler(ctx, relations, request_uri).get_value())
                elif request_uri.startswith(ctx.get_ini().get_uri_prefix() + "/webhooks/github"):
                    doc.append_value(webframe.handle_github_webhook(environ["wsgi.input"], ctx).get_value())
                else:
                    doc.append_value(handle_main(request_uri, ctx, relations).get_value())

        err = ctx.get_unit().make_error()
        if err:
            return [], err
        filtered_environ = {k: v for k, v in environ.items() if k == "HTTP_ACCEPT_ENCODING"}
        status, headers, output_byte_list = webframe.send_response(filtered_environ,
                                                                   webframe.make_response("text/html", "200 OK", util.to_bytes(doc.get_value()), []))
        start_response(status, headers)
        return output_byte_list, err
    # pylint: disable=broad-except
    except Exception:  # pragma: no cover
        return [], traceback.format_exc()


def application(
        environ: Dict[str, Any],
        start_response: 'StartResponse',
        ctx: rust.PyContext
) -> Iterable[bytes]:
    """The entry point of this WSGI app."""
    ret, err = our_application(environ, start_response, ctx)
    if err:
        filtered_environ = {k: v for k, v in environ.items() if k in ("HTTP_ACCEPT_ENCODING", "PATH_INFO")}
        status, headers, output_bytes_list = webframe.handle_exception(filtered_environ, err)
        start_response(status, headers)
        return output_bytes_list
    return ret


# vim:set shiftwidth=4 softtabstop=4 expandtab:
