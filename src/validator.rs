/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The validator module validates yaml files under data/.

use crate::areas;
use crate::context;
use anyhow::Context;
use std::collections::HashMap;
use std::io::Write;

/// Validates a range description: check for missing keys."""
fn validate_range_missing_keys(
    errors: &mut Vec<String>,
    parent: &str,
    range_data: &areas::RelationRangesDict,
    filter_data: &areas::RelationFiltersDict,
) -> anyhow::Result<()> {
    let start: i64 = match range_data.start.parse() {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!(
                "expected value type for '{parent}.start' is a digit str"
            ));
            return Ok(());
        }
    };
    let end: i64 = match range_data.end.parse() {
        Ok(value) => value,
        Err(_) => {
            errors.push(format!(
                "expected value type for '{parent}.end' is a digit str"
            ));
            return Ok(());
        }
    };
    if start > end {
        errors.push(format!("expected end >= start for '{parent}'"));
    }

    if filter_data.interpolation.is_none() && start % 2 != end % 2 {
        errors.push(format!("expected start % 2 == end % 2 for '{parent}'"))
    }

    Ok(())
}

/// Validates a range description.
fn validate_range(
    errors: &mut Vec<String>,
    parent: &str,
    range_data: &areas::RelationRangesDict,
    filter_data: &areas::RelationFiltersDict,
) -> anyhow::Result<()> {
    validate_range_missing_keys(errors, parent, range_data, filter_data)?;
    Ok(())
}

/// Validates a range list.
fn validate_ranges(
    errors: &mut Vec<String>,
    parent: &str,
    ranges: &[areas::RelationRangesDict],
    filter_data: &areas::RelationFiltersDict,
) -> anyhow::Result<()> {
    for (index, range_data) in ranges.iter().enumerate() {
        let parent = format!("{parent}[{index}]");
        validate_range(errors, &parent, range_data, filter_data)?;
    }

    Ok(())
}

/// Validates an 'invalid' or 'valid' list.
fn validate_filter_invalid_valid(
    errors: &mut Vec<String>,
    parent: &str,
    invalid: &[String],
) -> anyhow::Result<()> {
    let is_number = regex::Regex::new(r"^[0-9]+$")?;
    let is_number_and_letter = regex::Regex::new(r"^[0-9]+[a-z]$")?;
    let is_number_per_letter = regex::Regex::new(r"^[0-9]+/[0-9]$")?;
    let is_range = regex::Regex::new(r"^[0-9]+[a-z]?-[0-9]*[a-z]?$")?;
    for (index, invalid_data) in invalid.iter().enumerate() {
        if is_number.is_match(invalid_data) {
            continue;
        }
        if is_number_and_letter.is_match(invalid_data) {
            continue;
        }
        if is_number_per_letter.is_match(invalid_data) {
            continue;
        }

        // 40-60 or 50a-b or 70-80a: OK, but won't be parsed.
        if is_range.is_match(invalid_data) {
            continue;
        }
        errors.push(format!(
            "expected format for '{parent}[{index}]' is '42', '42a' or '42/1'"
        ));
    }

    Ok(())
}

/// Validates a filter dictionary.
fn validate_filter(
    errors: &mut Vec<String>,
    parent: &str,
    filter_data: &areas::RelationFiltersDict,
) -> anyhow::Result<()> {
    let context = format!("{parent}.");
    if let Some(ref ranges) = filter_data.ranges {
        validate_ranges(errors, &format!("{context}ranges"), ranges, filter_data)?;
    }

    if let Some(ref invalid) = filter_data.invalid {
        validate_filter_invalid_valid(errors, &format!("{}{}", context, "invalid"), invalid)?;
    }
    if let Some(ref valid) = filter_data.valid {
        validate_filter_invalid_valid(errors, &format!("{}{}", context, "valid"), valid)?;
    }

    Ok(())
}

/// Validates a filter list.
fn validate_filters(
    errors: &mut Vec<String>,
    parent: &str,
    filters: &HashMap<String, areas::RelationFiltersDict>,
) -> anyhow::Result<()> {
    let context = format!("{parent}.");
    for (key, value) in filters {
        // At least one sub-key is now provided, otherwise the type would be null.
        validate_filter(errors, &format!("{context}{key}"), value)?;
    }

    Ok(())
}

/// Validates a reference streets list.
fn validate_refstreets(
    errors: &mut Vec<String>,
    parent: &str,
    refstreets: &HashMap<String, String>,
) -> anyhow::Result<()> {
    let context = format!("{parent}.");
    for (key, value) in refstreets {
        if key.contains('\'') || key.contains('"') {
            errors.push(format!("expected no quotes in '{context}{key}'"));
        }
        if value.contains('\'') || value.contains('"') {
            errors.push(format!("expected no quotes in value of '{context}{key}'"));
        }
        if key == value {
            errors.push(format!("expected value != key for '{context}{key}'"));
        }
    }
    let mut reverse: Vec<_> = refstreets.values().map(|value| value.as_str()).collect();
    reverse.sort_unstable();
    reverse.dedup();
    if refstreets.keys().len() != reverse.len() {
        errors.push(format!(
            "osm and ref streets are not a 1:1 mapping in '{parent}'"
        ));
    }

    Ok(())
}

/// Validates a toplevel or a nested relation.
fn validate_relation(
    errors: &mut Vec<String>,
    parent: &str,
    relation: &areas::RelationDict,
) -> anyhow::Result<()> {
    let mut context: String = "".into();
    if !parent.is_empty() {
        context = format!("{parent}.");

        // Just to be consistent, we require these keys in relations.yaml for now, even if code would
        // handle having them there or in relation-foo.yaml as well.
        if relation.osmrelation.is_none() {
            errors.push(format!("missing key '{context}osmrelation'"));
        }
        if relation.refcounty.is_none() {
            errors.push(format!("missing key '{context}refcounty'"));
        }
        if relation.refsettlement.is_none() {
            errors.push(format!("missing key '{context}refsettlement'"));
        }
    }

    if let Some(ref filters) = relation.filters {
        validate_filters(errors, &format!("{}{}", context, "filters"), filters)?;
    }
    if let Some(ref refstreets) = relation.refstreets {
        validate_refstreets(errors, &format!("{}{}", context, "refstreets"), refstreets)?;
    }

    Ok(())
}

/// Validates a relation list.
fn validate_relations(
    errors: &mut Vec<String>,
    relations: &areas::RelationsDict,
) -> anyhow::Result<()> {
    for (key, value) in relations {
        validate_relation(errors, key, value)?;
    }

    Ok(())
}

/// Similar to plain main(), but with an interface that allows testing.
pub fn main(argv: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    match our_main(argv, stream, ctx) {
        Ok(_) => 0,
        Err(err) => {
            stream
                .write_all(format!("{err:?}\n").as_bytes())
                .expect("write_all() failed");
            1
        }
    }
}

/// Inner main() that is allowed to fail.
pub fn our_main(
    argv: &[String],
    stream: &mut dyn Write,
    ctx: &context::Context,
) -> anyhow::Result<()> {
    let yaml_path = &argv[1];
    let data = ctx.get_file_system().read_to_string(yaml_path)?;
    let mut errors: Vec<String> = Vec::new();

    if data.contains('\t') {
        // serde can parse this, but not some of the 3rd-party parsers.
        errors.push("expected indent with 2 spaces, not with tabs".to_string());
    }

    if yaml_path.ends_with("relations.yaml") {
        let relations_dict: areas::RelationsDict =
            serde_yaml::from_str(&data).context("serde_yaml::from_str() failed")?;
        validate_relations(&mut errors, &relations_dict)?;
    } else {
        // This will fail if the data is not well-formed (e.g. in case of duplicated keys):
        serde_yaml::from_str::<serde_yaml::Value>(&data)
            .context(format!("failed to validate {yaml_path}"))?;

        // Then check if the data is valid:
        let relation_dict_json: serde_json::Value = serde_yaml::from_str(&data)
            .context(format!("failed to convert {yaml_path} to json"))?;
        let relation_dict: areas::RelationDict = serde_json::from_value(relation_dict_json.clone())
            .context("failed to parse as json")?;
        let parent = "";
        validate_relation(&mut errors, parent, &relation_dict)?;
    }
    if !errors.is_empty() {
        for error in errors {
            stream.write_all(format!("{error}\n").as_bytes())?;
        }
        return Err(anyhow::anyhow!("failed to validate {}", yaml_path));
    }

    Ok(())
}

#[cfg(test)]
mod tests;
