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

import wsgi
import config

if TYPE_CHECKING:
    # pylint: disable=no-name-in-module,import-error,unused-import
    from wsgiref.types import StartResponse


def main(conf: config.Config) -> None:
    """
    Commandline interface to this module.

    Once this is started, a reverse proxy on top of this can add SSL support. For example, Apache
    needs something like:

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:8000/
    ProxyPassReverse / http://127.0.0.1:8000/

    While wsgiref is part of stock Python and is ideal for local development, CherryPy supports
    automatic reloading, which is super-handy in production.
    """
    def app(environ: Dict[str, Any], start_response: 'StartResponse') -> Iterable[bytes]:
        return wsgi.application(environ, start_response, conf)
    cherrypy.tree.graft(app, "/")
    cherrypy.server.unsubscribe()
    # This is documented at <https://docs.cherrypy.org/en/latest/advanced.html>, so:
    # pylint: disable=protected-access
    server = cherrypy._cpserver.Server()
    server.socket_host = "127.0.0.1"
    server.socket_port = conf.get_tcp_port()
    server.thread_pool = 8
    server.subscribe()
    cherrypy.engine.start()
    cherrypy.engine.block()


if __name__ == "__main__":
    main(config.Config(""))

# vim:set shiftwidth=4 softtabstop=4 expandtab:
