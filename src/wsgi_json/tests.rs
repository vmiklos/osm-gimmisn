/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi_json module.

use std::io::Seek;
use std::io::SeekFrom;
use std::sync::Arc;

use crate::context;
use crate::wsgi;

/// Tests streets_update_result_json(): if the update-result json output is well-formed.
#[test]
fn test_json_streets_update_result() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "tests/network/overpass-streets-gazdagret.csv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.get_ctx().set_network(&network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-myrelation.csv", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_json_for_path("/streets/myrelation/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests streets_update_result_json(): if the update-result json output on error is well-formed.
#[test]
fn test_json_streets_update_result_error() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.get_ctx().set_network(&network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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

    let root = test_wsgi.get_json_for_path("/streets/myrelation/update-result.json");

    let error = root.as_object().unwrap()["error"].as_str().unwrap();
    assert_eq!(error.is_empty(), false);
}

/// Tests street_housenumbers_update_result_json(): if the update-result output is well-formed.
#[test]
fn test_json_housenumbers_update_result() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "tests/network/overpass-housenumbers-gazdagret.csv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.get_ctx().set_network(&network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let housenumbers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &housenumbers_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_json_for_path("/street-housenumbers/gazdagret/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let mut guard = housenumbers_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests street_housenumbers_update_result_json(): if the update-result output on error is
/// well-formed.
#[test]
fn test_json_housenumbers_update_result_error() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.get_ctx().set_network(&network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
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

    let root = test_wsgi.get_json_for_path("/street-housenumbers/gazdagret/update-result.json");

    let error = root.as_object().unwrap()["error"].as_str().unwrap();
    assert_eq!(error.is_empty(), false);
}

/// Tests missing_housenumbers_update_result_json().
#[test]
fn test_missing_housenumbers_update_result_json() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let ref_housenumbers_cache = context::tests::TestFileSystem::make_file();
    let ref_housenumbers2_cache = context::tests::TestFileSystem::make_file();
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let housenumbers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "refdir/hazszamok_20190511.tsv-01-v1.cache",
                &ref_housenumbers_cache,
            ),
            (
                "refdir/hazszamok_kieg_20190808.tsv-01-v1.cache",
                &ref_housenumbers2_cache,
            ),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &housenumbers_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_json_for_path("/missing-housenumbers/gazdagret/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let mut guard = housenumbers_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests missing_streets_update_result_json().
#[test]
fn test_missing_streets_update_result_json() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-reference-gazdagret.lst", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_json_for_path("/missing-streets/gazdagret/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}
