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
#[derive(Clone, Copy, Debug, FromPyObject)]
pub struct Range {
    start: i64,
    end: i64,
    is_odd: Option<bool>,
}

impl Range {
    fn new(start: i64, end: i64, interpolation: String) -> Self {
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
        let range = Range::new(start, end, interpolation);
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
#[derive(Debug, FromPyObject)]
pub struct Ranges {
    items: Vec<Range>,
}

impl Ranges {
    fn new(items: Vec<Range>) -> Self {
        Ranges { items }
    }

    /// The list of contained Range objects.
    fn get_items(&self) -> &Vec<Range> {
        &self.items
    }

    fn contains(&self, item: i64) -> bool {
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
    ranges: Ranges,
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
