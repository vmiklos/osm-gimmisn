/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The ranges module contains functionality related to the Ranges class.

use pyo3::class::basic::CompareOp;
use pyo3::class::PyObjectProtocol;
use pyo3::class::PySequenceProtocol;
use pyo3::prelude::*;

/// A range object represents an odd or even range of integer numbers.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    start: i64,
    end: i64,
    is_odd: Option<bool>,
}

impl Range {
    pub fn new(start: i64, end: i64, interpolation: &str) -> Self {
        let mut is_odd = Some(start % 2 == 1);
        if interpolation == "all" {
            is_odd = None
        }
        Range { start, end, is_odd }
    }

    /// The smallest integer.
    fn get_start(&self) -> i64 {
        self.start
    }

    /// The largest integer.
    fn get_end(&self) -> i64 {
        self.end
    }

    /// None for all house numbers on one side, bool otherwise.
    fn is_odd(&self) -> Option<bool> {
        self.is_odd
    }

    fn contains(&self, item: i64) -> bool {
        if let Some(is_odd) = self.is_odd {
            if is_odd != (item % 2 == 1) {
                return false;
            }
        }

        if self.start <= item && item <= self.end {
            return true;
        }

        false
    }
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        if self.start != other.start {
            return false;
        }

        if self.end != other.end {
            return false;
        }

        if self.is_odd != other.is_odd {
            return false;
        }

        true
    }
}

#[pyclass]
#[derive(Debug)]
pub struct PyRange {
    range: Range,
}

#[pymethods]
impl PyRange {
    #[new]
    fn new(start: i64, end: i64, interpolation: String) -> Self {
        let range = Range::new(start, end, &interpolation);
        PyRange { range }
    }

    fn get_start(&self) -> i64 {
        self.range.get_start()
    }

    fn get_end(&self) -> i64 {
        self.range.get_end()
    }

    fn is_odd(&self) -> Option<bool> {
        self.range.is_odd()
    }
}

#[pyproto]
impl PySequenceProtocol for PyRange {
    fn __contains__(&self, item: i64) -> PyResult<bool> {
        Ok(self.range.contains(item))
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for PyRange {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    fn __richcmp__(&'p self, other: PyRef<'p, PyRange>, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(self.range.eq(&(*other).range).into_py(other.py())),
            _ => Ok(other.py().NotImplemented()),
        }
    }
}

/// A Ranges object contains an item if any of its Range objects contains it.
#[derive(Clone, Debug)]
pub struct Ranges {
    items: Vec<Range>,
}

impl Ranges {
    pub fn new(items: Vec<Range>) -> Self {
        Ranges { items }
    }

    /// The list of contained Range objects.
    fn get_items(&self) -> &Vec<Range> {
        &self.items
    }

    pub fn contains(&self, item: i64) -> bool {
        for i in &self.items {
            if i.contains(item) {
                return true;
            }
        }

        false
    }
}

impl PartialEq for Ranges {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

#[pyclass]
#[derive(Debug)]
pub struct PyRanges {
    pub ranges: Ranges,
}

#[pymethods]
impl PyRanges {
    #[new]
    fn new(py: Python<'_>, items: Vec<PyObject>) -> PyResult<Self> {
        // Convert Vec<PyObject> to Vec<Range>.
        let items: Vec<Range> = items
            .iter()
            .map(|item| {
                let item: PyRefMut<'_, PyRange> = item.extract(py)?;
                Ok(item.range)
            })
            .collect::<PyResult<Vec<Range>>>()?;
        let ranges = Ranges::new(items);

        Ok(PyRanges { ranges })
    }

    fn get_items(&self) -> Vec<PyRange> {
        // Convert Vec<Range> to Vec<PyRange>.
        let items: Vec<PyRange> = self
            .ranges
            .get_items()
            .iter()
            .map(|item| PyRange { range: *item })
            .collect();

        items
    }
}

#[pyproto]
impl PySequenceProtocol for PyRanges {
    fn __contains__(&self, item: i64) -> PyResult<bool> {
        Ok(self.ranges.contains(item))
    }
}

#[pyproto]
impl<'p> PyObjectProtocol<'p> for PyRanges {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    fn __richcmp__(&'p self, other: PyRef<'p, PyRanges>, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(self.ranges.eq(&(*other).ranges).into_py(other.py())),
            _ => Ok(other.py().NotImplemented()),
        }
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyRange>()?;
    module.add_class::<PyRanges>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Factory for Range without specifying interpolation.
    fn make_range(start: i64, end: i64) -> Range {
        Range::new(start, end, "".into())
    }

    /// Range: Tests an odd range with an even number.
    #[test]
    fn test_range_isodd_bad() {
        let test = make_range(1, 3);
        assert_eq!(test.contains(2), false);
    }

    /// Range: Tests an odd range with a large number.
    #[test]
    fn test_range_bad() {
        let test = make_range(1, 3);
        assert_eq!(test.contains(5), false);
    }

    /// Range: Tests the happy path.
    #[test]
    fn test_range_happy() {
        let test = make_range(1, 5);
        assert_eq!(test.contains(1), true);
        assert_eq!(test.contains(3), true);
        assert_eq!(test.contains(5), true);
        assert_eq!(test.get_start(), 1);
        assert_eq!(test.get_end(), 5);
    }

    /// Range: Tests equality code.
    #[test]
    fn test_range_eq() {
        assert_eq!(make_range(1, 5) != make_range(3, 5), true);
        assert_eq!(make_range(1, 5) != make_range(1, 3), true);
        assert_eq!(
            make_range(1, 3) != Range::new(1, 3, /*interpolation=*/ "all"),
            true
        );
    }

    /// Range: Tests the interpolation modes.
    #[test]
    fn test_range_interpolation_all() {
        assert_eq!(make_range(1, 3).contains(2), false);
        assert_eq!(Range::new(1, 3, /*interpolation=*/ "all").contains(2), true);
    }

    /// Ranges: Tests when the arg is in the first range.
    #[test]
    fn test_ranges_a() {
        let test = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
        assert_eq!(test.contains(0), true);
    }

    /// Ranges: Tests when the arg is in the second range.
    #[test]
    fn test_ranges_b() {
        let test = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
        assert_eq!(test.contains(1), true);
    }

    /// Ranges: Tests when the arg is in both ranges.
    #[test]
    fn test_ranges_ab() {
        let test = Ranges::new(vec![make_range(1, 1), make_range(1, 1)]);
        assert_eq!(test.contains(1), true);
    }

    /// Ranges: Tests when the arg is in neither ranges.
    #[test]
    fn test_ranges_none() {
        let test = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
        assert_eq!(test.contains(2), false);
    }
}
