/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The accept_language module parses an Accept-Language HTTP header.

use pyo3::prelude::*;

#[pyfunction]
pub fn py_parse(raw_languages: &str) -> Vec<String> {
    accept_language::parse(raw_languages)
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_parse, module)?)?;
    Ok(())
}
