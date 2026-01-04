/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi_json module.

use std::io::Write as _;
use std::rc::Rc;

use crate::areas;
use crate::context;
use crate::util;
use crate::wsgi;

/// Tests streets_update_result_json(): if the update-result json output is well-formed.
#[test]
fn test_json_streets_update_result() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-streets-gazdagret.json",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
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
    let template_value = context::tests::TestFileSystem::make_file();
    template_value
        .borrow_mut()
        .write_all(b"aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_json_for_path("/streets/myrelation/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let ctx = test_wsgi.get_ctx();
    let mut relations = areas::Relations::new(ctx).unwrap();
    assert_eq!(
        relations
            .get_relation("myrelation")
            .unwrap()
            .get_files()
            .get_osm_json_streets(ctx)
            .unwrap()
            .len(),
        4
    );
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
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let template_value = context::tests::TestFileSystem::make_file();
    template_value
        .borrow_mut()
        .write_all(b"aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
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
        /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.json",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
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
    let overpass_template = context::tests::TestFileSystem::make_file();
    overpass_template
        .borrow_mut()
        .write_all(b"housenr aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');"
        )
        .unwrap();
    }

    let root = test_wsgi.get_json_for_path("/street-housenumbers/gazdagret/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let ctx = test_wsgi.get_ctx();
    let mut relations = areas::Relations::new(ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    assert_eq!(
        relation
            .get_files()
            .get_osm_json_streets(ctx)
            .unwrap()
            .len(),
        4
    );
}

/// Tests street_housenumbers_update_result_json(): if the update-result output on error is
/// well-formed.
#[test]
fn test_json_housenumbers_update_result_error() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
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
    let overpass_template = context::tests::TestFileSystem::make_file();
    overpass_template
        .borrow_mut()
        .write_all(b"housenr aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_json_for_path("/street-housenumbers/gazdagret/update-result.json");

    let error = root.as_object().unwrap()["error"].as_str().unwrap();
    assert_eq!(error.is_empty(), false);
}

/// Tests street_housenumbers_update_result_json(): the case when it has to re-try the network
/// request to succeed.
#[test]
fn test_street_housenumbers_update_result_json_retry() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-duplicate.json",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let overpass_template = context::tests::TestFileSystem::make_file();
    overpass_template
        .borrow_mut()
        .write_all(b"housenr aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);
    let root = test_wsgi.get_json_for_path("/street-housenumbers/myrelation/update-result.json");

    // Without the fix, this was:
    // String("empty result_path for url 'https://overpass-api.de/api/interpreter'")
    assert_eq!(root.as_object().unwrap()["error"], "");
}

/// Tests missing_housenumbers_view_result_json().
#[test]
fn test_missing_housenumbers_view_result_json() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
                "refcounty": "0",
                "refsettlement": "0",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '34', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '36', ' ');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '12', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '2', '');",
         )
         .unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('budafok', '458338075', 'Vöröskúti határsor', '', '', '', '', '');"
        )
        .unwrap();
    }

    let result = test_wsgi.get_json_for_path("/missing-housenumbers/budafok/view-result.json");

    // The json equivalent of test_missing_housenumbers_view_result_txt().
    let missing_housenumbers: areas::MissingHousenumbers = serde_json::from_value(result).unwrap();
    assert_eq!(missing_housenumbers.ongoing_streets.len(), 1);
    let ongoing_street = &missing_housenumbers.ongoing_streets[0];
    assert_eq!(ongoing_street.street.get_osm_name(), "Vöröskúti határsor");
    // 2, 12, 34, 36.
    assert_eq!(ongoing_street.house_numbers.len(), 4);
}

/// Tests additional_housenumbers_view_result_json().
#[test]
fn test_additional_housenumbers_view_result_json() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '34', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '36', ' ');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '12', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '2', '');",
         )
         .unwrap();
    }

    let result = test_wsgi.get_json_for_path("/additional-housenumbers/budafok/view-result.json");

    // The json equivalent of test_additional_housenumbers_well_formed().
    let additional_housenumbers: util::NumberedStreets = serde_json::from_value(result).unwrap();
    assert_eq!(additional_housenumbers.len(), 0);
}

/// Tests the /api part of our_application_json(), the relations + some case.
#[test]
fn test_our_application_json_api_relations_some() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into stats_jsons (category, json) values ('relations', '[1, 2]');",
        )
        .unwrap();
    }

    let result = test_wsgi.get_json_for_path("/api/relations.json");

    let relations: Vec<u64> = serde_json::from_value(result).unwrap();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0], 1);
    assert_eq!(relations[1], 2);
}

/// Tests the /api part of our_application_json(), the relations + none case.
#[test]
fn test_our_application_json_api_relations_none() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();

    let result = test_wsgi.get_json_for_path("/api/relations.json");

    let relations: Vec<u64> = serde_json::from_value(result).unwrap();
    assert_eq!(relations.len(), 0);
}
