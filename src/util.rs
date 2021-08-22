/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The util module contains functionality shared between other modules.

use pyo3::prelude::*;

/// Specifies the style of the output of normalize_letter_suffix().
enum LetterSuffixStyle {
    /// "42/A"
    Upper,
    /// "42a"
    Lower,
}

#[pyclass]
pub struct PyLetterSuffixStyle {
}

#[pymethods]
impl PyLetterSuffixStyle {
    #[staticmethod]
    fn upper() -> i32 {
        LetterSuffixStyle::Upper as i32
    }

    #[staticmethod]
    fn lower() -> i32 {
        LetterSuffixStyle::Lower as i32
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyLetterSuffixStyle>()?;
    Ok(())
}
