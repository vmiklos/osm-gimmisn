/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi_json module.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Seek;
use std::io::SeekFrom;
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
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.csv",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.json",
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
    let housenumbers_value = context::tests::TestFileSystem::make_file();
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
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let housenumbers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &housenumbers_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "1", "Törökugrató utca", "1", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "2", "Törökugrató utca", "2", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "3", "Tűzkő utca", "9", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "4", "Tűzkő utca", "10", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "5", "OSM Name 1", "1", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "6", "OSM Name 1", "2", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "7", "Only In OSM utca", "1", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "8", "Second Only In OSM utca", "1", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
    }

    let root = test_wsgi.get_json_for_path("/missing-housenumbers/gazdagret/update-result.json");

    assert_eq!(root.as_object().unwrap()["error"], "");
    let mut guard = housenumbers_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests missing_streets_update_result_json().
#[test]
fn test_missing_streets_update_result_json() {
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
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
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

/// Tests missing_housenumbers_view_result_json().
#[test]
fn test_missing_housenumbers_view_result_json() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let json_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[("workdir/cache-budafok.json", &json_cache)],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi
            .get_ctx()
            .get_abspath("workdir/cache-budafok.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["budafok", "458338075", "Vöröskúti határsor", "", "", "", "", ""],
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
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        test_wsgi.get_ctx(),
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/additional-cache-budafok.json", &cache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi
            .get_ctx()
            .get_abspath("workdir/additional-cache-budafok.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

    let result = test_wsgi.get_json_for_path("/additional-housenumbers/budafok/view-result.json");

    // The json equivalent of test_additional_housenumbers_well_formed().
    let additional_housenumbers: util::NumberedStreets = serde_json::from_value(result).unwrap();
    assert_eq!(additional_housenumbers.len(), 0);
}
