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
    let prefix = ctx.get_ini().get_uri_prefix();
    writeln!(
        stream,
        "Starting the server at <http://127.0.0.1:{}{}/>.",
        port, prefix
    )
    .unwrap();
    osm_gimmisn::util::get_tz_offset();
    rouille::start_server_with_pool(format!("127.0.0.1:{}", port), None, move |request| {
        rouille_app(request)
    });
}

/// Sets up logging.
fn cron_setup_logging(ctx: &osm_gimmisn::context::Context) {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(simplelog::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .set_time_offset_to_local()
        .unwrap()
        .build();
    let logpath = ctx.get_abspath("workdir/cron.log");
    let file = std::fs::File::create(logpath).expect("failed to create cron.log");
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            simplelog::LevelFilter::Info,
            config.clone(),
            simplelog::TerminalMode::Stdout,
            simplelog::ColorChoice::Never,
        ),
        simplelog::WriteLogger::new(simplelog::LevelFilter::Info, config, file),
    ])
    .expect("failed to init the combined logger");
}

fn cron_main(args: &[String], stream: &mut dyn Write, ctx: &osm_gimmisn::context::Context) -> i32 {
    cron_setup_logging(ctx);
    osm_gimmisn::cron::main(args, stream, ctx)
}

lazy_static::lazy_static! {
    static ref HANDLERS: HashMap<String, Handler> = {
        let mut ret: HashMap<String, Handler> = HashMap::new();
        ret.insert("cache-yamls".into(), osm_gimmisn::cache_yamls::main);
        ret.insert("cron".into(), cron_main);
        ret.insert("missing-housenumbers".into(), osm_gimmisn::missing_housenumbers::main);
        ret.insert("parse-access-log".into(), osm_gimmisn::parse_access_log::main);
        ret.insert("rouille".into(), rouille_main);
        ret.insert("sync-ref".into(), osm_gimmisn::sync_ref::main);
        ret.insert("validator".into(), osm_gimmisn::validator::main);
        ret
    };
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    let cache_yamls =
        clap::Command::new("cache-yamls").about("Caches YAML files from the data/ directory");
    let cron = clap::Command::new("cron").about("Performs nightly tasks");
    let missing_housenumbers = clap::Command::new("missing-housenumbers")
        .about("Compares reference house numbers with OSM ones and shows the diff");
    let parse_access_log = clap::Command::new("parse-access-log")
        .about("Parses the Apache access log of osm-gimmisn for 1 month");
    let rouille = clap::Command::new("rouille").about("Starts the web interface");
    let sync_ref = clap::Command::new("sync-ref")
        .about("Synchronizes the reference data from a public instance to a local dev instance");
    let validator = clap::Command::new("validator").about("Validates yaml files under data/");
    let subcommands = vec![
        cache_yamls,
        cron,
        missing_housenumbers,
        parse_access_log,
        rouille,
        sync_ref,
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
