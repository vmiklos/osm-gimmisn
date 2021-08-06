/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The version module allows tracking the last update of the app code.

use git_version::git_version;
use pyo3::prelude::*;

#[pyfunction]
pub fn py_get_version() -> String {
    String::from(git_version!())
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_get_version, module)?)?;
    Ok(())
}
