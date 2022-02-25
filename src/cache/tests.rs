/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
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
use std::sync::Arc;

/// Tests is_missing_housenumbers_html_cached().
#[test]
fn test_is_missing_housenumbers_html_cached() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let percent_value = context::tests::TestFileSystem::make_file();
    let html_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/gazdagret.percent", &percent_value),
            ("workdir/gazdagret.htmlcache.en", &html_cache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();
    get_missing_housenumbers_html(&ctx, &mut relation).unwrap();

    assert_eq!(
        is_missing_housenumbers_html_cached(&ctx, &mut relation).unwrap(),
        true
    );
}

/// Tests is_missing_housenumbers_html_cached(): the case when there is no cache.
#[test]
fn test_is_missing_housenumbers_html_cached_no_cache() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();
    get_missing_housenumbers_html(&ctx, &mut relation).unwrap();
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path();

    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[cache_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    assert_eq!(
        is_missing_housenumbers_html_cached(&ctx, &relation).unwrap(),
        false
    );
}

/// Tests is_missing_housenumbers_html_cached(): the case when osm_housenumbers is new, so the cache entry is old.
#[test]
fn test_is_missing_housenumbers_html_cached_osm_housenumbers_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();
    get_missing_housenumbers_html(&ctx, &mut relation).unwrap();
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path();
    let osm_housenumbers_path = relation.get_files().get_osm_housenumbers_path();

    let mut file_system = context::tests::TestFileSystem::new();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let metadata = std::fs::metadata(cache_path).unwrap();
    let modified = metadata.modified().unwrap();
    let mtime = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    mtimes.insert(
        osm_housenumbers_path,
        Rc::new(RefCell::new(mtime.as_secs_f64() + 1_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    assert_eq!(
        is_missing_housenumbers_html_cached(&ctx, &relation).unwrap(),
        false
    );
}

/// Tests is_missing_housenumbers_html_cached(): the case when ref_housenumbers is new, so the cache entry is old.
#[test]
fn test_is_missing_housenumbers_html_cached_ref_housenumbers_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();
    get_missing_housenumbers_html(&ctx, &mut relation).unwrap();
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path();
    let ref_housenumbers_path = relation.get_files().get_ref_housenumbers_path();

    let mut file_system = context::tests::TestFileSystem::new();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let metadata = std::fs::metadata(cache_path).unwrap();
    let modified = metadata.modified().unwrap();
    let mtime = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    mtimes.insert(
        ref_housenumbers_path,
        Rc::new(RefCell::new(mtime.as_secs_f64() + 1_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    assert_eq!(
        is_missing_housenumbers_html_cached(&ctx, &relation).unwrap(),
        false
    );
}

/// Tests is_missing_housenumbers_html_cached(): the case when relation is new, so the cache entry is old.
#[test]
fn test_is_missing_housenumbers_html_cached_relation_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();
    get_missing_housenumbers_html(&ctx, &mut relation).unwrap();
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path();
    let datadir = ctx.get_abspath("data");
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());

    let mut file_system = context::tests::TestFileSystem::new();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let metadata = std::fs::metadata(cache_path).unwrap();
    let modified = metadata.modified().unwrap();
    let mtime = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    mtimes.insert(
        relation_path,
        Rc::new(RefCell::new(mtime.as_secs_f64() + 1_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    assert_eq!(
        is_missing_housenumbers_html_cached(&ctx, &relation).unwrap(),
        false
    );
}

/// Tests get_additional_housenumbers_html(): the case when we find the result in cache
#[test]
fn test_get_additional_housenumbers_html() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();
    let first = get_additional_housenumbers_html(&ctx, &mut relation).unwrap();
    let second = get_additional_housenumbers_html(&ctx, &mut relation).unwrap();
    assert_eq!(first.get_value(), second.get_value());
}

/// Tests is_missing_housenumbers_txt_cached().
#[test]
fn test_is_missing_housenumbers_txt_cached() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Tűzkő utca": {
                    "interpolation": "all",
                },
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let txt_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.txtcache", &txt_cache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.txtcache"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_missing_housenumbers_txt(&ctx, &mut relation).unwrap();
    assert_eq!(ret.contains("Tűzkő utca\t[1, 2]"), true);

    assert_eq!(
        is_missing_housenumbers_txt_cached(&ctx, &relation).unwrap(),
        true
    );
}

/// Tests get_missing_housenumbers_txt().
#[test]
fn test_get_missing_housenumbers_txt() {
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
    let txt_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.txtcache", &txt_cache_value),
        ],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string("cached", &ctx.get_abspath("workdir/gazdagret.txtcache"))
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.txtcache"),
        Rc::new(RefCell::new(9999999999_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_missing_housenumbers_txt(&ctx, &mut relation).unwrap();

    assert_eq!(ret, "cached");
}
