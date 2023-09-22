/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
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

use crate::areas;
use crate::context;
use crate::wsgi;

use super::*;

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

/// Tests additional streets: the gpx output.
#[test]
fn test_streets_view_result_gpx() {
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
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "src/fixtures/network/overpass-additional-streets.overpassql",
        /*result_path=*/ "src/fixtures/network/overpass-additional-streets.json",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
    test_wsgi.set_content_type("text/gpx+xml; charset=utf-8");

    let _root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result.gpx");

    // TODO assert that there are two results here
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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let mut file_system = context::tests::TestFileSystem::new();
    let hide_path = ctx.get_abspath("workdir/budafok-additional-housenumbers.count");
    file_system.set_hide_paths(&[hide_path]);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("budafok").unwrap();

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
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/gazdagret-additional-housenumbers.count",
                &count_value,
            ),
            ("workdir/additional-cache-gazdagret.json", &cache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi
            .get_ctx()
            .get_abspath("workdir/additional-cache-gazdagret.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-ref-streets']");
    assert_eq!(results.len(), 1)
}

/// Tests get_gpx_street_lat_lon(), the case when a "street" is a node.
#[test]
fn test_get_gpx_street_lat_lon_node() {
    let json = serde_json::json!({
        "elements": [
            {
                "type": "node",
                "id": 42,
                "lat": 47,
                "lon": 18,
                "tags": {
                    "addr:city": "mycity",
                    "addr:housenumber": "43",
                    "addr:postcode": "1234",
                    "addr:street": "mystreet",
                },
            },
        ],
    });
    let overpass: OverpassResult = serde_json::from_value(json).unwrap();
    let element = &overpass.elements[0];
    let (lat, lon) = get_gpx_street_lat_lon(&overpass, &element).unwrap();
    assert_eq!(lat, "47");
    assert_eq!(lon, "18");
}

/// Tests get_gpx_street_lat_lon(), the case when a "street" is a relation.
#[test]
fn test_get_gpx_street_lat_lon_relation() {
    let json = serde_json::json!({
        "elements": [
            {
                "type": "relation",
                "id": 2262333,
                "members": [
                    {
                        "ref": 366696002,
                    },
                ],
            },
            {
                "type": "way",
                "id": 366696002,
                "nodes": [
                    370687421,
                ]
            },
            {
                "type": "node",
                "id": 370687421,
                "lat": 47.0273397,
                "lon": 18.0187039
            },
        ]
    });
    let overpass: OverpassResult = serde_json::from_value(json).unwrap();
    let element = &overpass.elements[0];
    let (lat, lon) = get_gpx_street_lat_lon(&overpass, &element).unwrap();
    assert_eq!(lat, "47.0273397");
    assert_eq!(lon, "18.0187039");
}
