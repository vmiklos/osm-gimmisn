/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
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
                "expected value type for '{}.start' is a digit str",
                parent
            ));
            return Ok(());
        }
    };
    let end: i64 = match range_data.end.parse() {
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

    if filter_data.interpolation.is_none() && start % 2 != end % 2 {
        errors.push(format!("expected start % 2 == end % 2 for '{}'", parent))
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
    invalid: &[String],
) -> anyhow::Result<()> {
    for (index, invalid_data) in invalid.iter().enumerate() {
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
    filter_data: &areas::RelationFiltersDict,
) -> anyhow::Result<()> {
    let context = format!("{}.", parent);
    if let Some(ref ranges) = filter_data.ranges {
        validate_ranges(errors, &format!("{}ranges", context), ranges, filter_data)?;
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
    let context = format!("{}.", parent);
    for (key, value) in filters {
        validate_filter(errors, &format!("{}{}", context, key), value)?;
    }

    Ok(())
}

/// Validates a reference streets list.
fn validate_refstreets(
    errors: &mut Vec<String>,
    parent: &str,
    refstreets: &HashMap<String, String>,
) -> anyhow::Result<()> {
    let context = format!("{}.", parent);
    for (key, value) in refstreets {
        if value.parse::<i64>().is_ok() {
            errors.push(format!(
                "expected value type for '{}{}' is str",
                context, key
            ));
        }
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
    reverse.sort_unstable();
    reverse.dedup();
    if refstreets.keys().len() != reverse.len() {
        errors.push(format!(
            "osm and ref streets are not a 1:1 mapping in '{}'",
            parent
        ));
    }

    Ok(())
}

/// Validates a street filter list.
fn validate_street_filters(
    errors: &mut Vec<String>,
    parent: &str,
    street_filters: &[String],
) -> anyhow::Result<()> {
    for (index, street_filter) in street_filters.iter().enumerate() {
        if street_filter.parse::<i64>().is_ok() {
            errors.push(format!(
                "expected value type for '{}[{}]' is str",
                parent, index
            ));
        }
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
        context = format!("{}.", parent);

        // Just to be consistent, we require these keys in relations.yaml for now, even if code would
        // handle having them there or in relation-foo.yaml as well.
        if relation.osmrelation.is_none() {
            errors.push(format!("missing key '{}osmrelation'", context));
        }
        if relation.refcounty.is_none() {
            errors.push(format!("missing key '{}refcounty'", context));
        }
        if relation.refsettlement.is_none() {
            errors.push(format!("missing key '{}refsettlement'", context));
        }
    }

    if let Some(ref filters) = relation.filters {
        validate_filters(errors, &format!("{}{}", context, "filters"), filters)?;
    }
    if let Some(ref refstreets) = relation.refstreets {
        validate_refstreets(errors, &format!("{}{}", context, "refstreets"), refstreets)?;
    }
    if let Some(ref street_filters) = relation.street_filters {
        validate_street_filters(
            errors,
            &format!("{}{}", context, "street-filters"),
            street_filters,
        )?;
    }
    if let Some(ref source) = relation.source {
        if source.parse::<i64>().is_ok() {
            errors.push(format!(
                "expected value type for '{}source' is str",
                context
            ));
        }
    }
    if let Some(ref aliases) = relation.alias {
        for (index, alias) in aliases.iter().enumerate() {
            if alias.parse::<i64>().is_ok() {
                errors.push(format!(
                    "expected value type for '{}alias[{}]' is str",
                    context, index
                ));
            }
        }
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
            stream.write_all(format!("{:?}\n", err).as_bytes()).unwrap();
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
    let yaml_path = argv[1].clone();
    let data = ctx.get_file_system().read_to_string(&yaml_path)?;
    let mut errors: Vec<String> = Vec::new();
    if yaml_path.ends_with("relations.yaml") {
        let relations_dict: areas::RelationsDict =
            serde_yaml::from_str(&data).context("serde_yaml::from_str() failed")?;
        validate_relations(&mut errors, &relations_dict)?;
    } else {
        let relation_dict: areas::RelationDict =
            serde_yaml::from_str(&data).context(format!("failed to validate {}", yaml_path))?;
        let parent = "";
        validate_relation(&mut errors, parent, &relation_dict)?;
    }
    if !errors.is_empty() {
        for error in errors {
            stream.write_all(format!("{}\n", error).as_bytes())?;
        }
        return Err(anyhow::anyhow!("failed to validate {}", yaml_path));
    }

    Ok(())
}

#[cfg(test)]
mod tests;
