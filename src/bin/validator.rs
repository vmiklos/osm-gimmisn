/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Provides the 'validator' cmdline tool.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    std::process::exit(osm_gimmisn::validator::main(
        &args,
        &mut std::io::stdout(),
        &ctx,
    ))
}
