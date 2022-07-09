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

fn usage() {
    println!("Valid commands:\n");
    for (k, _v) in HANDLERS.iter() {
        println!("{}", k);
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    let command = match args.get(1) {
        Some(value) => value.to_string(),
        None => {
            println!("osm_gimmisn: missing command\n");
            usage();
            std::process::exit(1);
        }
    };
    args.remove(1);
    let handler: &Handler = match HANDLERS.get(&command) {
        Some(value) => value,
        None => {
            println!("osm_gimmisn: invalid command\n");
            usage();
            std::process::exit(1);
        }
    };

    std::process::exit(handler(&args, &mut std::io::stdout(), &ctx))
}
