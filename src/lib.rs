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
mod area_files;
mod areas;
mod cache;
pub mod cache_yamls;
pub mod context;
pub mod cron;
mod i18n;
pub mod missing_housenumbers;
mod overpass_query;
pub mod parse_access_log;
mod ranges;
mod stats;
mod util;
pub mod validator;
mod webframe;
pub mod wsgi;
mod wsgi_additional;
mod wsgi_json;
mod yattag;

#[pymodule]
fn rust(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    areas::register_python_symbols(m)?;
    context::register_python_symbols(m)?;
    parse_access_log::register_python_symbols(m)?;
    Ok(())
}
