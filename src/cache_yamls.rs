/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cache_yamls module caches YAML files from the data/ directory.

use crate::areas;
use crate::context;
use anyhow::Context;
use std::collections::HashMap;
use std::ops::DerefMut;

/// Commandline interface.
pub fn main(argv: &[String], ctx: &context::Context) -> anyhow::Result<()> {
    let mut cache: HashMap<String, serde_json::Value> = HashMap::new();
    let datadir = ctx.get_abspath(&argv[1]);
    let entries =
        std::fs::read_dir(&datadir).context(format!("failed to read_dir() {}", datadir))?;
    let mut yaml_paths: Vec<String> = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let path = path.to_str().context("failed to convert path to string")?;
        if path.ends_with(".yaml") {
            yaml_paths.push(path.to_string());
        }
    }
    yaml_paths.sort();
    for yaml_path in yaml_paths {
        let cache_key = yaml_path
            .strip_prefix(&format!("{}/", datadir))
            .context("yaml outside datadir")?
            .to_string();
        let data = std::fs::read_to_string(&yaml_path)?;
        let cache_value = serde_yaml::from_str::<serde_json::Value>(&data)
            .context(format!("serde_yaml::from_str() failed for {}", yaml_path))?;
        cache.insert(cache_key, cache_value);
    }

    let cache_path = format!("{}/yamls.cache", datadir);
    {
        let write_stream = ctx.get_file_system().open_write(&cache_path)?;
        let mut guard = write_stream.borrow_mut();
        let write = guard.deref_mut();
        serde_json::to_writer(write, &cache)?;
    }

    let workdir = argv[2].clone();
    let yaml_path = format!("{}/relations.yaml", datadir);
    let mut relation_ids: Vec<u64> = Vec::new();
    let stream = std::fs::File::open(yaml_path)?;
    let relations: areas::RelationsDict = serde_yaml::from_reader(stream)?;
    for (_key, value) in relations {
        relation_ids.push(value.osmrelation.context("no osmrelation")?);
    }
    relation_ids.sort_unstable();
    relation_ids.dedup();
    let statsdir = format!("{}/stats", workdir);
    std::fs::create_dir_all(&statsdir)?;
    {
        let write_stream = ctx
            .get_file_system()
            .open_write(&format!("{}/relations.json", statsdir))?;
        let mut guard = write_stream.borrow_mut();
        let write = guard.deref_mut();
        serde_json::to_writer(write, &relation_ids)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::sync::Arc;

    /// Tests main().
    #[test]
    fn test_main() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let cache_path = ctx.get_abspath("data/yamls.cache");
        let argv = vec!["".to_string(), "data".to_string(), "workdir".to_string()];
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[cache_path]);
        let cache_value = context::tests::TestFileSystem::make_file();
        let stats_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[
                ("data/yamls.cache", &cache_value),
                ("workdir/stats/relations.json", &stats_value),
            ],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);

        main(&argv, &mut ctx).unwrap();

        // Just assert that the result is created, the actual content is validated by the other
        // tests.
        {
            let mut guard = cache_value.borrow_mut();
            assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        }

        let relation_ids_path = ctx.get_abspath("workdir/stats/relations.json");
        let file = std::fs::File::open(relation_ids_path).unwrap();
        let relation_ids: serde_json::Value = serde_json::from_reader(&file).unwrap();
        let relation_ids: Vec<_> = relation_ids
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.as_u64().unwrap())
            .collect();
        let mut relations = areas::Relations::new(&ctx).unwrap();
        let mut osmids: Vec<_> = relations
            .get_relations()
            .unwrap()
            .iter()
            .map(|i| i.get_config().get_osmrelation())
            .collect();
        osmids.sort();
        osmids.dedup();
        assert_eq!(relation_ids, osmids);
    }
}
