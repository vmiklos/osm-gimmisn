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
use crate::cache;
use crate::context;
use crate::i18n;
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

/// Update the OSM housenumber list of all relations.
fn update_osm_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    update: bool,
) -> anyhow::Result<()> {
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update
            && std::path::Path::new(&relation.get_files().get_osm_housenumbers_path()?).exists()
        {
            continue;
        }
        log::info!("update_osm_housenumbers: start: {}", relation_name);
        let mut retry = 0;
        while should_retry(retry) {
            if retry > 0 {
                log::info!("update_osm_housenumbers: try #{}", retry);
            }
            retry += 1;
            overpass_sleep(ctx);
            let query = relation.get_osm_housenumbers_query()?;
            let buf = match overpass_query::overpass_query(ctx, query) {
                Ok(value) => value,
                Err(err) => {
                    log::info!("update_osm_housenumbers: http error: {:?}", err);
                    continue;
                }
            };
            if relation.get_files().write_osm_housenumbers(ctx, &buf)? == 0 {
                log::info!("update_osm_housenumbers: short write");
                continue;
            }
            break;
        }
        log::info!("update_osm_housenumbers: end: {}", relation_name);
    }

    Ok(())
}

#[pyfunction]
fn py_update_osm_housenumbers(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    update: bool,
) -> PyResult<()> {
    match update_osm_housenumbers(&ctx.context, &mut relations.relations, update)
        .context("update_osm_housenumbers() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Update the reference housenumber list of all relations.
fn update_ref_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    update: bool,
) -> anyhow::Result<()> {
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update
            && std::path::Path::new(&relation.get_files().get_ref_housenumbers_path()?).exists()
        {
            continue;
        }
        let references = ctx.get_ini().get_reference_housenumber_paths()?;
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "only" {
            continue;
        }

        log::info!("update_ref_housenumbers: start: {}", relation_name);
        if let Err(err) = relation.write_ref_housenumbers(&references) {
            log::info!("update_osm_housenumbers: failed: {:?}", err);
            continue;
        }
        log::info!("update_ref_housenumbers: end: {}", relation_name);
    }

    Ok(())
}

#[pyfunction]
fn py_update_ref_housenumbers(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    update: bool,
) -> PyResult<()> {
    match update_ref_housenumbers(&ctx.context, &mut relations.relations, update)
        .context("update_ref_housenumbers() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Update the reference street list of all relations.
fn update_ref_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    update: bool,
) -> anyhow::Result<()> {
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update && std::path::Path::new(&relation.get_files().get_ref_streets_path()?).exists() {
            continue;
        }
        let reference = ctx.get_ini().get_reference_street_path()?;
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "no" {
            continue;
        }

        log::info!("update_ref_streets: start: {}", relation_name);
        relation.write_ref_streets(&reference)?;
        log::info!("update_ref_streets: end: {}", relation_name);
    }

    Ok(())
}

#[pyfunction]
fn py_update_ref_streets(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    update: bool,
) -> PyResult<()> {
    match update_ref_streets(&ctx.context, &mut relations.relations, update)
        .context("update_ref_streets() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Update the relation's house number coverage stats.
fn update_missing_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    update: bool,
) -> anyhow::Result<()> {
    log::info!("update_missing_housenumbers: start");
    for relation_name in relations.get_active_names()? {
        let mut relation = relations.get_relation(&relation_name)?;
        if !update
            && std::path::Path::new(&relation.get_files().get_housenumbers_percent_path()?).exists()
        {
            continue;
        }
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "only" {
            continue;
        }

        let orig_language = i18n::get_language();
        relation.write_missing_housenumbers()?;
        for language in ["en", "hu"] {
            i18n::set_language(language)?;
            cache::get_missing_housenumbers_html(ctx, &mut relation)?;
        }
        i18n::set_language(&orig_language)?;
        cache::get_missing_housenumbers_txt(ctx, &mut relation)?;
    }
    log::info!("update_missing_housenumbers: end");

    Ok(())
}

#[pyfunction]
fn py_update_missing_housenumbers(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    update: bool,
) -> PyResult<()> {
    match update_missing_housenumbers(&ctx.context, &mut relations.relations, update)
        .context("update_missing_housenumbers() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Update the relation's street coverage stats.
fn update_missing_streets(relations: &mut areas::Relations, update: bool) -> anyhow::Result<()> {
    log::info!("update_missing_streets: start");
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update
            && std::path::Path::new(&relation.get_files().get_streets_percent_path()?).exists()
        {
            continue;
        }
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "no" {
            continue;
        }

        relation.write_missing_streets()?;
    }
    log::info!("update_missing_streets: end");

    Ok(())
}

#[pyfunction]
fn py_update_missing_streets(mut relations: areas::PyRelations, update: bool) -> PyResult<()> {
    match update_missing_streets(&mut relations.relations, update)
        .context("update_missing_streets() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Update the relation's "additional streets" stats.
fn update_additional_streets(relations: &mut areas::Relations, update: bool) -> anyhow::Result<()> {
    log::info!("update_additional_streets: start");
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update
            && std::path::Path::new(&relation.get_files().get_streets_additional_count_path()?)
                .exists()
        {
            continue;
        }
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "no" {
            continue;
        }

        relation.write_additional_streets()?;
    }
    log::info!("update_additional_streets: end");

    Ok(())
}

#[pyfunction]
fn py_update_additional_streets(mut relations: areas::PyRelations, update: bool) -> PyResult<()> {
    match update_additional_streets(&mut relations.relations, update)
        .context("update_additional_streets() failed")
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
    module.add_function(pyo3::wrap_pyfunction!(py_update_osm_housenumbers, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_update_ref_housenumbers, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_update_ref_streets, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_update_missing_housenumbers,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_update_missing_streets, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_update_additional_streets,
        module
    )?)?;
    Ok(())
}
