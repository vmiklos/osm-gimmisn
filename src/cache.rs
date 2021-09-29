/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cache module accelerates some functions of the areas module.

use crate::areas;
use crate::context;
use anyhow::Context;
use pyo3::prelude::*;

/// Decides if we have an up to date cache entry or not.
fn is_cache_outdated(
    ctx: &context::Context,
    cache_path: &str,
    dependencies: &[String],
) -> anyhow::Result<bool> {
    if !ctx.get_file_system().path_exists(cache_path) {
        return Ok(false);
    }

    let cache_mtime = ctx.get_file_system().getmtime(cache_path)?;

    for dependency in dependencies {
        if ctx.get_file_system().path_exists(dependency)
            && ctx.get_file_system().getmtime(dependency)? > cache_mtime
        {
            return Ok(false);
        }
    }

    Ok(true)
}

#[pyfunction]
fn py_is_cache_outdated(
    ctx: context::PyContext,
    cache_path: &str,
    dependencies: Vec<String>,
) -> PyResult<bool> {
    match is_cache_outdated(&ctx.context, cache_path, &dependencies)
        .context("is_cache_outdated() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Decides if we have an up to date HTML cache entry or not.
fn is_missing_housenumbers_html_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path()?;
    let datadir = ctx.get_abspath("data")?;
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path()?,
        relation.get_files().get_osm_housenumbers_path()?,
        relation.get_files().get_ref_housenumbers_path()?,
        relation_path,
    ];
    is_cache_outdated(ctx, &cache_path, &dependencies)
}

#[pyfunction]
fn py_is_missing_housenumbers_html_cached(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<bool> {
    match is_missing_housenumbers_html_cached(&ctx.context, &relation.relation)
        .context("is_missing_housenumbers_html_cached() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Decides if we have an up to date HTML cache entry for additional house numbers or not.
fn is_additional_housenumbers_html_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation
        .get_files()
        .get_additional_housenumbers_htmlcache_path()?;
    let datadir = ctx.get_abspath("data")?;
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path()?,
        relation.get_files().get_osm_housenumbers_path()?,
        relation.get_files().get_ref_housenumbers_path()?,
        relation_path,
    ];
    is_cache_outdated(ctx, &cache_path, &dependencies)
}

#[pyfunction]
fn py_is_additional_housenumbers_html_cached(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<bool> {
    match is_additional_housenumbers_html_cached(&ctx.context, &relation.relation)
        .context("is_additional_housenumbers_html_cached() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_is_cache_outdated, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_is_missing_housenumbers_html_cached,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_is_additional_housenumbers_html_cached,
        module
    )?)?;
    Ok(())
}
