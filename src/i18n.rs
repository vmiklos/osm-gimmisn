/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The i18n module allows UI translation via gettext.

use pyo3::prelude::*;

thread_local! {
    static TRANSLATIONS: std::cell::RefCell<Option<gettext::Catalog>> = std::cell::RefCell::new(None);
    static LANGUAGE: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

/// Sets the language of the current thread.
pub fn set_language(language: &str) -> anyhow::Result<()> {
    // Not using ctx.get_abspath() here, tests/ doesn't have its own dummy translations.
    let root_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!(
        "{}/locale/{}/LC_MESSAGES/osm-gimmisn.mo",
        root_dir, language
    );

    if std::path::Path::new(&path).exists() {
        let file = std::fs::File::open(path)?;
        let catalog = gettext::Catalog::parse(file)?;
        TRANSLATIONS.with(|it| {
            *it.borrow_mut() = Some(catalog);
        });
    } else {
        TRANSLATIONS.with(|it| {
            *it.borrow_mut() = None;
        });
    }
    LANGUAGE.with(|it| {
        *it.borrow_mut() = Some(String::from(language));
    });
    Ok(())
}

#[pyfunction]
pub fn py_set_language(language: String) -> PyResult<()> {
    match set_language(&language) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "set_language() failed: {}",
            err.to_string()
        ))),
    }
}

/// Gets the language of the current thread.
pub fn get_language() -> String {
    LANGUAGE.with(|language| {
        let language = language.borrow();
        match *language {
            Some(ref language) => language.clone(),
            None => String::from("en"),
        }
    })
}

#[pyfunction]
pub fn py_get_language(_py: Python<'_>) -> String {
    get_language()
}

/// Translates English input according to the current UI language.
pub fn translate(english: &str) -> String {
    TRANSLATIONS.with(|translations| {
        let translations = translations.borrow();
        match *translations {
            Some(ref translations) => translations.gettext(english).to_string(),
            None => english.to_string(),
        }
    })
}

#[pyfunction]
pub fn py_translate(_py: Python<'_>, english: String) -> String {
    translate(&english)
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_set_language, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_language, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_translate, module)?)?;
    Ok(())
}
