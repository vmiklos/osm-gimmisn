/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The overpass_query module allows getting data out of the OSM DB without a full download.

use crate::context;
use pyo3::prelude::*;

/// Posts the query string to the overpass API and returns the result string.
pub fn overpass_query(ctx: &context::Context, query: String) -> anyhow::Result<String> {
    let url = ctx.get_ini().get_overpass_uri() + "/api/interpreter";

    ctx.get_network().urlopen(&url, &query)
}

#[pyfunction]
pub fn py_overpass_query(py: Python<'_>, ctx: PyObject, query: String) -> PyResult<String> {
    let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(py)?;
    match overpass_query(&ctx.context, query) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "overpass_query() failed: {}",
            err.to_string()
        ))),
    }
}

/// Checks if we need to sleep before executing an overpass query.
pub fn overpass_query_need_sleep(ctx: &context::Context) -> i32 {
    let url = ctx.get_ini().get_overpass_uri() + "/api/status";
    let status = match ctx.get_network().urlopen(&url, "") {
        Ok(value) => value,
        _ => {
            return 0;
        }
    };
    let mut sleep = 0;
    let mut available = false;
    for line in status.lines() {
        if line.starts_with("Slot available after:") {
            let re = regex::Regex::new(r".*in (-?\d+) seconds.*").unwrap();
            for cap in re.captures_iter(line) {
                sleep = match cap[1].parse::<i32>() {
                    Ok(value) => value,
                    _ => {
                        return 0;
                    }
                };
                // Wait one more second just to be safe.
                sleep += 1;
                if sleep <= 0 {
                    sleep = 1;
                }
            }
            break;
        }
        if line.contains("available now") {
            available = true;
        }
    }
    if available {
        return 0;
    }
    sleep
}

#[pyfunction]
pub fn py_overpass_query_need_sleep(py: Python<'_>, ctx: PyObject) -> PyResult<i32> {
    let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(py)?;
    Ok(overpass_query_need_sleep(&ctx.context))
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_overpass_query, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_overpass_query_need_sleep,
        module
    )?)?;
    Ok(())
}
