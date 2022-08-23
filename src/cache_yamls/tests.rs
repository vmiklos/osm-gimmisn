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
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::Arc;

/// Tests main().
#[test]
fn test_main() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let cache_path = ctx.get_abspath("data/yamls.cache");
    let argv = vec!["".to_string(), "data".to_string(), "workdir".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[cache_path]);
    let relations_value = context::tests::TestFileSystem::make_file();
    let relations_content = r#"gazdagret:
    osmrelation: 2713748
    refcounty: "01"
    refsettlement: "011"
"#;
    relations_value
        .borrow_mut()
        .write_all(relations_content.as_bytes())
        .unwrap();
    let cache_value = context::tests::TestFileSystem::make_file();
    let stats_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/relations.yaml", &relations_value),
            ("data/yamls.cache", &cache_value),
            ("workdir/stats/relations.json", &stats_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let ret = main(&argv, &mut buf, &ctx);

    // Just assert that the result is created, the actual content is validated by the other
    // tests.
    assert_eq!(ret, 0);
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
    assert_eq!(relation_ids, [2713748]);
}

/// Tests main() failure.
#[test]
fn test_main_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_arc: Arc<dyn context::Unit> = Arc::new(unit);
    ctx.set_unit(&unit_arc);
    let cache_path = ctx.get_abspath("data/yamls.cache");
    let argv = vec!["".to_string(), "data".to_string(), "workdir".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
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

    let ret = main(&argv, &mut buf, &ctx);

    assert_eq!(ret, 1);
}
