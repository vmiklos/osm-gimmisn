/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Provides the 'osm_gimmisn' cmdline tool.

use std::collections::HashMap;
use std::io::Write;

type Handler = fn(&[String], &mut dyn Write, &osm_gimmisn::context::Context) -> i32;

/// Wraps wsgi::application() to an app for rouille.
fn rouille_app(request: &rouille::Request) -> rouille::Response {
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    osm_gimmisn::wsgi::application(request, &ctx)
}

/// Commandline interface to this module.
///
/// Once this is started, a reverse proxy on top of this can add SSL support. For example, Apache
/// needs something like:
///
/// ProxyPreserveHost On
/// ProxyPass / http://127.0.0.1:8000/
/// ProxyPassReverse / http://127.0.0.1:8000/
/// # Default would be 60
/// ProxyTimeout 120
fn rouille_main(_: &[String], stream: &mut dyn Write, ctx: &osm_gimmisn::context::Context) -> i32 {
    let port = ctx.get_ini().get_tcp_port().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix().unwrap();
    writeln!(
        stream,
        "Starting the server at <http://127.0.0.1:{}{}/>.",
        port, prefix
    )
    .unwrap();
    rouille::start_server_with_pool(format!("127.0.0.1:{}", port), None, move |request| {
        rouille_app(request)
    });
}

lazy_static::lazy_static! {
    static ref HANDLERS: HashMap<String, Handler> = {
        let mut ret: HashMap<String, Handler> = HashMap::new();
        ret.insert("cache_yamls".into(), osm_gimmisn::cache_yamls::main);
        ret.insert("missing_housenumbers".into(), osm_gimmisn::missing_housenumbers::main);
        ret.insert("parse_access_log".into(), osm_gimmisn::parse_access_log::main);
        ret.insert("rouille".into(), rouille_main);
        ret.insert("validator".into(), osm_gimmisn::validator::main);
        ret
    };
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    let cache_yamls =
        clap::Command::new("cache_yamls").about("Caches YAML files from the data/ directory");
    let missing_housenumbers = clap::Command::new("missing_housenumbers")
        .about("Compares reference house numbers with OSM ones and shows the diff");
    let parse_access_log = clap::Command::new("parse_access_log")
        .about("Parses the Apache access log of osm-gimmisn for 1 month");
    let rouille = clap::Command::new("rouille").about("Starts the web interface");
    let validator = clap::Command::new("validator").about("Validates yaml files under data/");
    let subcommands = vec![
        cache_yamls,
        missing_housenumbers,
        parse_access_log,
        rouille,
        validator,
    ];
    let app = clap::Command::new("osm-gimmisn").subcommand_required(true);
    let argv: Vec<String> = args.iter().take(2).cloned().collect();
    let matches = app
        .subcommands(subcommands)
        .try_get_matches_from(&argv)
        .unwrap_or_else(|e| e.exit());
    args.remove(1);
    let handler: &Handler = HANDLERS.get(matches.subcommand().unwrap().0).unwrap();

    std::process::exit(handler(&args, &mut std::io::stdout(), &ctx))
}
