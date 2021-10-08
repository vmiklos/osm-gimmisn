#!/usr/bin/env python3
#
# Copyright 2020 Miklos Vajna. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#

"""The cherry module is the glue layer between the CherryPy app server and the wsgi module."""

from typing import Any
from typing import Dict
from typing import Iterable
from typing import TYPE_CHECKING

import cherrypy  # type: ignore

import rust

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def app(environ: Dict[str, Any], start_response: 'StartResponse') -> Iterable[bytes]:
    """Wraps wsgi.application() to a wsgi app for cherrypy."""
    ctx = rust.PyContext("")
    request_headers: Dict[str, str] = {}
    request_data = bytes()
    for key, value in environ.items():
        if key in ("HTTP_ACCEPT_ENCODING", "HTTP_ACCEPT_LANGUAGE", "PATH_INFO"):
            request_headers[key] = value
        elif key == "wsgi.input":
            request_data = value.read()
    status, headers, data = rust.py_application(request_headers, request_data, ctx)
    start_response(status, headers)
    return [data]


def main() -> None:
    """
    Commandline interface to this module.

    Once this is started, a reverse proxy on top of this can add SSL support. For example, Apache
    needs something like:

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8000/
    ProxyPassReverse / http://127.0.0.1:8000/
    """
    cherrypy.tree.graft(app, "/")
    cherrypy.server.unsubscribe()
    # This is documented at <https://docs.cherrypy.org/en/latest/advanced.html>, so:
    # pylint: disable=protected-access
    server = cherrypy._cpserver.Server()
    ctx = rust.PyContext("")
    server.socket_host = "127.0.0.1"
    server.socket_port = ctx.get_ini().get_tcp_port()
    server.thread_pool = 8
    server.subscribe()
    cherrypy.engine.start()
    cherrypy.engine.block()


if __name__ == "__main__":
    main()

# vim:set shiftwidth=4 softtabstop=4 expandtab:
