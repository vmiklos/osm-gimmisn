/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi_additional module.

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
            "refcounty": "01",
            "refsettlement": "011",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Törökugrató utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Tűzkő utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Ref Name 1');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref Nonsense utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Hamzsabégi út');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '3', 'Tűzkő utca', '9', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '4', 'Tűzkő utca', '10', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '5', 'OSM Name 1', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '6', 'OSM Name 1', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '7', 'Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '8', 'Second Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/gazdagret', '0');",
        )
        .unwrap();
    }

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
            "refcounty": "01",
            "refsettlement": "011",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Törökugrató utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Tűzkő utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Ref Name 1');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref Nonsense utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Hamzsabégi út');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '3', 'Tűzkő utca', '9', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '4', 'Tűzkő utca', '10', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '5', 'OSM Name 1', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '6', 'OSM Name 1', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '7', 'Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '8', 'Second Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/gazdagret', '0');",
        )
        .unwrap();
    }

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
            "refcounty": "01",
            "refsettlement": "011",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Törökugrató utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Tűzkő utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Ref Name 1');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref Nonsense utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Hamzsabégi út');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '3', 'Tűzkő utca', '9', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '4', 'Tűzkő utca', '10', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '5', 'OSM Name 1', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '6', 'OSM Name 1', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '7', 'Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '8', 'Second Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/gazdagret', '0');",
        )
        .unwrap();
    }

    let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.chkl");

    assert_eq!(result, "[ ] Only In OSM utca\n");
}

/// Tests additional streets: the txt output, no osm streets case.
#[test]
fn test_streets_view_result_txt_no_osm_streets() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();

    let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

    assert_eq!(result, "No existing streets");
}

/// Tests additional streets: if the view-turbo output is well-formed.
#[test]
fn test_streets_view_turbo_well_formed() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "01",
                "refsettlement": "011",
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
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into additional_housenumbers_counts (relation, count) values ('budafok', '42');
             insert into osm_housenumber_coverages (relation_name, coverage, last_modified) values ('budafok', '100.00', '0');",
        )
        .unwrap();
    }
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

/// Tests handle_main_housenr_additional_count(): what happens when the count row is not there.
#[test]
fn test_handle_main_housenr_additional_count_no_count_file() {
    let ctx = context::tests::make_test_context().unwrap();
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
                "refcounty": "0",
                "refsettlement": "0",
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
            (
                "workdir/gazdagret-additional-housenumbers.count",
                &count_value,
            ),
        ],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Hamzsabégi út', '1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Ref Name 1', '1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Ref Name 1', '2', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Törökugrató utca', '1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Törökugrató utca', '10', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Törökugrató utca', '11', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Törökugrató utca', '12', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Törökugrató utca', '2', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Törökugrató utca', '7', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '10', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '2', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'my street', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '3', 'Tűzkő utca', '9', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '4', 'Tűzkő utca', '10', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '5', 'OSM Name 1', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '6', 'OSM Name 1', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '7', 'Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '8', 'Second Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/gazdagret', '0');",
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the additional house numbers page: if the output is well-formed, no osm streets case.
#[test]
fn test_additional_housenumbers_no_osm_streets_well_formed() {
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

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the additional house numbers page: if the output is well-formed, no osm housenumbers case.
#[test]
fn test_additional_housenumbers_no_osm_housenumbers_well_formed() {
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'my street', '', '', '', '', '');
            insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');",
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
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
            "refcounty": "01",
            "refsettlement": "011",
            "refstreets": {
                "Misspelled OSM Name 1": "OSM Name 1",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Törökugrató utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Tűzkő utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Ref Name 1');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Only In Ref Nonsense utca');
             insert into ref_streets (county_code, settlement_code, street) values ('01', '011', 'Hamzsabégi út');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');
             insert into mtimes (page, last_modified) values ('housenumbers/gazdagret', '0');",
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

    let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
    let count: String = conn
        .query_row(
            "select count from additional_streets_counts where relation = ?1",
            ["gazdagret"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, "1".to_string());
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
                "refcounty": "40",
                "refsettlement": "41",
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
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh611', '42', 'Street name', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gh611', '0');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gh611', '6852648009', 'Albert utca', '42', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/gh611', '0');",
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/additional-streets/gh611/view-result");

    let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
    let count: String = conn
        .query_row(
            "select count from additional_streets_counts where relation = ?1",
            ["gh611"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, "2".to_string());
    let results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the additional streets page: if the output is well-formed, no osm streets case.
#[test]
fn test_streets_no_osm_streets_well_formed() {
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

    let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
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
