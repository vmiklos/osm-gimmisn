/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi_additional module.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Seek;
use std::io::SeekFrom;
use std::rc::Rc;
use std::sync::Arc;

use crate::areas;
use crate::context;
use crate::wsgi;

/// Tests additional streets: the txt output.
#[test]
fn test_streets_view_result_txt() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

    assert_eq!(result, "Only In OSM utca\nSecond Only In OSM utca\n");
}

/// Tests additional streets: the chkl output.
#[test]
fn test_streets_view_result_chkl() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
            "osm-street-filters": ["Second Only In OSM utca"],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.chkl");

    assert_eq!(result, "[ ] Only In OSM utca\n");
}

/// Tests additional streets: the txt output, no osm streets case.
#[test]
fn test_streets_view_result_txt_no_osm_streets() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

    assert_eq!(result, "No existing streets");
}

/// Tests additional streets: the txt output, no ref streets case.
#[test]
fn test_streets_view_result_txt_no_ref_streets() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_ref_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

    assert_eq!(result, "No reference streets");
}

/// Tests additional streets: if the view-turbo output is well-formed.
#[test]
fn test_streets_view_turbo_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-turbo");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/pre");
    assert_eq!(results.len(), 1);
}

/// Tests handle_main_housenr_additional_count().
#[test]
fn test_handle_main_housenr_additional_count() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
                "osmrelation": 42,
            },
        },
        "relation-budafok.yaml": {
            "additional-housenumbers": true,
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("budafok").unwrap();

    let actual = wsgi::handle_main_housenr_additional_count(&ctx, &relation).unwrap();

    assert_eq!(actual.get_value().contains("42 house numbers"), true);
}

/// Tests handle_main_housenr_additional_count(): what happens when the count file is not there.
#[test]
fn test_handle_main_housenr_additional_count_no_count_file() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("budafok").unwrap();
    let hide_path = relation
        .get_files()
        .get_housenumbers_additional_count_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let actual = wsgi::handle_main_housenr_additional_count(&ctx, &relation).unwrap();

    // Assert that the info is not there to ensure a fast main page.
    assert_eq!(actual.get_value().contains("42 house numbers"), false);
}

/// Tests the additional house numbers page: if the output is well-formed.
#[test]
fn test_additional_housenumbers_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_value = context::tests::TestFileSystem::make_file();
    let cache_value = context::tests::TestFileSystem::make_file();
    let jsoncache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/gazdagret-additional-housenumbers.count",
                &count_value,
            ),
            ("workdir/gazdagret.additional-htmlcache.en", &cache_value),
            ("workdir/additional-cache-gazdagret.json", &jsoncache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi
            .get_ctx()
            .get_abspath("workdir/gazdagret.additional-htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        test_wsgi
            .get_ctx()
            .get_abspath("workdir/additional-cache-gazdagret.json"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the additional house numbers page: if the output is well-formed, no osm streets case.
#[test]
fn test_additional_housenumbers_no_osm_streets_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let hide_path = test_wsgi
        .get_ctx()
        .get_abspath("workdir/streets-gazdagret.csv");
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the additional house numbers page: if the output is well-formed, no osm housenumbers case.
#[test]
fn test_additional_housenumbers_no_osm_housenumbers_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_housenumbers_path();
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
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
    assert_eq!(results.len(), 1);
}

/// Tests the additional house numbers page: if the output is well-formed, no ref housenumbers case.
#[test]
fn test_additional_housenumbers_no_ref_housenumbers_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let hide_path = test_wsgi
        .get_ctx()
        .get_abspath("workdir/street-housenumbers-reference-gazdagret.lst");
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-ref-housenumbers']");
    assert_eq!(results.len(), 1);
}

/// Tests the additional streets page: if the output is well-formed.
#[test]
fn test_streets_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "Misspelled OSM Name 1": "OSM Name 1",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-additional-streets.count", &count_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

    let mut guard = count_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let mut results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
    // refstreets: >0 invalid osm name
    results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='osm-invalids-container']");
    assert_eq!(results.len(), 1);
    // refstreets: >0 invalid ref name
    results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']");
    assert_eq!(results.len(), 1);
}

/// Tests the additional streets page: if the output is well-formed when the street name comes
/// from a housenr.
#[test]
fn test_streets_street_from_housenr_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh611": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gh611-additional-streets.count", &count_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/additional-streets/gh611/view-result");

    let mut guard = count_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the additional streets page: if the output is well-formed, no osm streets case.
#[test]
fn test_streets_no_osm_streets_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let hide_path = test_wsgi
        .get_ctx()
        .get_abspath("workdir/streets-gazdagret.csv");
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the additional streets page: if the output is well-formed, no ref streets case.
#[test]
fn test_streets_no_ref_streets_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let hide_path = test_wsgi
        .get_ctx()
        .get_abspath("workdir/streets-reference-gazdagret.lst");
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-ref-streets']");
    assert_eq!(results.len(), 1)
}
