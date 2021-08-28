/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The util module contains functionality shared between other modules.

use anyhow::anyhow;
use lazy_static::lazy_static;
use pyo3::class::basic::CompareOp;
use pyo3::class::PyObjectProtocol;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::types::PyType;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::Read;

lazy_static! {
    static ref NUMBER_PER_LETTER: regex::Regex =
        regex::Regex::new(r"^([0-9]+)( |/)?[A-Za-z]$").unwrap();
    static ref NUMBER_PER_NUMBER: regex::Regex = regex::Regex::new(r"^([0-9]+)/[0-9]$").unwrap();
    static ref NUMBER_WITH_JUNK: regex::Regex = regex::Regex::new(r"([0-9]+).*").unwrap();
    static ref NUMBER_WITH_REMAINDER: regex::Regex =
        regex::Regex::new(r"^([0-9]*)([^0-9].*|)$").unwrap();
    static ref LETTER_SUFFIX: regex::Regex = regex::Regex::new(r".*([A-Za-z]+)\*?").unwrap();
    static ref NUMBER_SUFFIX: regex::Regex = regex::Regex::new(r"^.*/([0-9])\*?$").unwrap();
    static ref NULL_END: regex::Regex = regex::Regex::new(r" null$").unwrap();
}

/// Specifies the style of the output of normalize_letter_suffix().
#[derive(PartialEq)]
enum LetterSuffixStyle {
    /// "42/A"
    Upper,
    /// "42a"
    Lower,
}

/// Only needed for Python interop.
impl TryFrom<i32> for LetterSuffixStyle {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == LetterSuffixStyle::Upper as i32 => Ok(LetterSuffixStyle::Upper),
            x if x == LetterSuffixStyle::Lower as i32 => Ok(LetterSuffixStyle::Lower),
            _ => Err(()),
        }
    }
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
#[derive(Clone, Debug)]
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

/// A house number is a string which remembers what was its provider range.  E.g. the "1-3" string
/// can generate 3 house numbers, all of them with the same range.
/// The comment is similar to source, it's ignored during __eq__() and __hash__().
#[derive(Debug)]
struct HouseNumber {
    number: String,
    source: String,
    comment: String,
}

impl HouseNumber {
    fn new(number: &str, source: &str, comment: &str) -> Self {
        HouseNumber {
            number: number.into(),
            source: source.into(),
            comment: comment.into(),
        }
    }

    /// Returns the house number string.
    fn get_number(&self) -> &str {
        &self.number
    }

    /// Gets a string that is used while diffing.
    fn get_diff_key(&self) -> String {
        let re = regex::Regex::new(r"\*$").unwrap();
        re.replace(&self.number, "").to_string()
    }

    /// Returns the source range.
    fn get_source(&self) -> &str {
        &self.source
    }

    /// Returns the comment.
    fn get_comment(&self) -> &str {
        &self.comment
    }

    /// Decides if house_number is invalid according to invalids.
    fn is_invalid(house_number: &str, invalids: &[String]) -> bool {
        if invalids.contains(&house_number.to_string()) {
            return true;
        }

        let mut number: String = "".into();
        if let Some(cap) = NUMBER_WITH_JUNK.captures_iter(house_number).next() {
            number = cap[1].into();
        }
        let mut suffix: String = "".into();
        // Check for letter suffix.
        if let Some(cap) = LETTER_SUFFIX.captures_iter(house_number).next() {
            suffix = cap[1].to_string().to_lowercase();
        }
        // If not, then try digit suggfix, but then only '/' is OK as a separator.
        if suffix.is_empty() {
            let mut iter = NUMBER_SUFFIX.captures_iter(house_number);
            if let Some(cap) = iter.next() {
                suffix = "/".into();
                suffix += &cap[1].to_string();
            }
        }

        let house_number = number + &suffix;
        invalids.contains(&house_number)
    }

    /// Determines if the input is a house number, allowing letter suffixes. This means not only
    /// '42' is allowed, but also '42a', '42/a' and '42 a'. Everything else is still considered just
    /// junk after the numbers.
    fn has_letter_suffix(house_number: &str, source_suffix: &str) -> bool {
        let mut house_number: String = house_number.into();
        if !source_suffix.is_empty() {
            house_number = house_number[..house_number.len() - source_suffix.len()].into();
        }
        // Check for letter suffix.
        if NUMBER_PER_LETTER.is_match(&house_number) {
            return true;
        }
        // If not, then try digit suggfix, but then only '/' is OK as a separator.
        NUMBER_PER_NUMBER.is_match(&house_number)
    }

    /// Turn '42A' and '42 A' (and their lowercase versions) into '42/A'.
    fn normalize_letter_suffix(
        house_number: &str,
        source_suffix: &str,
        style: LetterSuffixStyle,
    ) -> anyhow::Result<String> {
        let mut house_number: String = house_number.into();
        if !source_suffix.is_empty() {
            house_number = house_number[..house_number.len() - source_suffix.len()].into();
        }
        // Check for letter suffix.
        let re = regex::Regex::new(r"^([0-9]+)( |/)?([A-Za-z])$").unwrap();
        let is_match = re.is_match(&house_number);
        let mut digit_match = false;
        let mut groups: Vec<String> = Vec::new();
        if is_match {
            if let Some(cap) = re.captures_iter(&house_number).next() {
                for index in 1..=3 {
                    match cap.get(index) {
                        Some(_) => groups.push(cap[index].to_string()),
                        None => groups.push(String::from("")),
                    }
                }
            }
        } else {
            // If not, then try digit suggfix, but then only '/' is OK as a separator.
            let re = regex::Regex::new(r"^([0-9]+)(/)([0-9])$").unwrap();
            let is_match = re.is_match(&house_number);
            digit_match = true;
            if !is_match {
                return Err(anyhow!("ValueError"));
            }
            if let Some(cap) = re.captures_iter(&house_number).next() {
                for index in 1..=3 {
                    match cap.get(index) {
                        Some(_) => groups.push(cap[index].to_string()),
                        None => groups.push(String::from("")),
                    }
                }
            };
        }

        let mut ret: String = groups[0].clone();
        if style == LetterSuffixStyle::Upper || digit_match {
            ret += "/";
            ret += &groups[2].to_uppercase();
        } else {
            ret += &groups[2].to_lowercase();
        }
        ret += source_suffix;
        Ok(ret)
    }
}

impl PartialEq for HouseNumber {
    /// Source is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Hash for HouseNumber {
    /// Source is explicitly non-interesting.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

#[pyclass]
#[derive(Debug)]
struct PyHouseNumber {
    house_number: HouseNumber,
}

#[pymethods]
impl PyHouseNumber {
    #[new]
    fn new(number: &str, source: &str, comment: &str) -> Self {
        let house_number = HouseNumber::new(number, source, comment);
        PyHouseNumber { house_number }
    }

    fn get_number(&self) -> &str {
        self.house_number.get_number()
    }

    fn get_diff_key(&self) -> String {
        self.house_number.get_diff_key()
    }

    fn get_source(&self) -> &str {
        self.house_number.get_source()
    }

    fn get_comment(&self) -> &str {
        self.house_number.get_comment()
    }

    #[staticmethod]
    fn is_invalid(house_number: &str, invalids: Vec<String>) -> bool {
        HouseNumber::is_invalid(house_number, &invalids)
    }

    #[staticmethod]
    fn has_letter_suffix(house_number: &str, source_suffix: &str) -> bool {
        HouseNumber::has_letter_suffix(house_number, source_suffix)
    }

    #[staticmethod]
    fn normalize_letter_suffix(
        house_number: &str,
        source_suffix: &str,
        style: i32,
    ) -> PyResult<String> {
        let style: LetterSuffixStyle = match LetterSuffixStyle::try_from(style) {
            Ok(value) => value,
            Err(_) => {
                return Err(pyo3::exceptions::PyOSError::new_err(
                    "failed to convert style to LetterSuffixStyle",
                ));
            }
        };
        match HouseNumber::normalize_letter_suffix(
            house_number,
            source_suffix,
            style as LetterSuffixStyle,
        ) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "normalize_letter_suffix() failed: {}",
                err.to_string()
            ))),
        }
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for PyHouseNumber {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    fn __richcmp__(&'p self, other: PyRef<'p, PyHouseNumber>, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(self
                .house_number
                .eq(&(*other).house_number)
                .into_py(other.py())),
            _ => Ok(other.py().NotImplemented()),
        }
    }

    fn __hash__(&self) -> PyResult<isize> {
        let mut hasher = DefaultHasher::new();
        self.house_number.hash(&mut hasher);
        Ok(hasher.finish() as isize)
    }
}

/// Like Read, but for CSV reading.
struct CsvRead<'a> {
    reader: csv::Reader<&'a mut dyn Read>,
}

impl<'a> CsvRead<'a> {
    fn new(read: &'a mut dyn Read) -> Self {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .double_quote(true)
            .from_reader(read);
        CsvRead { reader }
    }

    /// Gets access to the rows of the CSV.
    fn records(&mut self) -> csv::StringRecordsIter<'_, &'a mut dyn Read> {
        self.reader.records()
    }
}

#[pyclass]
struct PyCsvRead {
    buf: Vec<u8>,
}

#[pymethods]
impl PyCsvRead {
    #[new]
    fn new(py: Python<'_>, stream: PyObject) -> PyResult<Self> {
        let any = match stream.call_method0(py, "read") {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(err.to_string()));
            }
        };
        stream.call_method0(py, "close").unwrap();
        let bytes = match any.as_ref(py).downcast::<PyBytes>() {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(err.to_string()));
            }
        };
        let buf: Vec<u8> = bytes.extract().unwrap();
        Ok(PyCsvRead { buf })
    }

    fn get_rows(&mut self) -> PyResult<Vec<Vec<String>>> {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut self.buf);
        let mut csv_read = CsvRead::new(&mut cursor);
        for result in csv_read.records() {
            let record: csv::StringRecord = match result {
                Ok(value) => value,
                Err(err) => {
                    return Err(pyo3::exceptions::PyOSError::new_err(err.to_string()));
                }
            };
            let mut row: Vec<String> = Vec::new();
            for col in record.iter() {
                row.push(col.into());
            }
            rows.push(row);
        }
        Ok(rows)
    }

    fn __enter__(&self) -> Self {
        let buf = self.buf.clone();
        PyCsvRead { buf }
    }

    fn __exit__(
        &mut self,
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> bool {
        let gil = Python::acquire_gil();
        ty == Some(gil.python().get_type::<PyValueError>())
    }
}

/// Splits house_number into a numerical and a remainder part.
fn split_house_number(house_number: &str) -> (i32, String) {
    let mut number = 0;
    let mut remainder: String = "".into();
    if let Some(cap) = NUMBER_WITH_REMAINDER.captures_iter(house_number).next() {
        if let Ok(value) = cap[1].parse::<i32>() {
            number = value;
        }
        remainder = cap[2].to_string();
    }
    (number, remainder)
}

#[pyfunction]
pub fn py_split_house_number(house_number: String) -> PyResult<(i32, String)> {
    Ok(split_house_number(&house_number))
}

/// Wrapper around split_house_number() for HouseNumberRange objects.
fn split_house_number_range(house_number: &HouseNumberRange) -> (i32, String) {
    split_house_number(house_number.get_number())
}

#[pyfunction]
pub fn py_split_house_number_range(
    py: Python<'_>,
    house_number_range: PyObject,
) -> PyResult<(i32, String)> {
    let py_house_number_range: PyRefMut<'_, PyHouseNumberRange> = house_number_range.extract(py)?;
    Ok(split_house_number_range(
        &py_house_number_range.house_number_range,
    ))
}

/// Separates even and odd numbers.
fn separate_even_odd(
    only_in_ref: &[HouseNumberRange],
    even: &mut Vec<HouseNumberRange>,
    odd: &mut Vec<HouseNumberRange>,
) {
    let mut even_unsorted: Vec<HouseNumberRange> = only_in_ref
        .iter()
        .filter(|i| split_house_number(i.get_number()).0 % 2 == 0)
        .cloned()
        .collect();
    even_unsorted.sort_by(|a, b| {
        split_house_number_range(a)
            .0
            .cmp(&split_house_number_range(b).0)
    });
    *even = even_unsorted;

    let mut odd_unsorted: Vec<HouseNumberRange> = only_in_ref
        .iter()
        .filter(|i| split_house_number(i.get_number()).0 % 2 == 1)
        .cloned()
        .collect();
    odd_unsorted.sort_by(|a, b| {
        split_house_number_range(a)
            .0
            .cmp(&split_house_number_range(b).0)
    });
    *odd = odd_unsorted;
}

/// Formats even and odd numbers.
fn format_even_odd(only_in_ref: &[HouseNumberRange]) -> Vec<String> {
    let mut even: Vec<HouseNumberRange> = Vec::new();
    let mut odd: Vec<HouseNumberRange> = Vec::new();
    separate_even_odd(only_in_ref, &mut even, &mut odd);
    let even_numbers: Vec<String> = even.iter().map(|i| i.get_number().clone()).collect();
    let even_string = even_numbers.join(", ");
    let odd_numbers: Vec<String> = odd.iter().map(|i| i.get_number().clone()).collect();
    let mut elements: Vec<String> = Vec::new();
    let odd_string = odd_numbers.join(", ");
    if !odd_string.is_empty() {
        elements.push(odd_string);
    }
    if !even_string.is_empty() {
        elements.push(even_string);
    }
    elements
}

#[pyfunction]
fn py_format_even_odd(py: Python<'_>, items: Vec<PyObject>) -> PyResult<Vec<String>> {
    // Convert Vec<PyObject> to Vec<HouseNumberRange>.
    let items: Vec<HouseNumberRange> = items
        .iter()
        .map(|item| {
            let item: PyRefMut<'_, PyHouseNumberRange> = item.extract(py)?;
            Ok(item.house_number_range.clone())
        })
        .collect::<PyResult<Vec<HouseNumberRange>>>()?;

    Ok(format_even_odd(&items))
}

/// Formats even and odd numbers, HTML version.
fn format_even_odd_html(only_in_ref: &[HouseNumberRange]) -> crate::yattag::Doc {
    let mut even: Vec<HouseNumberRange> = Vec::new();
    let mut odd: Vec<HouseNumberRange> = Vec::new();
    separate_even_odd(only_in_ref, &mut even, &mut odd);
    let doc = crate::yattag::Doc::new();
    for (index, elem) in odd.iter().enumerate() {
        if index > 0 {
            doc.text(", ");
        }
        doc.append_value(color_house_number(elem).get_value());
    }
    if !even.is_empty() && !odd.is_empty() {
        doc.stag("br", vec![]);
    }
    for (index, elem) in even.iter().enumerate() {
        if index > 0 {
            doc.text(", ");
        }
        doc.append_value(color_house_number(elem).get_value());
    }
    doc
}

#[pyfunction]
fn py_format_even_odd_html(py: Python<'_>, items: Vec<PyObject>) -> PyResult<crate::yattag::PyDoc> {
    // Convert Vec<PyObject> to Vec<HouseNumberRange>.
    let items: Vec<HouseNumberRange> = items
        .iter()
        .map(|item| {
            let item: PyRefMut<'_, PyHouseNumberRange> = item.extract(py)?;
            Ok(item.house_number_range.clone())
        })
        .collect::<PyResult<Vec<HouseNumberRange>>>()?;

    Ok(crate::yattag::PyDoc {
        doc: format_even_odd_html(&items),
    })
}

/// Colors a house number according to its suffix.
fn color_house_number(house_number: &HouseNumberRange) -> crate::yattag::Doc {
    let doc = crate::yattag::Doc::new();
    let number = house_number.get_number();
    if !number.ends_with('*') {
        doc.text(number);
        return doc;
    }
    let mut chars = number.chars();
    chars.next_back();
    let number = chars.as_str();
    let title = house_number.get_comment().replace("&#013;", "\n");
    let _span = doc.tag("span", vec![("style", "color: blue;")]);
    if !title.is_empty() {
        {
            let _abbr = doc.tag("abbr", vec![("title", &title), ("tabindex", "0")]);
            doc.text(number);
        }
    } else {
        doc.text(number);
    }
    doc
}

#[pyfunction]
fn py_color_house_number(py: Python<'_>, house_number: PyObject) -> PyResult<crate::yattag::PyDoc> {
    let house_number: PyRefMut<'_, PyHouseNumberRange> = house_number.extract(py)?;
    Ok(crate::yattag::PyDoc {
        doc: color_house_number(&house_number.house_number_range),
    })
}

/// refcounty -> refsettlement -> streets cache.
type StreetReferenceCache = HashMap<String, HashMap<String, Vec<String>>>;

/// Builds an in-memory cache from the reference on-disk TSV (street version).
fn build_street_reference_cache(local_streets: &str) -> anyhow::Result<StreetReferenceCache> {
    let mut memory_cache: StreetReferenceCache = HashMap::new();

    let disk_cache = local_streets.to_string() + ".cache";
    if std::path::Path::new(&disk_cache).exists() {
        let stream = std::fs::File::open(disk_cache)?;
        memory_cache = serde_json::from_reader(&stream)?;
        return Ok(memory_cache);
    }

    let stream = std::io::BufReader::new(std::fs::File::open(local_streets)?);
    let mut first = true;
    for line in stream.lines() {
        let line = line?.to_string();
        if first {
            first = false;
            continue;
        }

        let columns: Vec<&str> = line.split('\t').collect();
        let refcounty = columns[0];
        let refsettlement = columns[1];
        // Filter out invalid street type.
        let street = NULL_END.replace(columns[2], "").to_string();
        let refcounty_key = memory_cache
            .entry(refcounty.into())
            .or_insert_with(HashMap::new);
        let refsettlement_key = refcounty_key
            .entry(refsettlement.into())
            .or_insert_with(Vec::new);
        refsettlement_key.push(street);
    }

    let stream = std::fs::File::create(disk_cache)?;
    serde_json::to_writer(&stream, &memory_cache)?;

    Ok(memory_cache)
}

#[pyfunction]
fn py_build_street_reference_cache(local_streets: String) -> PyResult<StreetReferenceCache> {
    match build_street_reference_cache(&local_streets) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "build_street_reference_cache() failed: {}",
            err.to_string()
        ))),
    }
}

/// Gets the filename of the (house number) reference cache file.
fn get_reference_cache_path(local: &str, refcounty: &str) -> String {
    format!("{}-{}-v1.cache", local, refcounty)
}

#[pyfunction]
fn py_get_reference_cache_path(local: String, refcounty: String) -> String {
    get_reference_cache_path(&local, &refcounty)
}

/// Two strings: first is a range, second is an optional comment.
type HouseNumberWithComment = Vec<String>;

/// refcounty -> refsettlement -> street -> housenumbers cache.
type HouseNumberReferenceCache =
    HashMap<String, HashMap<String, HashMap<String, Vec<HouseNumberWithComment>>>>;

/// Builds an in-memory cache from the reference on-disk TSV (house number version).
fn build_reference_cache(
    local: &str,
    refcounty: &str,
) -> anyhow::Result<HouseNumberReferenceCache> {
    let mut memory_cache: HouseNumberReferenceCache = HashMap::new();

    let disk_cache = get_reference_cache_path(local, refcounty);

    if std::path::Path::new(&disk_cache).exists() {
        let stream = std::fs::File::open(disk_cache)?;
        memory_cache = serde_json::from_reader(&stream)?;
        return Ok(memory_cache);
    }

    let stream = std::io::BufReader::new(std::fs::File::open(local)?);
    let mut first = true;
    for line in stream.lines() {
        let line = line?.to_string();
        if first {
            first = false;
            continue;
        }

        if !line.starts_with(refcounty) {
            continue;
        }

        let columns: Vec<&str> = line.split('\t').collect();
        let refcounty = columns[0];
        let refsettlement = columns[1];
        let street = columns[2];
        let num: String = columns[3].into();
        let mut comment: String = "".into();
        if columns.len() >= 5 {
            comment = columns[4].into();
        }
        let refcounty_key = memory_cache
            .entry(refcounty.into())
            .or_insert_with(HashMap::new);
        let refsettlement_key = refcounty_key
            .entry(refsettlement.into())
            .or_insert_with(HashMap::new);
        let street_key = refsettlement_key
            .entry(street.into())
            .or_insert_with(Vec::new);
        street_key.push(vec![num, comment]);
    }

    let stream = std::fs::File::create(disk_cache)?;
    serde_json::to_writer(&stream, &memory_cache)?;

    Ok(memory_cache)
}

#[pyfunction]
fn py_build_reference_cache(
    local: String,
    refcounty: String,
) -> PyResult<HouseNumberReferenceCache> {
    match build_reference_cache(&local, &refcounty) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "build_reference_cache() failed: {}",
            err.to_string()
        ))),
    }
}

/// Handles a list of references for build_reference_cache().
fn build_reference_caches(
    references: Vec<String>,
    refcounty: &str,
) -> anyhow::Result<Vec<HouseNumberReferenceCache>> {
    references
        .iter()
        .map(|reference| build_reference_cache(reference, refcounty))
        .collect()
}

#[pyfunction]
fn py_build_reference_caches(
    references: Vec<String>,
    refcounty: String,
) -> PyResult<Vec<HouseNumberReferenceCache>> {
    match build_reference_caches(references, &refcounty) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "build_reference_caches() failed: {}",
            err.to_string()
        ))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyHouseNumber>()?;
    module.add_class::<PyHouseNumberRange>()?;
    module.add_class::<PyLetterSuffixStyle>()?;
    module.add_class::<PyStreet>()?;
    module.add_class::<PyCsvRead>()?;
    module.add_function(pyo3::wrap_pyfunction!(py_split_house_number, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_split_house_number_range, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_format_even_odd, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_format_even_odd_html, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_color_house_number, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_build_street_reference_cache,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_reference_cache_path, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_build_reference_cache, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_build_reference_caches, module)?)?;
    Ok(())
}
