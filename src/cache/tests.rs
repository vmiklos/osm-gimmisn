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
use context::FileSystem;
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
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/additional-cache-gazdagret.json", &json_cache_value),
        ],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "{'cached':'yes'}",
            &ctx.get_abspath("workdir/additional-cache-gazdagret.json"),
        )
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/additional-cache-gazdagret.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::now_utc())),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_additional_housenumbers_json(&mut relation).unwrap();

    assert_eq!(ret, "{'cached':'yes'}");
}

/// Tests is_cache_current()
#[test]
fn test_is_cache_current() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let cache_path = "workdir/gazdagret.json.cache";
    file_system.set_hide_paths(&[cache_path.to_string()]);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let ret = is_cache_current(&ctx, cache_path, &[], &[]).unwrap();

    assert_eq!(ret, false);
}

/// Tests is_cache_current(), when an sql dependency is newer.
#[test]
fn test_is_cache_current_false_from_sql() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let cache_path = ctx.get_abspath("workdir/gazdagret.json.cache");
    let mut file_system = context::tests::TestFileSystem::new();
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/gazdagret.json.cache", &json_cache_value)],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        cache_path.to_string(),
        Rc::new(RefCell::new(
            time::OffsetDateTime::from_unix_timestamp(0).unwrap(),
        )),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into mtimes (page, last_modified) values ('streets/gazdagret', '1');",
        )
        .unwrap();
    }

    let sql_dependencies = vec!["streets/gazdagret".to_string()];
    let ret = is_cache_current(&ctx, &cache_path, &[], &sql_dependencies).unwrap();

    assert_eq!(ret, false);
}
