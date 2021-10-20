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
use crate::util;
use anyhow::Context;
use pyo3::prelude::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufRead;
use std::ops::DerefMut;

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

/// Writes a daily .count file.
fn write_count_path(
    ctx: &context::Context,
    count_path: &str,
    house_numbers: &HashSet<String>,
) -> anyhow::Result<()> {
    let stream = ctx.get_file_system().open_write(count_path)?;
    let mut guard = stream.lock().unwrap();
    let house_numbers_len = house_numbers.len().to_string();
    Ok(guard.write_all(house_numbers_len.as_bytes())?)
}

/// Writes a daily .citycount file.
fn write_city_count_path(
    ctx: &context::Context,
    city_count_path: &str,
    cities: &HashMap<String, HashSet<String>>,
) -> anyhow::Result<()> {
    let stream = ctx.get_file_system().open_write(city_count_path)?;
    let mut guard = stream.lock().unwrap();
    let mut cities: Vec<_> = cities.iter().map(|(key, value)| (key, value)).collect();
    cities.sort_by_key(|(key, _value)| util::get_sort_key(key).unwrap());
    cities.dedup();
    // Locale-aware sort, by key.
    for (key, value) in cities {
        let line = format!("{}\t{}\n", key, value.len());
        guard.write_all(line.as_bytes())?;
    }

    Ok(())
}

/// Counts the # of all house numbers as of today.
fn update_stats_count(ctx: &context::Context, today: &str) -> anyhow::Result<()> {
    let statedir = ctx.get_abspath("workdir/stats")?;
    let csv_path = format!("{}/{}.csv", statedir, today);
    if !ctx.get_file_system().path_exists(&csv_path) {
        return Ok(());
    }
    let count_path = format!("{}/{}.count", statedir, today);
    let city_count_path = format!("{}/{}.citycount", statedir, today);
    let mut house_numbers: HashSet<String> = HashSet::new();
    let mut cities: HashMap<String, HashSet<String>> = HashMap::new();
    let mut first = true;
    let valid_settlements = util::get_valid_settlements(ctx)?;
    let stream = ctx.get_file_system().open_read(&csv_path)?;
    let mut guard = stream.lock().unwrap();
    let reader = std::io::BufReader::new(guard.deref_mut());
    for line in reader.lines() {
        if first {
            // Ignore the oneliner header.
            first = false;
            continue;
        }
        let line = line?.to_string();
        // postcode, city name, street name, house number, user
        let cells: Vec<String> = line.split('\t').map(|i| i.into()).collect();
        // Ignore last column, which is the user who touched the object last.
        house_numbers.insert(cells[0..4].join("\t"));
        let city_key = util::get_city_key(&cells[0], &cells[1], &valid_settlements)?;
        let city_value = cells[2..4].join("\t");
        let entry = cities.entry(city_key).or_insert_with(HashSet::new);
        entry.insert(city_value);
    }
    write_count_path(ctx, &count_path, &house_numbers)?;
    write_city_count_path(ctx, &city_count_path, &cities)
}

#[pyfunction]
fn py_update_stats_count(ctx: context::PyContext, today: &str) -> PyResult<()> {
    match update_stats_count(&ctx.context, today).context("update_stats_count() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Counts the top housenumber editors as of today.
fn update_stats_topusers(ctx: &context::Context, today: &str) -> anyhow::Result<()> {
    let statedir = ctx.get_abspath("workdir/stats")?;
    let csv_path = format!("{}/{}.csv", statedir, today);
    if !ctx.get_file_system().path_exists(&csv_path) {
        return Ok(());
    }
    let topusers_path = format!("{}/{}.topusers", statedir, today);
    let usercount_path = format!("{}/{}.usercount", statedir, today);
    let mut users: HashMap<String, u64> = HashMap::new();
    {
        let stream = ctx.get_file_system().open_read(&csv_path)?;
        let mut guard = stream.lock().unwrap();
        let reader = std::io::BufReader::new(guard.deref_mut());
        for line in reader.lines() {
            let line = line?.to_string();
            let cells: Vec<String> = line.split('\t').map(|i| i.into()).collect();
            // Only care about the last column.
            let user = cells[cells.len() - 1].clone();
            let entry = users.entry(user).or_insert(0);
            (*entry) += 1;
        }
    }
    {
        let stream = ctx.get_file_system().open_write(&topusers_path)?;
        let mut guard = stream.lock().unwrap();
        let mut users: Vec<_> = users.iter().map(|(key, value)| (key, value)).collect();
        users.sort_by_key(|i| Reverse(i.1));
        users.dedup();
        users = users[0..std::cmp::min(20, users.len())].to_vec();
        for user in users {
            let line = format!("{} {}\n", user.1, user.0);
            guard.write_all(line.as_bytes())?;
        }
    }

    let stream = ctx.get_file_system().open_write(&usercount_path)?;
    let mut guard = stream.lock().unwrap();
    let line = format!("{}\n", users.len());
    Ok(guard.write_all(line.as_bytes())?)
}

#[pyfunction]
fn py_update_stats_topusers(ctx: context::PyContext, today: &str) -> PyResult<()> {
    match update_stats_topusers(&ctx.context, today).context("update_stats_topusers() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Performs the update of workdir/stats/ref.count.
fn update_stats_refcount(ctx: &context::Context, state_dir: &str) -> anyhow::Result<()> {
    let mut count = 0;
    {
        let stream = ctx
            .get_file_system()
            .open_read(&ctx.get_ini().get_reference_citycounts_path()?)?;
        let mut guard = stream.lock().unwrap();
        let mut read = guard.deref_mut();
        let mut csv_read = util::CsvRead::new(&mut read);
        let mut first = true;
        for result in csv_read.records() {
            let row = result?;
            if first {
                first = false;
                continue;
            }

            count += row[1].parse::<i32>()?;
        }
    }

    let stream = ctx
        .get_file_system()
        .open_write(&format!("{}/ref.count", state_dir))?;
    let mut guard = stream.lock().unwrap();
    Ok(guard.write_all(format!("{}\n", count).as_bytes())?)
}

#[pyfunction]
fn py_update_stats_refcount(ctx: context::PyContext, state_dir: &str) -> PyResult<()> {
    match update_stats_refcount(&ctx.context, state_dir).context("update_stats_refcount() failed") {
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
    module.add_function(pyo3::wrap_pyfunction!(py_update_stats_count, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_update_stats_topusers, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_update_stats_refcount, module)?)?;
    Ok(())
}
