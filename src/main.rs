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

/// This handles the update of data/wsgi.ini.template, tools/sync-ref.sh has to be still invoked
/// after this.
/// TODO merge tools/sync-ref.sh into this function.
fn sync_ref_main(
    args: &[String],
    _stream: &mut dyn Write,
    _ctx: &osm_gimmisn::context::Context,
) -> i32 {
    // TODO move this to a library and write tests.
    // Download HTML.
    use isahc::config::Configurable as _;
    use isahc::ReadResponseExt as _;
    use isahc::RequestExt as _;

    let mut args_iter = args.iter();
    let _self = args_iter.next();
    let url = match args_iter.next() {
        Some(s) => s,
        None => {
            println!("usage: osm-gimmisn sync-ref https://www.example.com/osm/data/");
            return 1;
        }
    };
    // let html = std::fs::read_to_string("osm-data.html").unwrap();
    let mut buf = isahc::Request::get(url)
        .redirect_policy(isahc::config::RedirectPolicy::Limit(1))
        .body(())
        .unwrap()
        .send()
        .unwrap();
    let html: String = buf.text().unwrap();

    // Parse the HTML.
    let dom = html_parser::Dom::parse(&html).unwrap();
    let mut dom_iter = dom.children.iter();
    let mut root = dom_iter.next().unwrap();
    if root.text().is_some() {
        // Skip a first-line comment before the real root.
        root = dom_iter.next().unwrap();
    }
    let root = root.into_iter();

    // The format is type_date.tsv, figure out the latest date for each type.
    let mut files: HashMap<String, u64> = HashMap::new();
    for node in root {
        if node.element().is_none() {
            continue;
        }

        let element = node.element().unwrap();
        if element.name != "a" {
            continue;
        }

        let href_value: Option<String> = element.attributes.get("href").unwrap().clone();
        let mut href = href_value.unwrap();
        if !href.ends_with(".tsv") {
            continue;
        }

        href = href.strip_suffix(".tsv").unwrap().into();
        let tokens: Vec<&str> = href.split('_').collect();
        let file: String = tokens[0..tokens.len() - 1].join("_");
        let href_date: u64 = tokens[tokens.len() - 1].parse().unwrap();
        files
            .entry(file)
            .and_modify(|date| *date = std::cmp::max(*date, href_date))
            .or_insert(href_date);
    }

    // Generate config.
    let mut config: Vec<String> = Vec::new();
    config.push("[wsgi]".into());
    config.push(format!(
        "reference_housenumbers = refdir/hazszamok_{}.tsv refdir/hazszamok_kieg_{}.tsv",
        files["hazszamok"], files["hazszamok_kieg"]
    ));
    config.push(format!(
        "reference_street = refdir/utcak_{}.tsv",
        files["utcak"]
    ));
    config.push(format!(
        "reference_citycounts = refdir/varosok_count_{}.tsv",
        files["varosok_count"]
    ));
    config.push(format!(
        "reference_zipcounts = refdir/irsz_count_{}.tsv",
        files["irsz_count"]
    ));

    // TODO get rid of these
    config.push("overpass_uri = https://z.overpass-api.de".into());
    config.push("cron_update_inactive = False".into());
    config.push(String::new());

    // Write config.
    let config_file = "data/wsgi.ini.template";
    std::fs::write(config_file, config.join("\n")).unwrap();
    let max = files.iter().map(|(_k, v)| v).max().unwrap();
    println!(
        "Now you can run: git commit -m 'Update reference to {}'",
        max
    );
    0
}

lazy_static::lazy_static! {
    static ref HANDLERS: HashMap<String, Handler> = {
        let mut ret: HashMap<String, Handler> = HashMap::new();
        ret.insert("cache_yamls".into(), osm_gimmisn::cache_yamls::main);
        ret.insert("cron".into(), cron_main);
        ret.insert("missing_housenumbers".into(), osm_gimmisn::missing_housenumbers::main);
        ret.insert("parse_access_log".into(), osm_gimmisn::parse_access_log::main);
        ret.insert("rouille".into(), rouille_main);
        ret.insert("sync-ref".into(), sync_ref_main);
        ret.insert("validator".into(), osm_gimmisn::validator::main);
        ret
    };
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    let cache_yamls =
        clap::Command::new("cache_yamls").about("Caches YAML files from the data/ directory");
    let cron = clap::Command::new("cron").about("Performs nightly tasks");
    let missing_housenumbers = clap::Command::new("missing_housenumbers")
        .about("Compares reference house numbers with OSM ones and shows the diff");
    let parse_access_log = clap::Command::new("parse_access_log")
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
