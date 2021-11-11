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
    filter_data: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    let range_data = range_data.as_object().unwrap();

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
    filter_data: &serde_json::Map<String, serde_json::Value>,
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

/// Validates a range list.
fn validate_ranges(
    errors: &mut Vec<String>,
    parent: &str,
    ranges: &[serde_json::Value],
    filter_data: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    for (index, range_data) in ranges.iter().enumerate() {
        validate_range(
            errors,
            &format!("{}[{}]", parent, index),
            range_data,
            filter_data,
        )?;
    }

    Ok(())
}

/// Validates an 'invalid' or 'valid' list.
fn validate_filter_invalid_valid(
    errors: &mut Vec<String>,
    parent: &str,
    invalid: &serde_json::Value,
) -> anyhow::Result<()> {
    for (index, invalid_data) in invalid.as_array().unwrap().iter().enumerate() {
        if !invalid_data.is_string() {
            errors.push(format!(
                "expected value type for '{}[{}]' is str",
                parent, index
            ));
            continue;
        }
        let invalid_data = invalid_data.as_str().unwrap();
        if regex::Regex::new(r"^[0-9]+$")
            .unwrap()
            .is_match(invalid_data)
        {
            continue;
        }
        if regex::Regex::new(r"^[0-9]+[a-z]$")
            .unwrap()
            .is_match(invalid_data)
        {
            continue;
        }
        if regex::Regex::new(r"^[0-9]+/[0-9]$")
            .unwrap()
            .is_match(invalid_data)
        {
            continue;
        }
        errors.push(format!(
            "expected format for '{}[{}]' is '42', '42a' or '42/1'",
            parent, index
        ));
    }

    Ok(())
}

/// Validates a filter dictionary.
fn validate_filter(
    errors: &mut Vec<String>,
    parent: &str,
    filter_data: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    let context = format!("{}.", parent);
    for (key, value) in filter_data {
        if key == "ranges" {
            if !value.is_array() {
                errors.push(format!(
                    "expected value type for '{}{}' is list",
                    context, key
                ));
                continue;
            }
            validate_ranges(
                errors,
                &format!("{}ranges", context),
                value.as_array().unwrap(),
                filter_data,
            )?;
        } else if key == "invalid" || key == "valid" {
            if !value.is_array() {
                errors.push(format!(
                    "expected value type for '{}{}' is list",
                    context, key
                ));
                continue;
            }
            validate_filter_invalid_valid(errors, &format!("{}{}", context, key), value)?;
        } else if key == "refsettlement" || key == "interpolation" {
            if !value.is_string() {
                errors.push(format!(
                    "expected value type for '{}{}' is str",
                    context, key
                ));
            }
        } else if key == "show-refstreet" {
            if !value.is_boolean() {
                errors.push(format!(
                    "expected value type for '{}{}' is bool",
                    context, key
                ));
            }
        } else {
            errors.push(format!("unexpected key '{}{}'", context, key));
        }
    }

    Ok(())
}

/// Validates a filter list.
fn validate_filters(
    errors: &mut Vec<String>,
    parent: &str,
    filters: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    let context = format!("{}.", parent);
    for (key, value) in filters {
        validate_filter(
            errors,
            &format!("{}{}", context, key),
            value.as_object().unwrap(),
        )?;
    }

    Ok(())
}

#[pyfunction]
fn py_validate_filters(
    mut errors: Vec<String>,
    parent: &str,
    filters: &str,
) -> PyResult<Vec<String>> {
    let filters: serde_json::Value = serde_json::from_str(filters).unwrap();
    match validate_filters(&mut errors, parent, filters.as_object().unwrap())
        .context("validate_filters() failed")
    {
        Ok(_) => Ok(errors),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_validate_filters, module)?)?;
    Ok(())
}
