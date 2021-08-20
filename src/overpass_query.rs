/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The overpass_query module allows getting data out of the OSM DB without a full download.

use pyo3::prelude::*;

/// Posts the query string to the overpass API and returns the result string.
pub fn overpass_query(ctx: &crate::context::Context, query: String) -> anyhow::Result<String> {
    let url = ctx.get_ini().get_overpass_uri() + "/api/interpreter";

    ctx.get_network().urlopen(&url, &query)
}

#[pyfunction]
pub fn py_overpass_query(py: Python<'_>, ctx: PyObject, query: String) -> PyResult<String> {
    let ctx: PyRefMut<'_, crate::context::PyContext> = ctx.extract(py)?;
    match overpass_query(&ctx.context, query) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "overpass_query() failed: {}",
            err.to_string()
        ))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_overpass_query, module)?)?;
    Ok(())
}
