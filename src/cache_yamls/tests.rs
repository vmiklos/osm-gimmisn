/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the cache_yamls module.

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

    let mut guard = stats_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut read = guard.deref_mut();
    let relation_ids: serde_json::Value = serde_json::from_reader(&mut read).unwrap();
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
