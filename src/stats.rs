/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The stats module creates statistics about missing / non-missing house numbers.

use crate::context;
use anyhow::Context;
use pyo3::prelude::*;
use std::io::BufRead;

/// Generates stats for a global progressbar.
fn handle_progress(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let mut ret = serde_json::json!({});
    let num_ref: f64 = std::fs::read_to_string(format!("{}/ref.count", src_root))?
        .trim()
        .parse()
        .context("failed to parse ref.count")?;
    let today = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let mut num_osm = 0_f64;
    let count_path = format!("{}/{}.count", src_root, today);
    if std::path::Path::new(&count_path).exists() {
        num_osm = std::fs::read_to_string(&count_path)?
            .trim()
            .parse()
            .context("failed to parse today.count")?;
    }
    // Round to 2 digits.
    let percentage = ((num_osm * 100.0 / num_ref) * 100.0).round() / 100.0;
    let ret_obj = ret.as_object_mut().unwrap();
    ret_obj.insert("date".into(), serde_json::json!(today));
    ret_obj.insert("percentage".into(), serde_json::json!(percentage));
    ret_obj.insert("reference".into(), serde_json::json!(num_ref));
    ret_obj.insert("osm".into(), serde_json::json!(num_osm));
    j.as_object_mut().unwrap().insert("progress".into(), ret);

    Ok(())
}

#[pyfunction]
fn py_handle_progress(ctx: context::PyContext, src_root: &str, j: &str) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_progress(&ctx.context, src_root, &mut j).context("handle_progress() failed") {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Generates stats for top users.
fn handle_topusers(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let today = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let mut ret: Vec<(String, String)> = Vec::new();
    let topusers_path = format!("{}/{}.topusers", src_root, today);
    if std::path::Path::new(&topusers_path).exists() {
        let stream = std::io::BufReader::new(std::fs::File::open(topusers_path)?);
        for line in stream.lines() {
            let line = line?.trim().to_string();
            let mut tokens = line.split(' ');
            let count = tokens.next().unwrap();
            let user = tokens.next().unwrap();
            ret.push((user.into(), count.into()));
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("topusers".into(), serde_json::to_value(&ret)?);

    Ok(())
}

#[pyfunction]
fn py_handle_topusers(ctx: context::PyContext, src_root: &str, j: &str) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_topusers(&ctx.context, src_root, &mut j).context("handle_topusers() failed") {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Registers Python wrappers of Rust structs into the Python module.
pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_handle_progress, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_topusers, module)?)?;
    Ok(())
}
