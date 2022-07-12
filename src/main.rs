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

lazy_static::lazy_static! {
    static ref HANDLERS: HashMap<String, Handler> = {
        let mut ret: HashMap<String, Handler> = HashMap::new();
        ret.insert("cache_yamls".into(), osm_gimmisn::cache_yamls::main);
        ret.insert("missing_housenumbers".into(), osm_gimmisn::missing_housenumbers::main);
        ret.insert("parse_access_log".into(), osm_gimmisn::parse_access_log::main);
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
    let validator = clap::Command::new("validator")
        .about("The validator module validates yaml files under data/");
    let subcommands = vec![
        cache_yamls,
        missing_housenumbers,
        parse_access_log,
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
