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
use lazy_static::lazy_static;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::io::Write;

use crate::context;

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
    filters: &serde_json::Value,
) -> anyhow::Result<()> {
    let filters = filters.as_object().unwrap();
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

/// Validates a reference streets list.
fn validate_refstreets(
    errors: &mut Vec<String>,
    parent: &str,
    refstreets: &serde_json::Value,
) -> anyhow::Result<()> {
    let refstreets = refstreets.as_object().unwrap();
    let context = format!("{}.", parent);
    for (key, value) in refstreets {
        if !value.is_string() {
            errors.push(format!(
                "expected value type for '{}{}' is str",
                context, key
            ));
            continue;
        }
        let value = value.as_str().unwrap();
        if key.contains('\'') || key.contains('"') {
            errors.push(format!("expected no quotes in '{}{}'", context, key));
        }
        if value.contains('\'') || value.contains('"') {
            errors.push(format!(
                "expected no quotes in value of '{}{}'",
                context, key
            ));
        }
    }
    let mut reverse: Vec<_> = refstreets
        .iter()
        .map(|(_key, value)| value.as_str())
        .collect();
    reverse.sort();
    reverse.dedup();
    if refstreets.keys().len() != reverse.len() {
        // TODO use parent here, not context
        errors.push(format!(
            "osm and ref streets are not a 1:1 mapping in '{}'",
            context
        ));
    }

    Ok(())
}

/// Validates a street filter list.
fn validate_street_filters(
    errors: &mut Vec<String>,
    parent: &str,
    street_filters: &serde_json::Value,
) -> anyhow::Result<()> {
    let street_filters = street_filters.as_array().unwrap();
    for (index, street_filter) in street_filters.iter().enumerate() {
        if !street_filter.is_string() {
            errors.push(format!(
                "expected value type for '{}[{}]' is str",
                parent, index
            ));
        }
    }

    Ok(())
}

/// Validates an 'alias' list.
fn validate_relation_alias(
    errors: &mut Vec<String>,
    parent: &str,
    alias: &serde_json::Value,
) -> anyhow::Result<()> {
    let alias = alias.as_array().unwrap();
    for (index, alias_data) in alias.iter().enumerate() {
        if !alias_data.is_string() {
            errors.push(format!(
                "expected value type for '{}[{}]' is str",
                parent, index
            ));
        }
    }

    Ok(())
}

type TypeHandler = fn(&serde_json::Value) -> bool;
type ValueHandler = Option<fn(&mut Vec<String>, &str, &serde_json::Value) -> anyhow::Result<()>>;

lazy_static! {
    // TODO fix these odd value types.
    static ref HANDLERS: HashMap<String, (TypeHandler, String, ValueHandler)> = {
        let mut ret: HashMap<String, (TypeHandler, String, ValueHandler)> = HashMap::new();
        ret.insert("osmrelation".into(), (|v: &serde_json::Value| v.is_number(), "<class 'int'>".into(), None));
        ret.insert("refcounty".into(), (|v: &serde_json::Value| v.is_string(), "<class 'str'>".into(), None));
        ret.insert("refsettlement".into(), (|v: &serde_json::Value| v.is_string(), "<class 'str'>".into(), None));
        ret.insert("source".into(), (|v: &serde_json::Value| v.is_string(), "<class 'str'>".into(), None));
        ret.insert("filters".into(), (|v: &serde_json::Value| v.is_object(), "<class 'dict'>".into(), Some(validate_filters)));
        ret.insert("refstreets".into(), (|v: &serde_json::Value| v.is_object(), "<class 'dict'>".into(), Some(validate_refstreets)));
        ret.insert("missing-streets".into(), (|v: &serde_json::Value| v.is_string(), "<class 'str'>".into(), None));
        ret.insert("street-filters".into(), (|v: &serde_json::Value| v.is_array(), "<class 'list'>".into(), Some(validate_street_filters)));
        ret.insert("osm-street-filters".into(), (|v: &serde_json::Value| v.is_array(), "<class 'list'>".into(), Some(validate_street_filters)));
        ret.insert("inactive".into(), (|v: &serde_json::Value| v.is_boolean(), "<class 'bool'>".into(), None));
        ret.insert("housenumber-letters".into(), (|v: &serde_json::Value| v.is_boolean(), "<class 'bool'>".into(), None));
        ret.insert("additional-housenumbers".into(), (|v: &serde_json::Value| v.is_boolean(), "<class 'bool'>".into(), None));
        ret.insert("alias".into(), (|v: &serde_json::Value| v.is_array(), "<class 'list'>".into(), Some(validate_relation_alias)));
        ret
    };
}

/// Validates a toplevel or a nested relation.
fn validate_relation(
    errors: &mut Vec<String>,
    parent: &str,
    relation: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    let mut context: String = "".into();
    if !parent.is_empty() {
        context = format!("{}.", parent);

        // Just to be consistent, we require these keys in relations.yaml for now, even if code would
        // handle having them there or in relation-foo.yaml as well.
        for key in ["osmrelation", "refcounty", "refsettlement"] {
            if !relation.contains_key(key) {
                errors.push(format!("missing key '{}{}'", context, key));
            }
        }
    }

    for (key, value) in relation {
        if HANDLERS.contains_key(key) {
            let (type_check, ref value_type, handler) = HANDLERS[key];
            if !type_check(value) {
                errors.push(format!(
                    "expected value type for '{}{}' is {}",
                    context, key, value_type
                ));
                continue;
            }
            if let Some(func) = handler {
                func(errors, &format!("{}{}", context, key), value)?;
            }
        } else {
            errors.push(format!("unexpected key '{}{}'", context, key));
        }
    }

    Ok(())
}

/// Validates a relation list.
fn validate_relations(
    errors: &mut Vec<String>,
    relations: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<()> {
    for (key, value) in relations {
        validate_relation(errors, key, value.as_object().unwrap())?;
    }

    Ok(())
}

/// Commandline interface to this module.
pub fn main(argv: &[String], stream: &mut dyn Write) -> anyhow::Result<i32> {
    let yaml_path = argv[1].clone();
    let path = std::path::Path::new(&yaml_path);
    let data = std::fs::read_to_string(&yaml_path)?;
    let yaml_data = serde_yaml::from_str::<serde_json::Value>(&data)?;
    let mut errors: Vec<String> = Vec::new();
    if path.ends_with("relations.yaml") {
        validate_relations(&mut errors, yaml_data.as_object().unwrap())?;
    } else {
        let parent = "";
        validate_relation(&mut errors, parent, yaml_data.as_object().unwrap())?;
    }
    if !errors.is_empty() {
        for error in errors {
            stream
                .write_all(format!("failed to validate {}: {}\n", yaml_path, error).as_bytes())?;
        }
        return Ok(1_i32);
    }
    Ok(0_i32)
}

#[pyfunction]
fn py_validator_main(argv: Vec<String>, stream: PyObject) -> PyResult<i32> {
    let mut stream = context::PyAnyWrite { write: stream };
    match main(&argv, &mut stream).context("main() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Registers Python wrappers of Rust structs into the Python module.
pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_validator_main, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests main(): valid relations.
    #[test]
    fn test_relations() {
        let paths = [
            "tests/data/relations.yaml",
            "tests/data/relation-gazdagret-filter-invalid-good.yaml",
            "tests/data/relation-gazdagret-filter-invalid-good2.yaml",
            "tests/data/relation-gazdagret-filter-valid-good.yaml",
            "tests/data/relation-gazdagret-filter-valid-good2.yaml",
        ];
        for path in paths {
            let argv = ["".to_string(), path.to_string()];
            let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
            let ret = main(&argv, &mut buf).unwrap();
            assert_eq!(ret, 0);
        }
    }

    /// Tests the missing-osmrelation relations path.
    #[test]
    fn test_relations_missing_osmrelation() {
        // Set up arguments.
        let argv: &[String] = &[
            "".into(),
            "tests/data/relations-missing-osmrelation/relations.yaml".into(),
        ];
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let ret = main(argv, &mut buf).unwrap();
        assert_eq!(ret, 1);
        let expected = b"failed to validate tests/data/relations-missing-osmrelation/relations.yaml: missing key 'gazdagret.osmrelation'\n";
        assert_eq!(buf.into_inner(), expected);
    }

    /// Tests the happy relation path.
    #[test]
    fn test_relation() {
        // Set up arguments.
        let argv: &[String] = &["".into(), "tests/data/relation-gazdagret.yaml".into()];
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let ret = main(argv, &mut buf).unwrap();
        assert_eq!(ret, 0);
        assert_eq!(buf.into_inner(), b"");
    }

    /// Asserts that a given input fails with a given error message.
    fn assert_failure_msg(path: &str, expected: &str) {
        let argv: &[String] = &["".to_string(), path.to_string()];
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let ret = main(argv, &mut buf).unwrap();
        assert_eq!(ret, 1);
        assert_eq!(buf.into_inner(), expected.as_bytes());
    }

    /// Tests the relation path: bad source type.
    #[test]
    fn test_relation_source_bad_type() {
        let expected = "failed to validate tests/data/relation-gazdagret-source-int.yaml: expected value type for 'source' is <class 'str'>\n";
        assert_failure_msg("tests/data/relation-gazdagret-source-int.yaml", expected);
    }

    /// Tests the relation path: bad filters type.
    #[test]
    fn test_relation_filters_bad_type() {
        let expected = "failed to validate tests/data/relation-gazdagret-filters-bad.yaml: expected value type for 'filters.Budaörsi út.ranges' is list\n";
        assert_failure_msg("tests/data/relation-gazdagret-filters-bad.yaml", expected);
    }

    /// Tests the relation path: bad toplevel key name.
    #[test]
    fn test_relation_bad_key_name() {
        let expected = "failed to validate tests/data/relation-gazdagret-bad-key.yaml: unexpected key 'invalid'\n";
        assert_failure_msg("tests/data/relation-gazdagret-bad-key.yaml", expected);
    }

    /// Tests the relation path: bad strfilters value type.
    #[test]
    fn test_relation_strfilters_bad_type() {
        let expected = "failed to validate tests/data/relation-gazdagret-street-filters-bad.yaml: expected value type for 'street-filters[0]' is str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-street-filters-bad.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad refstreets value type.
    #[test]
    fn test_relation_refstreets_bad_value_type() {
        let expected = "failed to validate tests/data/relation-gazdagret-refstreets-bad-value.yaml: expected value type for 'refstreets.OSM Name 1' is str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-refstreets-bad-value.yaml",
            expected,
        );
    }

    /// Tests the relation path: quote in refstreets key or value.
    #[test]
    fn test_relation_refstreets_quote() {
        let expected = r#"failed to validate tests/data/relation-gazdagret-refstreets-quote.yaml: expected no quotes in 'refstreets.OSM Name 1''
failed to validate tests/data/relation-gazdagret-refstreets-quote.yaml: expected no quotes in value of 'refstreets.OSM Name 1''
"#;
        assert_failure_msg(
            "tests/data/relation-gazdagret-refstreets-quote.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> interpolation value type.
    #[test]
    fn test_relation_filters_interpolation_bad() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-interpolation-bad.yaml: expected value type for 'filters.Hamzsabégi út.interpolation' is str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-interpolation-bad.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filterssubkey name.
    #[test]
    fn test_relation_filters_bad_subkey() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-bad.yaml: unexpected key 'filters.Budaörsi út.unexpected'\n";
        assert_failure_msg("tests/data/relation-gazdagret-filter-bad.yaml", expected);
    }

    /// Tests the relation path: bad filters -> refsettlement value type.
    #[test]
    fn test_relation_filters_refsettlement_bad() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-refsettlement-bad.yaml: expected value type for 'filters.Hamzsabégi út.refsettlement' is str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-refsettlement-bad.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> invalid subkey.
    #[test]
    fn test_relation_filters_invalid_bad() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-invalid-bad.yaml: expected value type for 'filters.Budaörsi út.invalid[0]' is str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-invalid-bad.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> invalid subkey.
    #[test]
    fn test_relation_filters_invalid_bad2() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-invalid-bad2.yaml: expected format for 'filters.Budaörsi út.invalid[0]' is '42', '42a' or '42/1'\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-invalid-bad2.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad type for the filters -> ... -> invalid subkey.
    #[test]
    fn test_relation_filters_invalid_bad_type() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-invalid-bad-type.yaml: expected value type for 'filters.Budaörsi út.invalid' is list\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-invalid-bad-type.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> ranges subkey.
    #[test]
    fn test_relation_filters_ranges_bad() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad.yaml: unexpected key 'filters.Budaörsi út.ranges[0].unexpected'\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-bad.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> ranges subkey type.
    #[test]
    fn test_relation_filters_ranges_bad_type() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-type.yaml: expected value type for 'filters.Budaörsi út.ranges[0].refsettlement' is str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-bad-type.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> ranges -> end type.
    #[test]
    fn test_relation_filters_ranges_bad_end() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-end.yaml: expected value type for 'filters.Budaörsi út.ranges[0].end' is a digit str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-bad-end.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> ranges -> if start/end is swapped type.
    #[test]
    fn test_relation_filters_ranges_start_end_swap() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-start-end-swap.yaml: expected end >= start for 'filters.Budaörsi út.ranges[0]'\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-start-end-swap.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> ranges -> if start/end is either both
    /// even/odd or not.
    #[test]
    fn test_relation_filters_ranges_start_end_even_odd() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-start-end-even-odd.yaml: expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-start-end-even-odd.yaml",
            expected,
        );
    }

    /// Tests the relation path: bad filters -> ... -> ranges -> start type.
    #[test]
    fn test_relation_filters_ranges_bad_start() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-bad-start.yaml: expected value type for 'filters.Budaörsi út.ranges[0].start' is a digit str\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-bad-start.yaml",
            expected,
        );
    }

    /// Tests the relation path: missing filters -> ... -> ranges -> start key.
    #[test]
    fn test_relation_filters_ranges_missing_start() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-missing-start.yaml: unexpected missing key 'start' for 'filters.Budaörsi út.ranges[0]'\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-missing-start.yaml",
            expected,
        );
    }

    /// Tests the relation path: missing filters -> ... -> ranges -> end key.
    #[test]
    fn test_relation_filters_ranges_missing_end() {
        let expected = "failed to validate tests/data/relation-gazdagret-filter-range-missing-end.yaml: unexpected missing key 'end' for 'filters.Budaörsi út.ranges[0]'\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-filter-range-missing-end.yaml",
            expected,
        );
    }

    /// Tests the housenumber-letters key: bad type.
    #[test]
    fn test_relation_housenumber_letters_bad() {
        let expected = "failed to validate tests/data/relation-gazdagret-housenumber-letters-bad.yaml: expected value type for 'housenumber-letters' is <class 'bool'>\n";
        assert_failure_msg(
            "tests/data/relation-gazdagret-housenumber-letters-bad.yaml",
            expected,
        );
    }
}
