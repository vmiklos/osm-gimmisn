/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

//! Finds objects missing from the OSM DB.

use pyo3::prelude::*;

mod accept_language;
mod context;
mod i18n;
mod overpass_query;
mod ranges;
mod util;
mod version;
mod yattag;

#[pymodule]
fn rust(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    accept_language::register_python_symbols(&m)?;
    context::register_python_symbols(&m)?;
    i18n::register_python_symbols(&m)?;
    overpass_query::register_python_symbols(&m)?;
    ranges::register_python_symbols(&m)?;
    util::register_python_symbols(&m)?;
    version::register_python_symbols(&m)?;
    yattag::register_python_symbols(&m)?;
    Ok(())
}
