/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Provides the 'cron' cmdline tool.

/// Sets up logging.
pub fn setup_logging(ctx: &osm_gimmisn::context::Context) -> anyhow::Result<()> {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format("%Y-%m-%d %H:%M:%S".into())
        .set_time_to_local(true)
        .build();
    let logpath = ctx.get_abspath("workdir/cron.log");
    let file = std::fs::File::create(logpath)?;
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            simplelog::LevelFilter::Info,
            config.clone(),
            simplelog::TerminalMode::Stdout,
            simplelog::ColorChoice::Never,
        ),
        simplelog::WriteLogger::new(simplelog::LevelFilter::Info, config, file),
    ])?;

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ctx = osm_gimmisn::context::Context::new("").unwrap();
    std::process::exit(osm_gimmisn::cron::main(&args, &mut std::io::stdout(), &ctx))
}
