/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Contains functionality specific to the json part of the web interface.

use crate::areas;
use crate::context;
use crate::overpass_query;
use anyhow::Context;
use pyo3::prelude::*;
use std::collections::HashMap;

/// Expected request_uri: e.g. /osm/streets/ormezo/update-result.json.
fn streets_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations.get_relation(relation_name)?;
    let query = relation.get_osm_streets_query()?;
    let mut ret: HashMap<String, String> = HashMap::new();
    match overpass_query::overpass_query(ctx, query) {
        Ok(buf) => {
            relation.get_files().write_osm_streets(ctx, &buf)?;
            ret.insert("error".into(), "".into())
        }
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

#[pyfunction]
fn py_streets_update_result_json(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<String> {
    match streets_update_result_json(&ctx.context, &mut relations.relations, request_uri)
        .context("streets_update_result_json() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/street-housenumbers/ormezo/update-result.json.
fn street_housenumbers_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations.get_relation(relation_name)?;
    let query = relation.get_osm_housenumbers_query()?;
    let mut ret: HashMap<String, String> = HashMap::new();
    match overpass_query::overpass_query(ctx, query) {
        Ok(buf) => {
            relation.get_files().write_osm_housenumbers(ctx, &buf)?;
            ret.insert("error".into(), "".into())
        }
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

#[pyfunction]
fn py_street_housenumbers_update_result_json(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<String> {
    match street_housenumbers_update_result_json(
        &ctx.context,
        &mut relations.relations,
        request_uri,
    )
    .context("street_housenumbers_update_result_json() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/update-result.json.
fn missing_housenumbers_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let references = ctx.get_ini().get_reference_housenumber_paths()?;
    let relation = relations.get_relation(relation_name)?;
    let mut ret: HashMap<String, String> = HashMap::new();
    match relation.write_ref_housenumbers(&references) {
        Ok(_) => ret.insert("error".into(), "".into()),
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

#[pyfunction]
fn py_missing_housenumbers_update_result_json(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<String> {
    match missing_housenumbers_update_result_json(
        &ctx.context,
        &mut relations.relations,
        request_uri,
    )
    .context("missing_housenumbers_update_result_json() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-streets/ormezo/update-result.json.
fn missing_streets_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let reference = ctx.get_ini().get_reference_street_path()?;
    let relation = relations.get_relation(relation_name)?;
    let mut ret: HashMap<String, String> = HashMap::new();
    match relation.write_ref_streets(&reference) {
        Ok(_) => ret.insert("error".into(), "".into()),
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

#[pyfunction]
fn py_missing_streets_update_result_json(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<String> {
    match missing_streets_update_result_json(&ctx.context, &mut relations.relations, request_uri)
        .context("missing_streets_update_result_json() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(
        py_streets_update_result_json,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_street_housenumbers_update_result_json,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_housenumbers_update_result_json,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_streets_update_result_json,
        module
    )?)?;
    Ok(())
}
