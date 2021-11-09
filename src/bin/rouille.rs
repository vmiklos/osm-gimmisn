/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Provides the glue layer between the Rouille app server and the wsgi module.

use rust::wsgi;
use std::collections::HashMap;
use std::io::Read;

/// Wraps wsgi::application() to an app for rouille.
fn app(request: &rouille::Request) -> anyhow::Result<rouille::Response> {
    let ctx = rust::context::Context::new("")?;
    let mut request_headers: HashMap<String, String> = HashMap::new();
    for (key, value) in request.headers() {
        request_headers.insert(key.to_string(), value.to_string());
    }
    // TODO work with the rouille::Request in wsgi::application() instead of this mapping.
    request_headers.insert("PATH_INFO".to_string(), request.url());
    let mut request_data = Vec::new();
    if let Some(mut reader) = request.data() {
        reader.read_to_end(&mut request_data)?;
    }
    // TODO return a numeric status in the first place.
    let (status, headers, data) = wsgi::application(&request_headers, &request_data, &ctx)?;
    let mut tokens = status.split(' ');
    let status_code: u16 = tokens.next().unwrap().parse()?;
    let headers: Vec<(
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    )> = headers
        .iter()
        .map(|(key, value)| {
            (
                std::borrow::Cow::Owned(key.into()),
                std::borrow::Cow::Owned(value.into()),
            )
        })
        .collect();
    Ok(rouille::Response {
        status_code,
        headers,
        data: rouille::ResponseBody::from_data(data),
        upgrade: None,
    })
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
    let ctx = rust::context::Context::new("")?;
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
