/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cron module allows doing nightly tasks.

use crate::areas;
use crate::context;
use crate::overpass_query;
use anyhow::Context;
use pyo3::prelude::*;

/// Sets up logging.
fn setup_logging(ctx: &context::Context) -> anyhow::Result<()> {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format("%Y-%m-%d %H:%M:%S".into())
        .set_time_to_local(true)
        .build();
    let logpath = ctx.get_abspath("workdir/cron.log")?;
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

#[pyfunction]
fn py_setup_logging(ctx: context::PyContext) -> PyResult<()> {
    match setup_logging(&ctx.context).context("setup_logging() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

#[pyfunction]
fn py_info(msg: String) {
    log::info!("{}", msg);
}

#[pyfunction]
fn py_error(msg: String) {
    log::error!("{}", msg);
}

/// Sleeps to respect overpass rate limit.
fn overpass_sleep(ctx: &context::Context) {
    loop {
        let sleep = overpass_query::overpass_query_need_sleep(ctx);
        if sleep == 0 {
            break;
        }
        log::info!("overpass_sleep: waiting for {} seconds", sleep);
        ctx.get_time().sleep(sleep as u64);
    }
}

#[pyfunction]
fn py_overpass_sleep(ctx: context::PyContext) {
    overpass_sleep(&ctx.context)
}

/// Decides if we should retry a query or not.
fn should_retry(retry: i32) -> bool {
    retry < 20
}

/// Update the OSM street list of all relations.
fn update_osm_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    update: bool,
) -> anyhow::Result<()> {
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update && std::path::Path::new(&relation.get_files().get_osm_streets_path()?).exists() {
            continue;
        }
        log::info!("update_osm_streets: start: {}", relation_name);
        let mut retry = 0;
        while should_retry(retry) {
            if retry > 0 {
                log::info!("update_osm_streets: try #{}", retry);
            }
            retry += 1;
            overpass_sleep(ctx);
            let query = relation.get_osm_streets_query()?;
            let buf = match overpass_query::overpass_query(ctx, query) {
                Ok(value) => value,
                Err(err) => {
                    log::info!("update_osm_streets: http error: {:?}", err);
                    continue;
                }
            };
            if relation.get_files().write_osm_streets(ctx, &buf)? == 0 {
                log::info!("update_osm_streets: short write");
                continue;
            }
            break;
        }
        log::info!("update_osm_streets: end: {}", relation_name);
    }

    Ok(())
}

#[pyfunction]
fn py_update_osm_streets(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    update: bool,
) -> PyResult<()> {
    match update_osm_streets(&ctx.context, &mut relations.relations, update)
        .context("update_osm_streets() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_setup_logging, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_info, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_error, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_overpass_sleep, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_update_osm_streets, module)?)?;
    Ok(())
}
