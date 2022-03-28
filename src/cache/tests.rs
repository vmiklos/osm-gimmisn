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
    let mut file_system = context::tests::TestFileSystem::new();
    let relation_percent = context::tests::TestFileSystem::make_file();
    let relation_htmlcache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/gazdagret.percent", &relation_percent),
            ("workdir/gazdagret.htmlcache.en", &relation_htmlcache),
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
    let relation = relations.get_relation("gazdagret").unwrap();

    let is_cached = is_missing_housenumbers_html_cached(&ctx, &relation).unwrap();

    assert_eq!(is_cached, false);
}

/// Tests is_missing_housenumbers_html_cached(): the case when osm_housenumbers is new, so the cache entry is old.
#[test]
fn test_is_missing_housenumbers_html_cached_osm_housenumbers_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let osm_housenumbers = context::tests::TestFileSystem::make_file();
    let html_cache = context::tests::TestFileSystem::make_file();
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &osm_housenumbers,
            ),
            ("workdir/gazdagret.htmlcache.en", &html_cache),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv"),
        Rc::new(RefCell::new(1_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let ret = is_missing_housenumbers_html_cached(&ctx, &relation).unwrap();

    assert_eq!(ret, false);
}

/// Tests is_missing_housenumbers_html_cached(): the case when ref_housenumbers is new, so the cache entry is old.
#[test]
fn test_is_missing_housenumbers_html_cached_ref_housenumbers_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let html_cache = context::tests::TestFileSystem::make_file();
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers,
            ),
            ("workdir/gazdagret.htmlcache.en", &html_cache),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        Rc::new(RefCell::new(1_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let ret = is_missing_housenumbers_html_cached(&ctx, &relation).unwrap();

    assert_eq!(ret, false);
}

/// Tests is_missing_housenumbers_html_cached(): the case when relation is new, so the cache entry is old.
#[test]
fn test_is_missing_housenumbers_html_cached_relation_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let osm_streets = context::tests::TestFileSystem::make_file();
    let osm_housenumbers = context::tests::TestFileSystem::make_file();
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let html_cache = context::tests::TestFileSystem::make_file();
    let relation_file = context::tests::TestFileSystem::make_file();
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/streets-gazdagret.csv", &osm_streets),
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &osm_housenumbers,
            ),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers,
            ),
            ("workdir/gazdagret.htmlcache.en", &html_cache),
            ("data/relation-gazdagret.yaml", &relation_file),
        ],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string("cached", &ctx.get_abspath("workdir/gazdagret.htmlcache.en"))
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/streets-gazdagret.csv"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("data/relation-gazdagret.yaml"),
        Rc::new(RefCell::new(1_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let ret = is_missing_housenumbers_html_cached(&ctx, &relation).unwrap();

    assert_eq!(ret, false);
}

/// Tests get_additional_housenumbers_html(): the case when we find the result in cache
#[test]
fn test_get_additional_housenumbers_html() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let relation_count = context::tests::TestFileSystem::make_file();
    let relation_htmlcache = context::tests::TestFileSystem::make_file();
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/gazdagret-additional-housenumbers.count",
                &relation_count,
            ),
            (
                "workdir/gazdagret.additional-htmlcache.en",
                &relation_htmlcache,
            ),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.additional-htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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

/// Tests get_missing_housenumbers_html().
#[test]
fn test_get_missing_housenumbers_html() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let osm_streets = context::tests::TestFileSystem::make_file();
    let osm_housenumbers = context::tests::TestFileSystem::make_file();
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let html_cache = context::tests::TestFileSystem::make_file();
    let relation_file = context::tests::TestFileSystem::make_file();
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/streets-gazdagret.csv", &osm_streets),
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &osm_housenumbers,
            ),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers,
            ),
            ("workdir/gazdagret.htmlcache.en", &html_cache),
            ("data/relation-gazdagret.yaml", &relation_file),
        ],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string("cached", &ctx.get_abspath("workdir/gazdagret.htmlcache.en"))
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/streets-gazdagret.csv"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
        Rc::new(RefCell::new(1_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("data/relation-gazdagret.yaml"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_missing_housenumbers_html(&ctx, &mut relation).unwrap();

    assert_eq!(ret.get_value(), "cached");
}
