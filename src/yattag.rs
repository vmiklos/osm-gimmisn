/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Generate HTML with Rust.
//!
//! This is more or less a Rust port of the Python package, mostly because
//! <https://crates.io/crates/html-builder> would require you to manually escape attribute values.

use pyo3::class::PyContextProtocol;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::sync::Arc;
use std::sync::Mutex;

/// Generates xml/html documents.
pub struct Doc {
    value: Arc<Mutex<String>>,
}

impl Doc {
    pub fn new() -> Doc {
        Doc {
            value: Arc::new(Mutex::new(String::from(""))),
        }
    }

    /// Factory of yattag.Doc from a string.
    fn from_text(text: &str) -> Self {
        let doc = Doc::new();
        doc.text(text);
        doc
    }

    /// Gets the escaped value.
    pub fn get_value(&self) -> String {
        self.value.lock().unwrap().clone()
    }

    /// Appends escaped content to the value.
    pub fn append_value(&self, value: String) {
        self.value.lock().unwrap().push_str(&value)
    }

    /// Starts a new tag.
    pub fn tag(&self, name: &str, attrs: Vec<(&str, &str)>) -> Tag {
        Tag::new(&self.value, name, attrs)
    }

    /// Starts a new tag and closes it as well.
    pub fn stag(&self, name: &str, attrs: Vec<(&str, &str)>) {
        self.append_value(format!("<{}", name));
        for attr in attrs {
            let key = attr.0;
            let value = html_escape::encode_double_quoted_attribute(&attr.1);
            self.append_value(format!(" {}=\"{}\"", key, value));
        }
        self.append_value(String::from(" />"))
    }

    /// Appends unescaped content to the document.
    pub fn text(&self, text: &str) {
        let encoded = html_escape::encode_safe(text).to_string();
        self.append_value(encoded);
    }
}

impl Default for Doc {
    fn default() -> Self {
        Self::new()
    }
}

#[pyclass]
pub struct PyDoc {
    pub doc: Doc,
}

#[pymethods]
impl PyDoc {
    #[new]
    fn new() -> Self {
        let doc = Doc::new();
        PyDoc { doc }
    }

    #[staticmethod]
    fn from_text(text: String) -> Self {
        let doc = Doc::from_text(&text);
        PyDoc { doc }
    }

    fn get_value(&self) -> String {
        self.doc.get_value()
    }

    fn append_value(&self, value: String) {
        self.doc.append_value(value)
    }

    fn tag(&self, name: &str, attrs: Vec<(&str, &str)>) -> PyTag {
        let tag = self.doc.tag(name, attrs);
        PyTag { tag: Some(tag) }
    }

    fn stag(&self, name: &str, attrs: Vec<(&str, &str)>) {
        self.doc.stag(name, attrs)
    }

    fn text(&self, text: &str) {
        self.doc.text(text)
    }
}

/// Starts a tag, which is closed automatically.
pub struct Tag {
    value: Arc<Mutex<String>>,
    name: String,
}

impl Tag {
    fn new(value: &Arc<Mutex<String>>, name: &str, attrs: Vec<(&str, &str)>) -> Tag {
        let mut locked_value = value.lock().unwrap();
        locked_value.push_str(&format!("<{}", name));
        for attr in attrs {
            let key = attr.0;
            let val = html_escape::encode_double_quoted_attribute(&attr.1);
            locked_value.push_str(&format!(" {}=\"{}\"", key, val));
        }
        locked_value.push('>');
        let value = value.clone();
        Tag {
            value,
            name: name.to_string(),
        }
    }
}

impl Drop for Tag {
    fn drop(&mut self) {
        self.value
            .lock()
            .unwrap()
            .push_str(&format!("</{}>", self.name));
    }
}

#[pyclass]
pub struct PyTag {
    tag: Option<Tag>,
}

#[pymethods]
impl PyTag {
    #[new]
    fn new(py: Python<'_>, doc: PyObject, name: &str, attrs: Vec<(&str, &str)>) -> PyResult<Self> {
        // Convert PyObject to Doc.
        let doc: PyRefMut<'_, PyDoc> = doc.extract(py)?;
        let tag = Tag::new(&doc.doc.value, name, attrs);
        Ok(PyTag { tag: Some(tag) })
    }
}

#[pyproto]
impl<'p> PyContextProtocol<'p> for PyTag {
    fn __enter__(&'p mut self) -> PyResult<()> {
        Ok(())
    }

    fn __exit__(
        &mut self,
        ty: Option<&'p PyType>,
        _value: Option<&'p PyAny>,
        _traceback: Option<&'p PyAny>,
    ) -> bool {
        if self.tag.is_some() {
            self.tag = None;
        }
        let gil = Python::acquire_gil();
        ty == Some(gil.python().get_type::<PyValueError>())
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyDoc>()?;
    module.add_class::<PyTag>()?;
    Ok(())
}
