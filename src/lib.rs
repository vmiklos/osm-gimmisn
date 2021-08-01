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

mod ranges;

#[pymodule]
fn rust(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ranges::PyRange>()?;
    m.add_class::<ranges::PyRanges>()?;

    Ok(())
}
