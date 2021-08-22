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
mod version;
mod yattag;

#[pymodule]
fn rust(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<context::PyIni>()?;
    m.add_class::<context::PyContext>()?;
    m.add_class::<ranges::PyRange>()?;
    m.add_class::<ranges::PyRanges>()?;
    m.add_class::<yattag::PyDoc>()?;
    m.add_class::<yattag::PyTag>()?;
    accept_language::register_python_symbols(&m)?;
    i18n::register_python_symbols(&m)?;
    overpass_query::register_python_symbols(&m)?;
    version::register_python_symbols(&m)?;

    Ok(())
}
