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
mod ranges;
mod stats;
mod util;
mod webframe;
pub mod wsgi;
mod wsgi_additional;
mod wsgi_json;
mod yattag;

#[pymodule]
fn rust(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    area_files::register_python_symbols(m)?;
    areas::register_python_symbols(m)?;
    cache::register_python_symbols(m)?;
    cache_yamls::register_python_symbols(m)?;
    context::register_python_symbols(m)?;
    cron::register_python_symbols(m)?;
    i18n::register_python_symbols(m)?;
    missing_housenumbers::register_python_symbols(m)?;
    overpass_query::register_python_symbols(m)?;
    stats::register_python_symbols(m)?;
    util::register_python_symbols(m)?;
    webframe::register_python_symbols(m)?;
    wsgi::register_python_symbols(m)?;
    yattag::register_python_symbols(m)?;
    Ok(())
}
