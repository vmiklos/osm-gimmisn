/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The util module contains functionality shared between other modules.

use crate::context;
use crate::i18n;
use crate::i18n::translate as tr;
use crate::overpass_query;
use crate::ranges;
use crate::yattag;
use anyhow::anyhow;
use lazy_static::lazy_static;
use pyo3::class::basic::CompareOp;
use pyo3::class::PyObjectProtocol;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::types::PyType;
use rust_icu_ucol as ucol;
use rust_icu_ustring as ustring;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::BufRead;
use std::io::Read;
use std::ops::DerefMut;

lazy_static! {
    static ref NUMBER_PER_LETTER: regex::Regex =
        regex::Regex::new(r"^([0-9]+)( |/)?([A-Za-z])$").unwrap();
    static ref NUMBER_PER_NUMBER: regex::Regex =
        regex::Regex::new(r"^([0-9]+)(/)([0-9])$").unwrap();
    static ref NUMBER_WITH_JUNK: regex::Regex = regex::Regex::new(r"([0-9]+).*").unwrap();
    static ref NUMBER_WITH_REMAINDER: regex::Regex =
        regex::Regex::new(r"^([0-9]*)([^0-9].*|)$").unwrap();
    static ref LETTER_SUFFIX: regex::Regex = regex::Regex::new(r".*([A-Za-z]+)\*?").unwrap();
    static ref NUMBER_SUFFIX: regex::Regex = regex::Regex::new(r"^.*/([0-9])\*?$").unwrap();
    static ref NULL_END: regex::Regex = regex::Regex::new(r" null$").unwrap();
    static ref GIT_HASH: regex::Regex = regex::Regex::new(r".*-g([0-9a-f]+)(-modified)?").unwrap();
}

/// Specifies the style of the output of normalize_letter_suffix().
#[derive(PartialEq)]
pub enum LetterSuffixStyle {
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
pub struct HouseNumberRange {
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
    pub fn get_number(&self) -> &String {
        &self.number
    }

    /// Returns the comment.
    fn get_comment(&self) -> &String {
        &self.comment
    }
}

impl Ord for HouseNumberRange {
    /// Comment is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

impl PartialOrd for HouseNumberRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HouseNumberRange {
    /// Comment is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Eq for HouseNumberRange {}

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
    fn get_number(&self) -> &String {
        self.house_number_range.get_number()
    }

    fn get_comment(&self) -> &String {
        self.house_number_range.get_comment()
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

/// Used to diff two lists of elements.
pub trait Diff {
    /// Gets a string that is used while diffing.
    fn get_diff_key(&self) -> String;
}

/// A street has an OSM and a reference name. Ideally the two are the same. Sometimes the reference
/// name differs.
#[derive(Clone, Debug)]
pub struct Street {
    osm_name: String,
    ref_name: String,
    show_ref_street: bool,
    osm_id: u64,
    osm_type: String,
    source: String,
}

impl Street {
    pub fn new(osm_name: &str, ref_name: &str, show_ref_street: bool, osm_id: u64) -> Street {
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
    pub fn from_string(osm_name: &str) -> Street {
        Street::new(osm_name, "", true, 0)
    }

    /// Returns the OSM name.
    pub fn get_osm_name(&self) -> &String {
        &self.osm_name
    }

    /// Returns the reference name.
    fn get_ref_name(&self) -> &String {
        &self.ref_name
    }

    /// Returns the OSM (way) id.
    pub fn get_osm_id(&self) -> u64 {
        self.osm_id
    }

    /// Sets the OSM type, e.g. 'way'.
    pub fn set_osm_type(&mut self, osm_type: &str) {
        self.osm_type = osm_type.into()
    }

    /// Returns the OSM type, e.g. 'way'.
    pub fn get_osm_type(&self) -> &String {
        &self.osm_type
    }

    /// Sets the source of this street.
    pub fn set_source(&mut self, source: &str) {
        self.source = source.into()
    }

    /// Gets the source of this street.
    fn get_source(&self) -> &str {
        &self.source
    }

    /// Writes the street as a HTML string.
    pub fn to_html(&self) -> yattag::Doc {
        let doc = yattag::Doc::new();
        doc.text(&self.osm_name);
        if self.osm_name != self.ref_name && self.show_ref_street {
            doc.stag("br", &[]);
            doc.text("(");
            doc.text(&self.ref_name);
            doc.text(")");
        }
        doc
    }
}

impl Ord for Street {
    /// OSM id is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.osm_name.cmp(&other.osm_name)
    }
}

impl PartialOrd for Street {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Diff for Street {
    fn get_diff_key(&self) -> String {
        let re = regex::Regex::new(r"\*$").unwrap();
        re.replace(&self.osm_name, "").to_string()
    }
}

impl PartialEq for Street {
    /// OSM id is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.osm_name == other.osm_name
    }
}

impl Eq for Street {}

impl Hash for Street {
    /// OSM id is explicitly not interesting.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.osm_name.hash(state);
    }
}

#[pyclass]
#[derive(Debug)]
pub struct PyStreet {
    pub street: Street,
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

    fn to_html(&self) -> yattag::PyDoc {
        let doc = self.street.to_html();
        yattag::PyDoc { doc }
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
#[derive(Clone, Debug)]
pub struct HouseNumber {
    number: String,
    source: String,
    comment: String,
}

pub type HouseNumbers = Vec<HouseNumber>;
pub type NumberedStreet = (Street, HouseNumbers);
pub type NumberedStreets = Vec<NumberedStreet>;

impl HouseNumber {
    pub fn new(number: &str, source: &str, comment: &str) -> Self {
        HouseNumber {
            number: number.into(),
            source: source.into(),
            comment: comment.into(),
        }
    }

    /// Returns the house number string.
    pub fn get_number(&self) -> &str {
        &self.number
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
    pub fn is_invalid(house_number: &str, invalids: &[String]) -> bool {
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
    pub fn has_letter_suffix(house_number: &str, source_suffix: &str) -> bool {
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
    pub fn normalize_letter_suffix(
        house_number: &str,
        source_suffix: &str,
        style: LetterSuffixStyle,
    ) -> anyhow::Result<String> {
        let mut house_number: String = house_number.into();
        if !source_suffix.is_empty() {
            house_number = house_number[..house_number.len() - source_suffix.len()].into();
        }
        // Check for letter suffix.
        let is_match = NUMBER_PER_LETTER.is_match(&house_number);
        let mut digit_match = false;
        let mut groups: Vec<String> = Vec::new();
        if is_match {
            if let Some(cap) = NUMBER_PER_LETTER.captures_iter(&house_number).next() {
                for index in 1..=3 {
                    match cap.get(index) {
                        Some(_) => groups.push(cap[index].to_string()),
                        None => groups.push(String::from("")),
                    }
                }
            }
        } else {
            // If not, then try digit suggfix, but then only '/' is OK as a separator.
            let is_match = NUMBER_PER_NUMBER.is_match(&house_number);
            digit_match = true;
            if !is_match {
                return Err(anyhow!("ValueError"));
            }
            if let Some(cap) = NUMBER_PER_NUMBER.captures_iter(&house_number).next() {
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

impl Diff for HouseNumber {
    fn get_diff_key(&self) -> String {
        if self.number.ends_with('*') {
            let mut chars = self.number.chars();
            chars.next_back();
            return chars.as_str().into();
        }

        self.number.clone()
    }
}

impl PartialEq for HouseNumber {
    /// Source is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Eq for HouseNumber {}

impl Hash for HouseNumber {
    /// Source is explicitly non-interesting.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

#[pyclass]
#[derive(Debug)]
pub struct PyHouseNumber {
    pub house_number: HouseNumber,
}

type PyHouseNumbers = Vec<PyHouseNumber>;
type PyNumberedStreet = (PyStreet, PyHouseNumbers);
pub type PyNumberedStreets = Vec<PyNumberedStreet>;

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
pub struct CsvRead<'a> {
    reader: csv::Reader<&'a mut dyn Read>,
}

impl<'a> CsvRead<'a> {
    pub fn new(read: &'a mut dyn Read) -> Self {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .double_quote(true)
            .from_reader(read);
        CsvRead { reader }
    }

    /// Gets access to the rows of the CSV.
    pub fn records(&mut self) -> csv::StringRecordsIter<'_, &'a mut dyn Read> {
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
pub fn split_house_number(house_number: &str) -> (i32, String) {
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
pub fn split_house_number_range(house_number: &HouseNumberRange) -> (i32, String) {
    split_house_number(house_number.get_number())
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
pub fn format_even_odd_html(only_in_ref: &[HouseNumberRange]) -> yattag::Doc {
    let mut even: Vec<HouseNumberRange> = Vec::new();
    let mut odd: Vec<HouseNumberRange> = Vec::new();
    separate_even_odd(only_in_ref, &mut even, &mut odd);
    let doc = yattag::Doc::new();
    for (index, elem) in odd.iter().enumerate() {
        if index > 0 {
            doc.text(", ");
        }
        doc.append_value(color_house_number(elem).get_value());
    }
    if !even.is_empty() && !odd.is_empty() {
        doc.stag("br", &[]);
    }
    for (index, elem) in even.iter().enumerate() {
        if index > 0 {
            doc.text(", ");
        }
        doc.append_value(color_house_number(elem).get_value());
    }
    doc
}

/// Colors a house number according to its suffix.
pub fn color_house_number(house_number: &HouseNumberRange) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let number = house_number.get_number();
    if !number.ends_with('*') {
        doc.text(number);
        return doc;
    }
    let mut chars = number.chars();
    chars.next_back();
    let number = chars.as_str();
    let title = house_number.get_comment().replace("&#013;", "\n");
    let _span = doc.tag("span", &[("style", "color: blue;")]);
    if !title.is_empty() {
        {
            let _abbr = doc.tag("abbr", &[("title", title.as_str()), ("tabindex", "0")]);
            doc.text(number);
        }
    } else {
        doc.text(number);
    }
    doc
}

/// refcounty -> refsettlement -> streets cache.
type StreetReferenceCache = HashMap<String, HashMap<String, Vec<String>>>;

/// Builds an in-memory cache from the reference on-disk TSV (street version).
pub fn build_street_reference_cache(local_streets: &str) -> anyhow::Result<StreetReferenceCache> {
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
pub type HouseNumberReferenceCache =
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
pub fn build_reference_caches(
    references: &[String],
    refcounty: &str,
) -> anyhow::Result<Vec<HouseNumberReferenceCache>> {
    references
        .iter()
        .map(|reference| build_reference_cache(reference, refcounty))
        .collect()
}

/// Parses a filter description, like 'filter-for', 'refcounty', '42'.
fn parse_filters(tokens: &[String]) -> HashMap<String, String> {
    let mut ret: HashMap<String, String> = HashMap::new();
    let mut filter_for = false;
    for (index, value) in tokens.iter().enumerate() {
        if value == "filter-for" {
            filter_for = true;
            continue;
        }

        if !filter_for {
            continue;
        }

        if value == "incomplete" || value == "everything" {
            ret.insert(value.clone(), "".into());
        }

        if index + 1 >= tokens.len() {
            continue;
        }

        if vec!["refcounty", "refsettlement", "relations"].contains(&value.as_str()) {
            ret.insert(value.clone(), tokens[index + 1].clone());
        }
    }
    ret
}

#[pyfunction]
fn py_parse_filters(tokens: Vec<String>) -> HashMap<String, String> {
    parse_filters(&tokens)
}

/// Handles a HTTP error from Overpass.
fn handle_overpass_error(ctx: &context::Context, http_error: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let _div = doc.tag("div", &[("id", "overpass-error")]);
    doc.text(&tr("Overpass error: {0}").replace("{0}", http_error));
    let sleep = overpass_query::overpass_query_need_sleep(ctx);
    if sleep > 0 {
        doc.stag("br", &[]);
        doc.text(&tr("Note: wait for {} seconds").replace("{}", &sleep.to_string()));
    }
    doc
}

#[pyfunction]
fn py_handle_overpass_error(
    py: Python<'_>,
    ctx: PyObject,
    http_error: String,
) -> PyResult<yattag::PyDoc> {
    let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(py)?;
    let doc = handle_overpass_error(&ctx.context, &http_error);
    Ok(yattag::PyDoc { doc })
}

/// Provides localized strings for this thread.
fn setup_localization(headers: &[(String, String)]) -> String {
    let mut languages: String = "".into();
    for (key, value) in headers {
        if key == "HTTP_ACCEPT_LANGUAGE" {
            languages = value.into();
        }
    }
    if !languages.is_empty() {
        let parsed = accept_language::parse(&languages);
        if !parsed.is_empty() {
            let language = parsed[0].clone();
            if i18n::set_language(&language).is_err() {
                return "".into();
            }
            return language;
        }
    }
    "".into()
}

#[pyfunction]
fn py_setup_localization(headers: Vec<(String, String)>) -> String {
    setup_localization(&headers)
}

/// Generates a link to a URL with a given label.
fn gen_link(url: &str, label: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let _a = doc.tag("a", &[("href", url)]);
    doc.text(&(label.to_string() + "..."));
    doc
}

#[pyfunction]
fn py_gen_link(url: String, label: String) -> yattag::PyDoc {
    yattag::PyDoc {
        doc: gen_link(&url, &label),
    }
}

/// Produces the verify first line of a HTML output.
pub fn write_html_header(doc: &yattag::Doc) {
    doc.append_value("<!DOCTYPE html>\n".into())
}

#[pyfunction]
fn py_write_html_header(py: Python<'_>, doc: PyObject) -> PyResult<()> {
    let doc: PyRefMut<'_, yattag::PyDoc> = doc.extract(py)?;
    write_html_header(&doc.doc);
    Ok(())
}

/// Turns an overpass query template to an actual query.
pub fn process_template(buf: &str, osm_relation: u64) -> String {
    let mut buf = buf.replace("@RELATION@", &osm_relation.to_string());
    // area is relation + 3600000000 (3600000000 == relation), see js/ide.js
    // in https://github.com/tyrasd/overpass-turbo
    buf = buf.replace("@AREA@", &(3600000000 + osm_relation).to_string());
    buf
}

#[pyfunction]
fn py_process_template(buf: String, osm_relation: u64) -> String {
    process_template(&buf, osm_relation)
}

/// Decides if an x-y range should be expanded. Returns a sanitized end value as well.
pub fn should_expand_range(numbers: &[i64], street_is_even_odd: bool) -> (bool, i64) {
    if numbers.len() != 2 {
        return (false, 0);
    }

    if numbers[1] < numbers[0] {
        // E.g. 42-1, -1 is just a suffix to be ignored.
        return (true, 0);
    }

    // If there is a parity mismatch, ignore.
    if street_is_even_odd && numbers[0] % 2 != numbers[1] % 2 {
        return (false, 0);
    }

    // Assume that 0 is just noise.
    if numbers[0] == 0 {
        return (false, 0);
    }

    // Ranges larger than this are typically just noise in the input data.
    if numbers[1] > 1000 || numbers[1] - numbers[0] > 24 {
        return (false, 0);
    }

    (true, numbers[1])
}

/// Produces a HTML table from a list of lists.
pub fn html_table_from_list(table: &[Vec<yattag::Doc>]) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let _table = doc.tag("table", &[("class", "sortable")]);
    for (row_index, row_content) in table.iter().enumerate() {
        let _tr = doc.tag("tr", &[]);
        for cell in row_content {
            if row_index == 0 {
                let _th = doc.tag("th", &[]);
                let _a = doc.tag("a", &[("href", "#")]);
                doc.text(&cell.get_value());
            } else {
                let _td = doc.tag("td", &[]);
                doc.append_value(cell.get_value())
            }
        }
    }
    doc
}

#[pyfunction]
fn py_html_table_from_list(py: Python<'_>, table: Vec<Vec<PyObject>>) -> PyResult<yattag::PyDoc> {
    // Convert Vec<Vec<PyObject>> to Vec<Vec<yattag::Doc>>.
    let mut native_table: Vec<Vec<yattag::Doc>> = Vec::new();
    for row in table {
        let mut native_row: Vec<yattag::Doc> = Vec::new();
        for cell in row {
            let cell: PyRefMut<'_, yattag::PyDoc> = cell.extract(py)?;
            native_row.push(cell.doc.clone());
        }
        native_table.push(native_row);
    }
    let doc = html_table_from_list(&native_table);
    Ok(yattag::PyDoc { doc })
}

/// Produces HTML enumerations for 2 string lists.
pub fn invalid_refstreets_to_html(osm_invalids: &[String], ref_invalids: &[String]) -> yattag::Doc {
    let doc = yattag::Doc::new();
    if !osm_invalids.is_empty() {
        doc.stag("br", &[]);
        let _div = doc.tag("div", &[("id", "osm-invalids-container")]);
        doc.text(&tr(
            "Warning: broken OSM <-> reference mapping, the following OSM names are invalid:",
        ));
        let _ul = doc.tag("ul", &[]);
        for osm_invalid in osm_invalids {
            let _li = doc.tag("li", &[]);
            doc.text(osm_invalid);
        }
    }
    if !ref_invalids.is_empty() {
        doc.stag("br", &[]);
        let _div = doc.tag("div", &[("id", "ref-invalids-container")]);
        doc.text(&tr(
            "Warning: broken OSM <-> reference mapping, the following reference names are invalid:",
        ));
        let _ul = doc.tag("ul", &[]);
        for ref_invalid in ref_invalids {
            let _li = doc.tag("li", &[]);
            doc.text(ref_invalid);
        }
    }
    if !osm_invalids.is_empty() || !ref_invalids.is_empty() {
        doc.stag("br", &[]);
        doc.text(&tr(
            "Note: an OSM name is invalid if it's not in the OSM database.",
        ));
        doc.text(&tr(
            "A reference name is invalid if it's in the OSM database.",
        ));
    }
    doc
}

#[pyfunction]
fn py_invalid_refstreets_to_html(
    osm_invalids: Vec<String>,
    ref_invalids: Vec<String>,
) -> yattag::PyDoc {
    let doc = invalid_refstreets_to_html(&osm_invalids, &ref_invalids);
    yattag::PyDoc { doc }
}

/// Produces HTML enumerations for a string list.
pub fn invalid_filter_keys_to_html(invalids: &[String]) -> yattag::Doc {
    let doc = yattag::Doc::new();
    if !invalids.is_empty() {
        doc.stag("br", &[]);
        let _div = doc.tag("div", &[("id", "osm-filter-key-invalids-container")]);
        doc.text(&tr(
            "Warning: broken filter key name, the following key names are not OSM names:",
        ));
        let _ul = doc.tag("ul", &[]);
        for invalid in invalids {
            let _li = doc.tag("li", &[]);
            doc.text(invalid);
        }
    }
    doc
}

#[pyfunction]
fn py_invalid_filter_keys_to_html(invalids: Vec<String>) -> yattag::PyDoc {
    let doc = invalid_filter_keys_to_html(&invalids);
    yattag::PyDoc { doc }
}

/// Gets the nth column of row.
fn get_column(row: &[yattag::Doc], column_index: usize) -> String {
    let ret: String;
    if column_index >= row.len() {
        ret = row[0].get_value();
    } else {
        ret = row[column_index].get_value();
    }
    ret
}

#[pyfunction]
fn py_get_column(py: Python<'_>, row: Vec<PyObject>, column_index: usize) -> PyResult<String> {
    // Convert Vec<PyObject> to Vec<yattag::Doc>.
    let row: Vec<yattag::Doc> = row
        .iter()
        .map(|item| {
            let item: PyRefMut<'_, yattag::PyDoc> = item.extract(py)?;
            Ok(item.doc.clone())
        })
        .collect::<PyResult<Vec<yattag::Doc>>>()?;
    Ok(get_column(&row, column_index))
}

/// Interpret the content as an integer.
fn natnum(column: &str) -> u64 {
    let mut number: String = "".into();
    if let Some(cap) = NUMBER_WITH_JUNK.captures_iter(column).next() {
        number = cap[1].into();
    }
    number.parse::<u64>().unwrap_or(0)
}

#[pyfunction]
fn py_natnum(column: String) -> u64 {
    natnum(&column)
}

/// Turns a tab-separated table into a list of lists.
fn tsv_to_list(csv_read: &mut CsvRead<'_>) -> anyhow::Result<Vec<Vec<yattag::Doc>>> {
    let mut table: Vec<Vec<yattag::Doc>> = Vec::new();

    let mut first = true;
    let mut columns: HashMap<String, usize> = HashMap::new();
    for result in csv_read.records() {
        let row = result?;
        if first {
            first = false;
            for (index, label) in row.iter().enumerate() {
                columns.insert(label.into(), index);
            }
        }
        let mut cells: Vec<yattag::Doc> = row
            .iter()
            .map(|cell| yattag::Doc::from_text(cell))
            .collect();
        if !cells.is_empty() && columns.contains_key("@type") {
            // We know the first column is an OSM ID.
            if let Ok(osm_id) = cells[0].get_value().parse::<u64>() {
                let osm_type = cells[columns["@type"]].get_value();
                let doc = yattag::Doc::new();
                let href = format!("https://www.openstreetmap.org/{}/{}", osm_type, osm_id);
                {
                    let _a = doc.tag("a", &[("href", href.as_str()), ("target", "_blank")]);
                    doc.text(&osm_id.to_string());
                }
                cells[0] = doc;
            }
        }
        table.push(cells);
    }

    if columns.contains_key("addr:street") && columns.contains_key("addr:housenumber") {
        let header = table[0].clone();
        table.remove(0);
        //table.sort(key=lambda row: natnum(get_column(row, columns["addr:housenumber"])));
        table.sort_by(|a, b| {
            let a_key = natnum(&get_column(a, *columns.get("addr:housenumber").unwrap()));
            let b_key = natnum(&get_column(b, *columns.get("addr:housenumber").unwrap()));
            a_key.cmp(&b_key)
        });
        table.sort_by(|a, b| {
            let a_key = natnum(&get_column(a, *columns.get("addr:street").unwrap()));
            let b_key = natnum(&get_column(b, *columns.get("addr:street").unwrap()));
            a_key.cmp(&b_key)
        });
        let mut merged = vec![header];
        merged.append(&mut table);
        table = merged;
    }

    Ok(table)
}

#[pyfunction]
fn py_tsv_to_list(py: Python<'_>, stream: PyObject) -> PyResult<Vec<Vec<yattag::PyDoc>>> {
    let mut stream: PyRefMut<'_, PyCsvRead> = stream.extract(py)?;
    let mut cursor = std::io::Cursor::new(&mut stream.buf);
    let mut csv_read = CsvRead::new(&mut cursor);
    let ret = match tsv_to_list(&mut csv_read) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "tsv_to_list() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(ret
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| yattag::PyDoc { doc: cell.clone() })
                .collect::<Vec<yattag::PyDoc>>()
        })
        .collect::<Vec<Vec<yattag::PyDoc>>>())
}

/// Reads a house number CSV and extracts streets from rows.
/// Returns a list of street objects, with their name, ID and type set.
pub fn get_street_from_housenumber(csv_read: &mut CsvRead<'_>) -> anyhow::Result<Vec<Street>> {
    let mut ret: Vec<Street> = Vec::new();

    let mut first = true;
    let mut columns: HashMap<String, usize> = HashMap::new();
    for result in csv_read.records() {
        let row = result?;
        if first {
            first = false;
            for (index, label) in row.iter().enumerate() {
                columns.insert(label.into(), index);
            }
            continue;
        }

        let housenumber_col = *match columns.get("addr:housenumber") {
            Some(value) => value,
            None => {
                // data/street-housenumbers-template.txt requests this, so we got garbage, give up.
                return Err(anyhow::anyhow!("missing addr:housenumber column in CSV"));
            }
        };

        let has_housenumber = &row[housenumber_col];
        let has_conscriptionnumber = &row[*columns.get("addr:conscriptionnumber").unwrap()];
        if has_housenumber.is_empty() && has_conscriptionnumber.is_empty() {
            continue;
        }

        let mut street_name = &row[*columns.get("addr:street").unwrap()];
        if street_name.is_empty() && columns.contains_key("addr:place") {
            street_name = &row[*columns.get("addr:place").unwrap()];
        }
        if street_name.is_empty() {
            continue;
        }

        let osm_type = &row[*columns.get("@type").unwrap()];
        let osm_id = row[0].parse::<u64>().unwrap_or(0);
        let mut street = Street::new(street_name, "", true, osm_id);
        street.set_osm_type(osm_type);
        street.set_source(&tr("housenumber"));
        ret.push(street);
    }

    Ok(ret)
}

#[pyfunction]
fn py_get_street_from_housenumber(py: Python<'_>, stream: PyObject) -> PyResult<Vec<PyStreet>> {
    let mut stream: PyRefMut<'_, PyCsvRead> = stream.extract(py)?;
    let mut cursor = std::io::Cursor::new(&mut stream.buf);
    let mut csv_read = CsvRead::new(&mut cursor);
    let ret = match get_street_from_housenumber(&mut csv_read) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_street_from_housenumber() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(ret
        .iter()
        .map(|street| PyStreet {
            street: street.clone(),
        })
        .collect::<Vec<PyStreet>>())
}

/// Gets a reference range list for a house number list by looking at what range provided a given
/// house number.
pub fn get_housenumber_ranges(house_numbers: &[HouseNumber]) -> Vec<HouseNumberRange> {
    let mut ret: Vec<HouseNumberRange> = Vec::new();
    for house_number in house_numbers {
        ret.push(HouseNumberRange::new(
            house_number.get_source(),
            house_number.get_comment(),
        ));
    }
    ret.sort();
    ret.dedup();
    ret
}

#[pyfunction]
fn py_get_housenumber_ranges(
    py: Python<'_>,
    house_numbers: Vec<PyObject>,
) -> PyResult<Vec<PyHouseNumberRange>> {
    let house_numbers: Vec<HouseNumber> = house_numbers
        .iter()
        .map(|house_number| {
            let house_number: PyRefMut<'_, PyHouseNumber> = house_number.extract(py)?;
            Ok(house_number.house_number.clone())
        })
        .collect::<PyResult<Vec<HouseNumber>>>()?;
    let ret = get_housenumber_ranges(&house_numbers);
    Ok(ret
        .iter()
        .map(|house_number_range| PyHouseNumberRange {
            house_number_range: house_number_range.clone(),
        })
        .collect())
}

/// Generates a HTML link based on a website prefix and a git-describe version.
pub fn git_link(version: &str, prefix: &str) -> yattag::Doc {
    let mut commit_hash: String = "".into();
    if let Some(cap) = GIT_HASH.captures_iter(version).next() {
        commit_hash = cap[1].into();
    }
    let doc = yattag::Doc::new();
    let _a = doc.tag(
        "a",
        &[("href", (prefix.to_string() + &commit_hash).as_str())],
    );
    doc.text(version);
    doc
}

#[pyfunction]
fn py_git_link(version: String, prefix: String) -> yattag::PyDoc {
    let doc = git_link(&version, &prefix);
    yattag::PyDoc { doc }
}

/// Sorts strings according to their numerical value, not alphabetically.
pub fn sort_numerically(strings: &[HouseNumber]) -> Vec<HouseNumber> {
    let mut ret: Vec<HouseNumber> = strings.to_owned();
    ret.sort_by(|a, b| {
        let a_key = split_house_number(a.get_number());
        let b_key = split_house_number(b.get_number());
        a_key.cmp(&b_key)
    });
    ret
}

#[pyfunction]
fn py_sort_numerically(py: Python<'_>, strings: Vec<PyObject>) -> PyResult<Vec<PyHouseNumber>> {
    let mut house_numbers: Vec<HouseNumber> = strings
        .iter()
        .map(|item| {
            let item: PyRefMut<'_, PyHouseNumber> = item.extract(py)?;
            Ok(item.house_number.clone())
        })
        .collect::<PyResult<Vec<HouseNumber>>>()?;
    house_numbers = sort_numerically(&house_numbers);
    Ok(house_numbers
        .iter()
        .map(|house_number| PyHouseNumber {
            house_number: house_number.clone(),
        })
        .collect())
}

/// Returns items which are in first, but not in second.
pub fn get_only_in_first<T: Clone + Diff>(first: &[T], second: &[T]) -> Vec<T> {
    if first.is_empty() {
        return Vec::new();
    }

    // Strip suffix that is ignored.
    let second: Vec<String> = second.iter().map(|i| i.get_diff_key()).collect();

    first
        .iter()
        .filter(|i| !second.contains(&i.get_diff_key()))
        .cloned()
        .collect()
}

/// Returns items which are in both first and second.
pub fn get_in_both<T: Clone + Diff>(first: &[T], second: &[T]) -> Vec<T> {
    if first.is_empty() {
        return Vec::new();
    }

    // Strip suffix that is ignored.
    let second: Vec<String> = second.iter().map(|i| i.get_diff_key()).collect();

    first
        .iter()
        .filter(|i| second.contains(&i.get_diff_key()))
        .cloned()
        .collect()
}

/// Gets the content of a file in workdir.
fn get_content(path: &str) -> anyhow::Result<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

#[pyfunction]
fn py_get_content(py: Python<'_>, path: String) -> PyResult<PyObject> {
    let buf = match get_content(&path) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_content() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(PyBytes::new(py, &buf).into())
}

type HttpHeaders = Vec<(String, String)>;

/// Gets the content of a file in workdir with metadata.
pub fn get_content_with_meta(path: &str) -> anyhow::Result<(Vec<u8>, HttpHeaders)> {
    let buf = get_content(path)?;

    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    let modified_utc: chrono::DateTime<chrono::offset::Utc> = modified.into();

    let extra_headers = vec![("Last-Modified".to_string(), modified_utc.to_rfc2822())];
    Ok((buf, extra_headers))
}

/// Determines the normalizer for a given street.
pub fn get_normalizer(
    street_name: &str,
    normalizers: &HashMap<String, ranges::Ranges>,
) -> ranges::Ranges {
    let normalizer: ranges::Ranges;
    if let Some(value) = normalizers.get(street_name) {
        // Have a custom filter.
        normalizer = value.clone();
    } else {
        // Default sanity checks.
        let default = vec![
            ranges::Range::new(1, 999, ""),
            ranges::Range::new(2, 998, ""),
        ];
        normalizer = ranges::Ranges::new(default);
    }
    normalizer
}

/// Splits a house number string (possibly a range) by a given separator.
/// Returns a filtered and a not filtered list of ints.
pub fn split_house_number_by_separator(
    house_numbers: &str,
    separator: &str,
    normalizer: &ranges::Ranges,
) -> (Vec<i64>, Vec<i64>) {
    let mut ret_numbers: Vec<i64> = Vec::new();
    // Same as ret_numbers, but if the range is 2-6 and we filter for 2-4, then 6 would be lost, so
    // in-range 4 would not be detected, so this one does not drop 6.
    let mut ret_numbers_nofilter: Vec<i64> = Vec::new();

    for house_number in house_numbers.split(separator) {
        let mut number: i64 = 0;
        if let Some(cap) = NUMBER_WITH_JUNK.captures_iter(house_number).next() {
            match cap[1].parse::<i64>() {
                Ok(value) => number = value,
                Err(_) => {
                    continue;
                }
            }
        }

        ret_numbers_nofilter.push(number);

        if !normalizer.contains(number) {
            continue;
        }

        ret_numbers.push(number);
    }

    (ret_numbers, ret_numbers_nofilter)
}

/// Constructs a city name based on postcode the nominal city.
fn get_city_key(
    postcode: &str,
    city: &str,
    valid_settlements: &HashSet<String>,
) -> anyhow::Result<String> {
    let city = city.to_lowercase();

    if !city.is_empty() && postcode.starts_with('1') {
        let mut chars = postcode.chars();
        chars.next();
        chars.next_back();
        let district = chars.as_str().parse::<i32>()?;
        if (1..=23).contains(&district) {
            return Ok(city + "_" + chars.as_str());
        }
        return Ok(city);
    }

    if valid_settlements.contains(&city) || city == "budapest" {
        return Ok(city);
    }
    if !city.is_empty() {
        return Ok("_Invalid".into());
    }
    Ok("_Empty".into())
}

#[pyfunction]
fn py_get_city_key(
    postcode: String,
    city: String,
    valid_settlements: HashSet<String>,
) -> PyResult<String> {
    match get_city_key(&postcode, &city, &valid_settlements) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "get_city_key() failed: {}",
            err.to_string()
        ))),
    }
}

/// Returns a string comparator which allows locale-aware lexical sorting.
pub fn get_sort_key(bytes: &str) -> anyhow::Result<Vec<u8>> {
    // This is good enough for now, English and Hungarian is all we support and this handles both.
    let collator = ucol::UCollator::try_from("hu")?;
    let string = ustring::UChar::try_from(bytes)?;
    Ok(collator.get_sort_key(&string))
}

#[pyfunction]
fn py_get_sort_key(py: Python<'_>, bytes: String) -> PyResult<PyObject> {
    let buf = match get_sort_key(&bytes) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_sort_key() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(PyBytes::new(py, &buf).into())
}

/// Builds a set of valid settlement names.
fn get_valid_settlements(ctx: &context::Context) -> anyhow::Result<HashSet<String>> {
    let mut settlements: HashSet<String> = HashSet::new();

    let path = ctx.get_ini().get_reference_citycounts_path()?;
    let stream = ctx.get_file_system().open_read(&path)?;
    let mut guard = stream.lock().unwrap();
    let mut read = guard.deref_mut();
    let mut csv_read = CsvRead::new(&mut read);
    let mut first = true;
    for result in csv_read.records() {
        if first {
            first = false;
            continue;
        }

        let record = match result {
            Ok(value) => value,
            Err(_) => {
                continue;
            }
        };
        if let Some(col) = record.iter().next() {
            settlements.insert(col.into());
        }
    }

    Ok(settlements)
}

#[pyfunction]
fn py_get_valid_settlements(py: Python<'_>, ctx: PyObject) -> PyResult<HashSet<String>> {
    let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(py)?;
    match get_valid_settlements(&ctx.context) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "get_valid_settlements() failed: {}",
            err.to_string()
        ))),
    }
}

/// Formats a percentage, taking locale into account.
pub fn format_percent(english: &str) -> anyhow::Result<String> {
    let parsed: f64 = english.parse()?;
    let formatted = format!("{0:.2}%", parsed);
    let language: &str = &i18n::get_language();
    let decimal_point = match language {
        "hu" => ",",
        _ => ".",
    };
    Ok(formatted.replace(".", decimal_point))
}

#[pyfunction]
fn py_format_percent(english: String) -> PyResult<String> {
    match format_percent(&english) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "format_percent() failed: {}",
            err.to_string()
        ))),
    }
}

/// Gets the timestamp of a file if it exists, 0 otherwise.
fn get_timestamp(path: &str) -> f64 {
    let metadata = match std::fs::metadata(path) {
        Ok(value) => value,
        Err(_) => {
            return 0.0;
        }
    };
    let modified = match metadata.modified() {
        Ok(value) => value,
        Err(_) => {
            return 0.0;
        }
    };
    let mtime = match modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(value) => value,
        Err(_) => {
            return 0.0;
        }
    };
    mtime.as_secs_f64()
}

#[pyfunction]
fn py_get_timestamp(path: String) -> f64 {
    get_timestamp(&path)
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyHouseNumber>()?;
    module.add_class::<PyHouseNumberRange>()?;
    module.add_class::<PyLetterSuffixStyle>()?;
    module.add_class::<PyStreet>()?;
    module.add_class::<PyCsvRead>()?;
    module.add_function(pyo3::wrap_pyfunction!(py_split_house_number, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_format_even_odd, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_build_street_reference_cache,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_reference_cache_path, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_build_reference_cache, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_parse_filters, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_overpass_error, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_setup_localization, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_gen_link, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_write_html_header, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_process_template, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_html_table_from_list, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_invalid_refstreets_to_html,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_invalid_filter_keys_to_html,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_column, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_natnum, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_tsv_to_list, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_get_street_from_housenumber,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_housenumber_ranges, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_git_link, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_sort_numerically, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_content, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_city_key, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_sort_key, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_valid_settlements, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_format_percent, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_timestamp, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convers a string list into a street list.
    fn street_list(streets: &[&str]) -> Vec<Street> {
        streets.iter().map(|i| Street::from_string(i)).collect()
    }

    /// Tests get_only_in_first().
    #[test]
    fn test_only_in_first() {
        let ret = get_only_in_first(
            &street_list(&vec!["1", "2", "3"]),
            &street_list(&vec!["3", "4"]),
        );
        let names: Vec<_> = ret.iter().map(|i| i.get_osm_name()).collect();
        assert_eq!(names, vec!["1", "2"]);
    }

    /// Tests get_in_both().
    #[test]
    fn test_get_in_both() {
        let ret = get_in_both(
            &street_list(&vec!["1", "2", "3"]),
            &street_list(&vec!["2", "3", "4"]),
        );
        let names: Vec<_> = ret.iter().map(|i| i.get_osm_name()).collect();
        assert_eq!(names, vec!["2", "3"]);
    }

    /// Converts a string list into a house number range list.
    fn hnr_list(ranges: Vec<&str>) -> Vec<HouseNumberRange> {
        ranges
            .iter()
            .map(|i| HouseNumberRange::new(i, ""))
            .collect()
    }

    /// Tests format_even_odd().
    #[test]
    fn test_format_even_odd() {
        let expected = vec!["1".to_string(), "2".to_string()];
        assert_eq!(format_even_odd(&hnr_list(vec!["1", "2"])), expected);
    }

    /// Tests format_even_odd(): when we have odd numbers only.
    #[test]
    fn test_format_even_odd_only_odd() {
        let expected = vec!["1, 3".to_string()];
        assert_eq!(format_even_odd(&hnr_list(vec!["1", "3"])), expected);
    }

    /// Tests format_even_odd(): when we have even numbers only.
    #[test]
    fn test_format_even_odd_only_even() {
        let expected = vec!["2, 4".to_string()];
        assert_eq!(format_even_odd(&hnr_list(vec!["2", "4"])), expected);
    }

    /// Tests format_even_odd(): HTML coloring.
    #[test]
    fn test_format_even_odd_html() {
        let doc = format_even_odd_html(&hnr_list(vec!["2*", "4"]));
        let expected = r#"<span style="color: blue;">2</span>, 4"#;
        assert_eq!(doc.get_value(), expected)
    }

    /// Tests format_even_odd(): HTML commenting.
    #[test]
    fn test_format_even_odd_html_comment() {
        let house_numbers = vec![
            HouseNumberRange::new("2*", "foo"),
            HouseNumberRange::new("4", ""),
        ];
        let doc = format_even_odd_html(&house_numbers);
        let expected =
            r#"<span style="color: blue;"><abbr title="foo" tabindex="0">2</abbr></span>, 4"#;
        assert_eq!(doc.get_value(), expected);
    }

    /// Tests format_even_odd(): HTML output with multiple odd numbers.
    #[test]
    fn test_format_even_odd_html_multi_odd() {
        let doc = format_even_odd_html(&hnr_list(vec!["1", "3"]));
        assert_eq!(doc.get_value(), "1, 3".to_string());
    }

    /// Tests build_street_reference_cache().
    #[test]
    fn test_build_street_reference_cache() {
        let refpath = "tests/refdir/utcak_20190514.tsv";
        let memory_cache = build_street_reference_cache(refpath).unwrap();
        let streets: Vec<String> = vec![
            "Törökugrató utca".into(),
            "Tűzkő utca".into(),
            "Ref Name 1".into(),
            "Only In Ref utca".into(),
            "Only In Ref Nonsense utca".into(),
            "Hamzsabégi út".into(),
        ];
        let mut settlement: HashMap<String, Vec<String>> = HashMap::new();
        settlement.insert("011".into(), streets);
        let mut expected: StreetReferenceCache = HashMap::new();
        expected.insert("01".into(), settlement);
        assert_eq!(memory_cache, expected);
    }
}
