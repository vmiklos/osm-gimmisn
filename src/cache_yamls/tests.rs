/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the cache_yamls module.

use super::*;
use std::io::Seek;
use std::io::SeekFrom;
use std::rc::Rc;

/// Tests main().
#[test]
fn test_main() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv = vec!["".to_string(), "data".to_string(), "workdir".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut file_system = context::tests::TestFileSystem::new();
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
    let refsettlements_names_value = context::tests::TestFileSystem::make_file();
    let refsettlements_names_content = r#"'01':
    '011': 'Újbuda'
    '012': 'Hegyvidék'
"#;
    refsettlements_names_value
        .borrow_mut()
        .write_all(refsettlements_names_content.as_bytes())
        .unwrap();
    let cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/relations.yaml", &relations_value),
            (
                "data/refsettlements-names.yaml",
                &refsettlements_names_value,
            ),
            ("data/yamls.cache", &cache_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let ret = main(&argv, &mut buf, &ctx);

    // Just assert that the result is created, the actual content is validated by the other
    // tests.
    assert_eq!(ret, 0);
    {
        let mut guard = cache_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
}

/// Tests main() failure.
#[test]
fn test_main_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_rc: Rc<dyn context::Unit> = Rc::new(unit);
    ctx.set_unit(&unit_rc);
    let argv = vec!["".to_string(), "data".to_string(), "workdir".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut file_system = context::tests::TestFileSystem::new();
    let cache_value = context::tests::TestFileSystem::make_file();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[("data/yamls.cache", &cache_value)]);
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let ret = main(&argv, &mut buf, &ctx);

    assert_eq!(ret, 1);
}
