/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The validator module validates yaml files under data/.

use anyhow::Context;
use pyo3::prelude::*;

/// Validates a range description: check for missing keys."""
fn validate_range_missing_keys(
    errors: &mut Vec<String>,
    parent: &str,
    range_data: &serde_json::Value,
    filter_data: &serde_json::Value,
) -> anyhow::Result<()> {
    let range_data = range_data.as_object().unwrap();
    let filter_data = filter_data.as_object().unwrap();

    if !range_data.contains_key("start") {
        errors.push(format!("unexpected missing key 'start' for '{}'", parent));
    }

    if !range_data.contains_key("end") {
        errors.push(format!("unexpected missing key 'end' for '{}'", parent));
    }

    if !range_data.contains_key("start") || !range_data.contains_key("end") {
        return Ok(());
    }

    let start = match range_data["start"].as_str() {
        Some(value) => value,
        None => {
            return Ok(());
        }
    };
    let start: i64 = match start.parse() {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!(
                "expected value type for '{}.start' is a digit str",
                parent
            ));
            return Ok(());
        }
    };
    let end = match range_data["end"].as_str() {
        Some(value) => value,
        None => {
            return Ok(());
        }
    };
    let end: i64 = match end.parse() {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!(
                "expected value type for '{}.end' is a digit str",
                parent
            ));
            return Ok(());
        }
    };
    if start > end {
        errors.push(format!("expected end >= start for '{}'", parent));
    }

    if !filter_data.contains_key("interpolation") && start % 2 != end % 2 {
        errors.push(format!("expected start % 2 == end % 2 for '{}'", parent))
    }

    Ok(())
}

/// Validates a range description.
fn validate_range(
    errors: &mut Vec<String>,
    parent: &str,
    range_data: &serde_json::Value,
    filter_data: &serde_json::Value,
) -> anyhow::Result<()> {
    let context = format!("{}.", parent);
    for (key, value) in range_data.as_object().unwrap() {
        if key == "start" {
            if !value.is_string() {
                errors.push(format!(
                    "expected value type for '{}{}' is a digit str",
                    context, key
                ));
            }
        } else if key == "end" {
            if !value.is_string() {
                errors.push(format!(
                    "expected value type for '{}{}' is a digit str",
                    context, key
                ))
            }
        } else if key == "refsettlement" {
            if !value.is_string() {
                errors.push(format!(
                    "expected value type for '{}{}' is str",
                    context, key
                ));
            }
        } else {
            errors.push(format!("unexpected key '{}{}'", context, key));
        }
    }
    validate_range_missing_keys(errors, parent, range_data, filter_data)?;
    Ok(())
}

#[pyfunction]
fn py_validate_range(
    mut errors: Vec<String>,
    parent: &str,
    range_data: &str,
    filter_data: &str,
) -> PyResult<Vec<String>> {
    let range_data: serde_json::Value = serde_json::from_str(range_data).unwrap();
    let filter_data: serde_json::Value = serde_json::from_str(filter_data).unwrap();
    match validate_range(&mut errors, parent, &range_data, &filter_data)
        .context("validate_range() failed")
    {
        Ok(_) => Ok(errors),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_validate_range, module)?)?;
    Ok(())
}
