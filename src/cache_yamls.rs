/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cache_yamls module caches YAML files from the data/ directory.

use crate::context;
use anyhow::Context;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::ops::DerefMut;

/// Commandline interface.
pub fn main(argv: &[String], ctx: &context::Context) -> anyhow::Result<()> {
    let mut cache: HashMap<String, serde_json::Value> = HashMap::new();
    let datadir = ctx.get_abspath(&argv[1])?;
    let entries = std::fs::read_dir(&datadir).unwrap();
    let mut yaml_paths: Vec<String> = Vec::new();
    for entry in entries {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();
        if path.ends_with(".yaml") {
            yaml_paths.push(path.to_string());
        }
    }
    yaml_paths.sort();
    for yaml_path in yaml_paths {
        let cache_key = std::path::Path::new(&yaml_path)
            .strip_prefix(&datadir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let data = std::fs::read_to_string(&yaml_path).unwrap();
        let cache_value = serde_yaml::from_str::<serde_json::Value>(&data).unwrap();
        cache.insert(cache_key, cache_value);
    }

    let cache_path = format!("{}/yamls.cache", datadir);
    {
        let write_stream = ctx.get_file_system().open_write(&cache_path).unwrap();
        let mut guard = write_stream.lock().unwrap();
        let write = guard.deref_mut();
        serde_json::to_writer(write, &cache).unwrap();
    }

    let workdir = argv[2].clone();
    let yaml_path = format!("{}/relations.yaml", datadir);
    let mut relation_ids: Vec<u64> = Vec::new();
    let stream = std::fs::File::open(yaml_path).unwrap();
    let relations: serde_yaml::Value = serde_yaml::from_reader(stream).unwrap();
    for (_key, value) in relations.as_mapping().unwrap() {
        relation_ids.push(value["osmrelation"].as_u64().unwrap());
    }
    relation_ids.sort_unstable();
    relation_ids.dedup();
    let statsdir = format!("{}/stats", workdir);
    std::fs::create_dir_all(&statsdir).unwrap();
    {
        let write_stream = ctx
            .get_file_system()
            .open_write(&format!("{}/relations.json", statsdir))
            .unwrap();
        let mut guard = write_stream.lock().unwrap();
        let write = guard.deref_mut();
        serde_json::to_writer(write, &relation_ids).unwrap();
    }

    Ok(())
}

/// Commandline interface.
#[pyfunction]
pub fn py_cache_yamls_main(argv: Vec<String>, ctx: &context::PyContext) -> PyResult<()> {
    match main(&argv, &ctx.context).context("main() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Registers Python wrappers of Rust structs into the Python module.
pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_cache_yamls_main, module)?)?;
    Ok(())
}
