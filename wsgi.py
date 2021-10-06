#!/usr/bin/env python3
#
# Copyright 2019 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The wsgi module contains functionality specific to the web interface."""

from typing import Callable
from typing import Dict
from typing import List
from typing import Optional
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


def handle_streets(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/streets/ormezo/view-query."""
    return rust.py_handle_streets(ctx, relations, request_uri)


def handle_street_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query."""
    return rust.py_handle_street_housenumbers(ctx, relations, request_uri)


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


def handle_missing_housenumbers(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-[result|query]."""
    return rust.py_handle_missing_housenumbers(ctx, relations, request_uri)


def handle_missing_streets(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-[result|query]."""
    return rust.py_handle_missing_streets(ctx, relations, request_uri)


def handle_additional_streets(ctx: rust.PyContext, relations: rust.PyRelations, request_uri: str) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-[result|query]."""
    return rust.py_handle_additional_streets(ctx, relations, request_uri)


def handle_additional_housenumbers(
    ctx: rust.PyContext,
    relations: rust.PyRelations,
    request_uri: str
) -> yattag.Doc:
    """Expected request_uri: e.g. /osm/additional-housenumbers/ujbuda/view-[result|query]."""
    return rust.py_handle_additional_housenumbers(ctx, relations, request_uri)


def handle_main_housenr_additional_count(ctx: rust.PyContext, relation: rust.PyRelation) -> yattag.Doc:
    """Handles the housenumber additional count part of the main page."""
    return rust.py_handle_main_housenr_additional_count(ctx, relation)


def handle_main(request_uri: str, ctx: rust.PyContext, relations: rust.PyRelations) -> yattag.Doc:
    """Handles the main wsgi page.

    Also handles /osm/filter-for/* which filters for a condition."""
    return rust.py_handle_main(request_uri, ctx, relations)


def get_html_title(request_uri: str) -> str:
    """Determines the HTML title for a given function and relation name."""
    return rust.py_get_html_title(request_uri)


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
        ctx: rust.PyContext,
        relations: rust.PyRelations,
        request_uri: str
) -> rust.PyResponse:
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
    return webframe.make_response(content_type, "200 OK", output_bytes, headers)


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
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: rust.PyContext
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """Dispatches the request based on its URI."""
    language = util.setup_localization(list(request_headers.items()))

    relations = areas.make_relations(ctx)

    request_uri = webframe.get_request_uri(request_headers, ctx, relations)
    _, _, ext = request_uri.partition('.')

    if ext in ("txt", "chkl"):
        return webframe.compress_response(request_headers, our_application_txt(ctx, relations, request_uri))

    if not (request_uri == "/" or request_uri.startswith(ctx.get_ini().get_uri_prefix())):
        doc = webframe.handle_404()
        response = webframe.make_response("text/html", "404 Not Found", util.to_bytes(doc.get_value()), [])
        return webframe.compress_response(request_headers, response)

    if request_uri.startswith(ctx.get_ini().get_uri_prefix() + "/static/") or \
            request_uri.endswith("favicon.ico") or request_uri.endswith("favicon.svg"):
        output, content_type, headers = webframe.handle_static(ctx, request_uri)
        response = webframe.make_response(content_type, "200 OK", output, headers)
        return webframe.compress_response(request_headers, response)

    if ext == "json":
        return wsgi_json.our_application_json(request_headers, ctx, relations, request_uri)

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
                doc.append_value(webframe.handle_github_webhook(request_data, ctx).get_value())
            else:
                doc.append_value(handle_main(request_uri, ctx, relations).get_value())

    err = ctx.get_unit().make_error()
    if err:
        raise OSError(err)
    response = webframe.make_response("text/html", "200 OK", util.to_bytes(doc.get_value()), [])
    return webframe.compress_response(request_headers, response)


def application(
        request_headers: Dict[str, str],
        request_data: bytes,
        ctx: rust.PyContext
) -> Tuple[str, List[Tuple[str, str]], List[bytes]]:
    """The entry point of this WSGI app."""
    try:
        return our_application(request_headers, request_data, ctx)
    # pylint: disable=broad-except
    except Exception:  # pragma: no cover
        return webframe.handle_exception(request_headers, traceback.format_exc())


# vim:set shiftwidth=4 softtabstop=4 expandtab:
