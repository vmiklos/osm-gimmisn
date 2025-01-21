/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cache_yamls module caches YAML files from the data/ directory.

use crate::areas;
use crate::context;
use anyhow::Context;
use std::collections::HashMap;
use std::io::Write;
use std::ops::DerefMut;

/// Inner main() that is allowed to fail.
pub fn our_main(argv: &[String], ctx: &context::Context) -> anyhow::Result<()> {
    let mut cache: HashMap<String, serde_json::Value> = HashMap::new();
    let datadir = ctx.get_abspath(&argv[1]);
    let entries = ctx
        .get_file_system()
        .listdir(&datadir)
        .context(format!("failed to listdir() {datadir}"))?;
    let mut yaml_paths: Vec<String> = Vec::new();
    for path in entries {
        if path.ends_with(".yaml") {
            yaml_paths.push(path.to_string());
        }
    }
    yaml_paths.sort();
    for yaml_path in yaml_paths {
        let cache_key = yaml_path
            .strip_prefix(&format!("{datadir}/"))
            .context("yaml outside datadir")?
            .to_string();
        let data = ctx.get_file_system().read_to_string(&yaml_path)?;
        let cache_value = serde_yaml::from_str::<serde_json::Value>(&data)
            .context(format!("serde_yaml::from_str() failed for {yaml_path}"))?;
        cache.insert(cache_key, cache_value);
    }

    let cache_path = format!("{datadir}/yamls.cache");
    {
        let write_stream = ctx.get_file_system().open_write(&cache_path)?;
        let mut guard = write_stream.borrow_mut();
        let write = guard.deref_mut();
        serde_json::to_writer(write, &cache)?;
    }

    let yaml_path = format!("{datadir}/relations.yaml");
    let mut relation_ids: Vec<u64> = Vec::new();
    let data = ctx.get_file_system().read_to_string(&yaml_path)?;
    let relations: areas::RelationsDict = serde_yaml::from_str(&data)
        .context(format!("serde_yaml::from_str() failed for {yaml_path}"))?;
    for (_key, value) in relations {
        relation_ids.push(value.osmrelation.context("no osmrelation")?);
    }
    relation_ids.sort_unstable();
    relation_ids.dedup();
    {
        let conn = ctx.get_database_connection()?;
        let sql = r#"insert into stats_jsons (category, json) values ('relations', ?1)
                     on conflict(category) do update set json = excluded.json"#;
        conn.execute(sql, [serde_json::to_string(&relation_ids)?])?;
    }

    ctx.get_unit().make_error()
}

/// Similar to plain main(), but with an interface that allows testing.
pub fn main(argv: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    match our_main(argv, ctx) {
        Ok(_) => 0,
        Err(err) => {
            stream.write_all(format!("{err:?}\n").as_bytes()).unwrap();
            1
        }
    }
}

#[cfg(test)]
mod tests;
