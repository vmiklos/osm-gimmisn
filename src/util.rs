/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The util module contains functionality shared between other modules.

use pyo3::class::basic::CompareOp;
use pyo3::class::PyObjectProtocol;
use pyo3::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// Specifies the style of the output of normalize_letter_suffix().
enum LetterSuffixStyle {
    /// "42/A"
    Upper,
    /// "42a"
    Lower,
}

#[pyclass]
pub struct PyLetterSuffixStyle {}

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

/// A house number range is a string that may expand to one or more HouseNumber instances in the
/// future. It can also have a comment.
#[derive(Debug)]
struct HouseNumberRange {
    number: String,
    comment: String,
}

impl HouseNumberRange {
    fn new(number: &str, comment: &str) -> Self {
        HouseNumberRange {
            number: number.into(),
            comment: comment.into(),
        }
    }

    /// Returns the house number (range) string.
    fn get_number(&self) -> &String {
        &self.number
    }

    /// Returns the comment.
    fn get_comment(&self) -> &String {
        &self.comment
    }

    /// Comment is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

impl PartialEq for HouseNumberRange {
    /// Comment is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Hash for HouseNumberRange {
    /// Comment is explicitly non-interesting.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

#[pyclass]
#[derive(Debug)]
struct PyHouseNumberRange {
    house_number_range: HouseNumberRange,
}

#[pymethods]
impl PyHouseNumberRange {
    #[new]
    fn new(number: &str, comment: &str) -> Self {
        let house_number_range = HouseNumberRange::new(number, comment);
        PyHouseNumberRange { house_number_range }
    }

    fn get_number(&self) -> &String {
        &self.house_number_range.get_number()
    }

    fn get_comment(&self) -> &String {
        &self.house_number_range.get_comment()
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for PyHouseNumberRange {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    fn __richcmp__(
        &'p self,
        other: PyRef<'p, PyHouseNumberRange>,
        op: CompareOp,
    ) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(self
                .house_number_range
                .eq(&(*other).house_number_range)
                .into_py(other.py())),
            CompareOp::Lt => Ok((self.house_number_range.cmp(&(*other).house_number_range)
                == std::cmp::Ordering::Less)
                .into_py(other.py())),
            _ => Ok(other.py().NotImplemented()),
        }
    }

    fn __hash__(&self) -> PyResult<isize> {
        let mut hasher = DefaultHasher::new();
        self.house_number_range.hash(&mut hasher);
        Ok(hasher.finish() as isize)
    }
}

/// A street has an OSM and a reference name. Ideally the two are the same. Sometimes the reference
/// name differs.
#[derive(Debug)]
struct Street {
    osm_name: String,
    ref_name: String,
    show_ref_street: bool,
    osm_id: u64,
    osm_type: String,
    source: String,
}

impl Street {
    fn new(osm_name: &str, ref_name: &str, show_ref_street: bool, osm_id: u64) -> Street {
        Street {
            osm_name: osm_name.into(),
            ref_name: ref_name.into(),
            show_ref_street,
            osm_id,
            osm_type: "way".into(),
            source: "".into(),
        }
    }

    /// Constructor that only requires an OSM name.
    fn from_string(osm_name: &str) -> Street {
        Street::new(osm_name, "", true, 0)
    }

    /// Gets a string that is used while diffing.
    fn get_diff_key(&self) -> String {
        let re = regex::Regex::new(r"\*$").unwrap();
        re.replace(&self.osm_name, "").to_string()
    }

    /// Returns the OSM name.
    fn get_osm_name(&self) -> &str {
        &self.osm_name
    }

    /// Returns the reference name.
    fn get_ref_name(&self) -> &str {
        &self.ref_name
    }

    /// Returns the OSM (way) id.
    fn get_osm_id(&self) -> u64 {
        self.osm_id
    }

    /// Sets the OSM type, e.g. 'way'.
    fn set_osm_type(&mut self, osm_type: &str) {
        self.osm_type = osm_type.into()
    }

    /// Returns the OSM type, e.g. 'way'.
    fn get_osm_type(&self) -> &str {
        &self.osm_type
    }

    /// Sets the source of this street.
    fn set_source(&mut self, source: &str) {
        self.source = source.into()
    }

    /// Gets the source of this street.
    fn get_source(&self) -> &str {
        &self.source
    }

    /// Writes the street as a HTML string.
    fn to_html(&self) -> crate::yattag::Doc {
        let doc = crate::yattag::Doc::new();
        doc.text(&self.osm_name);
        if self.osm_name != self.ref_name && self.show_ref_street {
            doc.stag("br", vec![]);
            doc.text("(");
            doc.text(&self.ref_name);
            doc.text(")");
        }
        doc
    }

    /// OSM id is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.osm_name.cmp(&other.osm_name)
    }
}

impl PartialEq for Street {
    /// OSM id is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.osm_name == other.osm_name
    }
}

impl Hash for Street {
    /// OSM id is explicitly not interesting.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.osm_name.hash(state);
    }
}

#[pyclass]
#[derive(Debug)]
struct PyStreet {
    street: Street,
}

#[pymethods]
impl PyStreet {
    #[new]
    fn new(osm_name: &str, ref_name: &str, show_ref_street: bool, osm_id: u64) -> Self {
        let street = Street::new(osm_name, ref_name, show_ref_street, osm_id);
        PyStreet { street }
    }

    #[staticmethod]
    fn from_string(osm_name: &str) -> Self {
        let street = Street::from_string(osm_name);
        PyStreet { street }
    }

    fn get_diff_key(&self) -> String {
        self.street.get_diff_key()
    }

    fn get_osm_name(&self) -> &str {
        self.street.get_osm_name()
    }

    fn get_ref_name(&self) -> &str {
        self.street.get_ref_name()
    }

    fn get_osm_id(&self) -> u64 {
        self.street.get_osm_id()
    }

    fn set_osm_type(&mut self, osm_type: &str) {
        self.street.set_osm_type(osm_type)
    }

    fn get_osm_type(&self) -> &str {
        self.street.get_osm_type()
    }

    fn set_source(&mut self, source: &str) {
        self.street.set_source(source)
    }

    fn get_source(&self) -> &str {
        self.street.get_source()
    }

    fn to_html(&self) -> crate::yattag::PyDoc {
        let doc = self.street.to_html();
        crate::yattag::PyDoc { doc }
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for PyStreet {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    fn __richcmp__(&'p self, other: PyRef<'p, PyStreet>, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(self.street.eq(&(*other).street).into_py(other.py())),
            CompareOp::Lt => Ok(
                (self.street.cmp(&(*other).street) == std::cmp::Ordering::Less).into_py(other.py()),
            ),
            _ => Ok(other.py().NotImplemented()),
        }
    }

    fn __hash__(&self) -> PyResult<isize> {
        let mut hasher = DefaultHasher::new();
        self.street.hash(&mut hasher);
        Ok(hasher.finish() as isize)
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyHouseNumberRange>()?;
    module.add_class::<PyLetterSuffixStyle>()?;
    module.add_class::<PyStreet>()?;
    Ok(())
}
