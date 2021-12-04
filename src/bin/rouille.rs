/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Provides the glue layer between the Rouille app server and the wsgi module.

use osm_gimmisn::wsgi;

/// Wraps wsgi::application() to an app for rouille.
fn app(request: &rouille::Request) -> anyhow::Result<rouille::Response> {
    let ctx = osm_gimmisn::context::Context::new("")?;
    wsgi::application(request, &ctx)
}

/// Commandline interface to this module.
///
/// Once this is started, a reverse proxy on top of this can add SSL support. For example, Apache
/// needs something like:
///
/// ProxyPreserveHost On
/// ProxyPass / http://127.0.0.1:8000/
/// ProxyPassReverse / http://127.0.0.1:8000/
fn main() -> anyhow::Result<()> {
    let ctx = osm_gimmisn::context::Context::new("")?;
    let port = ctx.get_ini().get_tcp_port()?;
    let prefix = ctx.get_ini().get_uri_prefix()?;
    // TODO no matching stop message.
    println!(
        "Starting the server at <http://127.0.0.1:{}{}/>.",
        port, prefix
    );
    rouille::start_server_with_pool(format!("127.0.0.1:{}", port), None, move |request| {
        app(request).unwrap()
    });
}
