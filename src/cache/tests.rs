/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the cache module.

use super::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Tests get_missing_housenumbers_json(): the cached case.
///
/// The non-cached case is covered by higher level
/// wsgi_json::tests::test_missing_housenumbers_view_result_json().
#[test]
fn test_get_missing_housenumbers_json() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            r#"insert into missing_housenumbers_cache (relation, json) values ('gazdagret', '{"cached":"yes"}');"#,
        )
        .unwrap();
    }
    stats::set_sql_mtime(&ctx, "missing-housenumbers-cache/gazdagret").unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers,
            ),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_missing_housenumbers_json(&mut relation).unwrap();

    assert_eq!(ret, r#"{"cached":"yes"}"#);
}

/// Tests get_missing_housenumbers_json(): the cached case, when an sql dependency is newer.
#[test]
fn test_get_missing_housenumbers_json_sql_newer() {
    // missing-housenumbers-cache/gazdagret is older than streets/gazdagret, both are in sql.
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            r#"insert into missing_housenumbers_cache (relation, json) values ('gazdagret', '{"cached":"yes"}');
               insert into mtimes (page, last_modified) values ('missing-housenumbers-cache/gazdagret', '0');"#,
        )
        .unwrap();
    }
    stats::set_sql_mtime(&ctx, "streets/gazdagret").unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers,
            ),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_missing_housenumbers_json(&mut relation).unwrap();

    assert!(ret != r#"{"cached":"yes"}"#);
}

/// Tests get_additional_housenumbers_json(): the cached case.
///
/// The non-cached case is covered by higher level
/// wsgi_json::tests::test_additional_housenumbers_view_result_json().
#[test]
fn test_get_additional_housenumbers_json() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            r#"insert into additional_housenumbers_cache (relation, json) values ('gazdagret', '{"cached":"yes"}');"#,
        )
        .unwrap();
    }
    stats::set_sql_mtime(&ctx, "additional-housenumbers-cache/gazdagret").unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_additional_housenumbers_json(&mut relation).unwrap();

    assert_eq!(ret, r#"{"cached":"yes"}"#);
}
