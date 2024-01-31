/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the cron module.

use super::*;
use context::FileSystem;
use std::cell::RefCell;
use std::io::Seek;
use std::io::SeekFrom;
use std::rc::Rc;

/// Tests overpass_sleep(): the case when no sleep is needed.
#[test]
fn test_overpass_sleep_no_sleep() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);

    overpass_sleep(&ctx);

    let time = ctx
        .get_time()
        .as_any()
        .downcast_ref::<context::tests::TestTime>()
        .unwrap();
    assert_eq!(time.get_sleep(), 0);
}

/// Tests overpass_sleep(): the case when sleep is needed.
#[test]
fn test_overpass_sleep_need_sleep() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-wait.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);

    overpass_sleep(&ctx);

    let time = ctx
        .get_time()
        .as_any()
        .downcast_ref::<context::tests::TestTime>()
        .unwrap();
    assert_eq!(time.get_sleep(), 12);
}

/// Tests update_ref_housenumbers().
#[test]
fn test_update_ref_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refsettlement": "42",
                "refcounty": "01",
                "refsettlement": "011",
            },
            "ujbuda": {
                "refsettlement": "42",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
        "relation-ujbuda.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file1 = context::tests::TestFileSystem::make_file();
    let ref_file2 = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file1,
            ),
            (
                "workdir/street-housenumbers-reference-ujbuda.lst",
                &ref_file2,
            ),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst");
    mtimes.insert(
        path.to_string(),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "1", "Tűzkő utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "2", "Törökugrató utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "3", "OSM Name 1", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "4", "Hamzsabégi út", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["ujbuda", "3", "OSM Name 1", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["ujbuda", "2", "Törökugrató utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["ujbuda", "1", "Tűzkő utca", "", "", "", "", ""],
        )
        .unwrap();
    }
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_ref_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();
    assert!(mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_ref_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);
    let actual = context::tests::TestFileSystem::get_content(&ref_file1);
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected);
    // Make sure housenumber ref is not created for the streets=only case.
    let mut guard = ref_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
}

/// Tests update_ref_streets().
#[test]
fn test_update_ref_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refsettlement": "42",
                "refcounty": "01",
                "refsettlement": "011",
            },
            "gellerthegy": {
                "refsettlement": "42",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
        "relation-gellerthegy.yaml": {
            "missing-streets": "no",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let streets_ref_myrelation1 = context::tests::TestFileSystem::make_file();
    let streets_ref_myrelation2 = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/streets-reference-gazdagret.lst",
                &streets_ref_myrelation1,
            ),
            (
                "workdir/streets-reference-gellerthegy.lst",
                &streets_ref_myrelation2,
            ),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/streets-reference-gazdagret.lst");
    mtimes.insert(
        path.to_string(),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_ref_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();
    assert!(mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_ref_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);
    let actual = context::tests::TestFileSystem::get_content(&streets_ref_myrelation1);
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected);
    // Make sure street ref is not created for the streets=no case.
    let mut guard = streets_ref_myrelation2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
}

/// Tests update_missing_housenumbers().
#[test]
fn test_update_missing_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "ujbuda": {
                "osmrelation": 2702687,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "housenumber-letters": true,
        },
        "relation-ujbuda.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let json_cache = context::tests::TestFileSystem::make_file();
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/cache-gazdagret.json", &json_cache),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "Tűzkő utca\t1/A\t",
            &ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        )
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/cache-gazdagret.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mtime = ctx.get_time().now_string();
    {
        let conn = ctx.get_database_connection().unwrap();
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
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }
    let mut relations = areas::Relations::new(&ctx).unwrap();
    // Only one housenumber and it's missing.
    let expected: String = "0.00".into();
    let relation = relations.get_relation("gazdagret").unwrap();

    update_missing_housenumbers(&mut relations, /*update=*/ true).unwrap();

    let expected_mtime = relation.get_osm_housenumber_coverage_mtime().unwrap();
    assert!(expected_mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_missing_housenumbers(&mut relations, /*update=*/ false).unwrap();

    let actual_mtime = relation.get_osm_housenumber_coverage_mtime().unwrap();
    assert_eq!(actual_mtime, expected_mtime);
    let actual = relation.get_osm_housenumber_coverage().unwrap();
    assert_eq!(actual, expected);
    // Make sure housenumber stat is not created for the streets=only case.
    let relation2 = relations.get_relation("ujbuda").unwrap();
    assert_eq!(relation2.has_osm_housenumber_coverage().unwrap(), false);
}

/// Tests update_missing_streets().
#[test]
fn test_update_missing_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "gellerthegy": {
                "osmrelation": 2702687,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
        },
        "relation-gellerthegy.yaml": {
            "missing-streets": "no",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "1", "Tűzkő utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "2", "Törökugrató utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "3", "OSM Name 1", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "4", "Hamzsabégi út", "", "", "", "", ""],
        )
        .unwrap();
    }
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "50.00".into();
    let relation = relations.get_relation("gazdagret").unwrap();

    update_missing_streets(&mut relations, /*update=*/ true).unwrap();

    let expected_mtime = relation.get_osm_street_coverage_mtime().unwrap();
    assert!(expected_mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_missing_streets(&mut relations, /*update=*/ false).unwrap();

    let actual_mtime = relation.get_osm_street_coverage_mtime().unwrap();
    assert_eq!(actual_mtime, expected_mtime);
    let actual = relation.get_osm_street_coverage().unwrap();
    assert_eq!(actual, expected);
    // Make sure street stat is not created for the streets=no case.
    let relation2 = relations.get_relation("ujbuda").unwrap();
    assert_eq!(relation2.has_osm_street_coverage().unwrap(), false);
}

/// Tests update_additional_streets().
#[test]
fn test_update_additional_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "gellerthegy": {
                "osmrelation": 2702687,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "osm-street-filters": ["Second Only In OSM utca"],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
        "relation-gellerthegy.yaml": {
            "missing-streets": "no",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_file1 = context::tests::TestFileSystem::make_file();
    let count_file2 = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-additional-streets.count", &count_file1),
            ("workdir/gellerthegy-additional-streets.count", &count_file2),
        ],
    );
    let path1 = ctx.get_abspath("workdir/gazdagret-additional-streets.count");
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        path1.to_string(),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mtime = ctx.get_time().now_string();
    {
        let conn = ctx.get_database_connection().unwrap();
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
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "1".into();
    update_additional_streets(&ctx, &mut relations, /*update=*/ true).unwrap();
    let mtime = file_system_rc.getmtime(&path1).unwrap();

    update_additional_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(file_system_rc.getmtime(&path1).unwrap(), mtime);
    let actual = context::tests::TestFileSystem::get_content(&count_file1);
    assert_eq!(actual, expected);
    // Make sure street stat is not created for the streets=no case.
    let mut guard = count_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, false);
}

/// Tests update_osm_housenumbers().
#[test]
fn test_update_osm_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.json",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
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
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected = relations
        .get_relation("gazdagret")
        .unwrap()
        .get_files()
        .get_osm_json_housenumbers(&ctx)
        .unwrap()
        .len();

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = stats::get_sql_mtime(&ctx, "housenumbers/gazdagret").unwrap();

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(
        stats::get_sql_mtime(&ctx, "housenumbers/gazdagret").unwrap(),
        mtime
    );
    let actual = relations
        .get_relation("gazdagret")
        .unwrap()
        .get_files()
        .get_osm_json_housenumbers(&ctx)
        .unwrap()
        .len();
    assert_eq!(actual, expected);
}

/// Tests update_osm_housenumbers(): the case when we keep getting HTTP errors.
#[test]
fn test_update_osm_housenumbers_http_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
    ctx.set_network(network_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "1", "Tűzkő utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "1", "Törökugrató utca", "1", "", "", "", "", "", "", "", "", "", "node"],
        )
        .unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();
    // Make sure that in case we keep getting errors we give up at some stage and
    // leave the last state unchanged.
    assert_eq!(
        relations
            .get_relation("gazdagret")
            .unwrap()
            .get_files()
            .get_osm_json_housenumbers(&ctx)
            .unwrap()
            .len(),
        1
    );
}

/// Tests update_osm_housenumbers(): the case when we ask for JSON but get XML.
#[test]
fn test_update_osm_housenumbers_xml_as_json() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
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
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass.xml",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gazdagret", "42", "my street", "1", "", "", "", "", "", "", "", "", "", "way"],
        )
        .unwrap();
    }

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    // Wanted JSON, got XML, make sure the db is left unchanged.
    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
        .prepare("select count(*) from osm_housenumbers")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let count: i64 = row.get(0).unwrap();
    assert_eq!(count, 1);
}

/// Tests update_osm_streets().
#[test]
fn test_update_osm_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-streets-gazdagret.json",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
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
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = stats::get_sql_mtime(&ctx, "streets/gazdagret").unwrap();
    assert!(mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_osm_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(
        stats::get_sql_mtime(&ctx, "streets/gazdagret").unwrap(),
        mtime
    );

    assert_eq!(
        relations
            .get_relation("gazdagret")
            .unwrap()
            .get_files()
            .get_osm_json_streets(&ctx)
            .unwrap()
            .len(),
        4
    );
}

/// Tests update_osm_streets(): the case when we keep getting HTTP errors.
#[test]
fn test_update_osm_streets_http_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
    ctx.set_network(network_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "1", "Tűzkő utca", "", "", "", "", ""],
        )
        .unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &overpass_template),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    // Make sure that in case we keep getting errors we give up at some stage and
    // leave the last state unchanged.
    assert_eq!(
        relations
            .get_relation("gazdagret")
            .unwrap()
            .get_files()
            .get_osm_json_streets(&ctx)
            .unwrap()
            .len(),
        1
    );
}

/// Tests update_osm_streets(): the case when we ask for JSON but get XML.
#[test]
fn test_update_osm_streets_xml_as_json() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
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
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass.xml",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            ["myrelation", &"1".to_string(), "myname", "myhighway", "myservice", "mysurface", "myleisure", "way"],
        )
        .unwrap();
    }

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    // Wanted JSON, got XML, make sure the db is left unchanged.
    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn.prepare("select count(*) from osm_streets").unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let count: i64 = row.get(0).unwrap();
    assert_eq!(count, 1);
}

/// Tests update_stats().
#[test]
fn test_update_stats() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-stats.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);

    let csv_value = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let stats_json = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/whole-country.csv", &csv_value),
            ("workdir/stats/ref.count", &ref_count),
            ("workdir/stats/stats.json", &stats_json),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let path = ctx.get_abspath(&format!("workdir/stats/whole-country.csv"));

    update_stats(&ctx, /*overpass=*/ true).unwrap();

    let actual = ctx.get_file_system().read_to_string(&path).unwrap();
    assert_eq!(
        actual,
        String::from_utf8(std::fs::read("src/fixtures/network/overpass-stats.csv").unwrap())
            .unwrap()
    );

    let num_ref: i64 = ctx
        .get_file_system()
        .read_to_string(&ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(num_ref, 300);
}

/// Tests update_stats(): the case when we keep getting HTTP errors.
#[test]
fn test_update_stats_http_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);

    let count_value = context::tests::TestFileSystem::make_file();
    count_value
        .borrow_mut()
        .write_all("254651\n".as_bytes())
        .unwrap();
    let ref_count = context::tests::TestFileSystem::make_file();
    let stats_json = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/ref.count", &ref_count),
            ("workdir/stats/stats.json", &stats_json),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    update_stats(&ctx, /*overpass=*/ true).unwrap();

    {
        let mut guard = stats_json.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
}

/// Tests update_stats(): the case when we don't call overpass.
#[test]
fn test_update_stats_no_overpass() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-wait.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);

    let ref_count = context::tests::TestFileSystem::make_file();
    let stats_json = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/ref.count", &ref_count),
            ("workdir/stats/stats.json", &stats_json),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    update_stats(&ctx, /*overpass=*/ false).unwrap();

    let time = ctx
        .get_time()
        .as_any()
        .downcast_ref::<context::tests::TestTime>()
        .unwrap();
    assert_eq!(time.get_sleep(), 0);
    let actual = ctx
        .get_file_system()
        .read_to_string(&ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap();
    // Same as in test_update_stats().
    assert_eq!(actual, "300\n");
}

/// Tests our_main().
#[test]
fn test_our_main() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-streets-gazdagret.json",
        ),
        // For update_osm_housenumbers().
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.json",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_streets_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_value = context::tests::TestFileSystem::make_file();
    let additional_streets_value = context::tests::TestFileSystem::make_file();
    let missing_housenumbers_json = context::tests::TestFileSystem::make_file();
    let template_value = context::tests::TestFileSystem::make_file();
    template_value
        .borrow_mut()
        .write_all(b"aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let housenr_template = context::tests::TestFileSystem::make_file();
    housenr_template
        .borrow_mut()
        .write_all(b"housenr aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/streets-reference-gazdagret.lst",
                &ref_streets_value,
            ),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers_value,
            ),
            (
                "workdir/gazdagret-additional-streets.count",
                &additional_streets_value,
            ),
            ("workdir/cache-gazdagret.json", &missing_housenumbers_json),
            ("data/streets-template.overpassql", &template_value),
            (
                "data/street-housenumbers-template.overpassql",
                &housenr_template,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/cache-gazdagret.json");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mtime = ctx.get_time().now_string();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "1", "Tűzkő utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "2", "Törökugrató utca", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "3", "OSM Name 1", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["gazdagret", "4", "Hamzsabégi út", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/gazdagret", &mtime],
        )
        .unwrap();
    }
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    our_main_inner(
        &ctx,
        &mut relations,
        /*mode=*/ &"relations".to_string(),
        /*update=*/ true,
        /*overpass=*/ true,
    )
    .unwrap();

    // update_osm_streets() is called.
    {
        let mtime = stats::get_sql_mtime(&ctx, "streets/gazdagret").unwrap();
        assert!(mtime > time::OffsetDateTime::UNIX_EPOCH);
    }
    // update_osm_housenumbers() is called.
    assert_eq!(
        relation
            .get_files()
            .get_osm_json_streets(&ctx)
            .unwrap()
            .is_empty(),
        false
    );
    // update_ref_streets() is called.
    {
        let mut guard = ref_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_ref_housenumbers() is called.
    {
        let mut guard = ref_housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_missing_streets() is called.
    assert_eq!(relation.has_osm_street_coverage().unwrap(), true);
    // update_missing_housenumbers() is called.
    assert_eq!(relation.has_osm_housenumber_coverage().unwrap(), true);
    // update_additional_streets() is called.
    {
        let mut guard = additional_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
}

/// Tests our_main(): the stats case.
#[test]
fn test_our_main_stats() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-stats.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let mut file_system = context::tests::TestFileSystem::new();
    let stats_value = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let today_csv = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/stats.json", &stats_value),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
            ("workdir/stats/whole-country.csv", &today_csv),
            ("workdir/stats/ref.count", &ref_count),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/stats/whole-country.csv");
    mtimes.insert(path, Rc::new(RefCell::new(ctx.get_time().now())));
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    our_main_inner(
        &ctx,
        &mut relations,
        /*mode=*/ &"stats".to_string(),
        /*update=*/ false,
        /*overpass=*/ true,
    )
    .unwrap();

    let mut guard = stats_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests main().
#[test]
fn test_main() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let stats_value = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/stats.json", &stats_value),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
            ("workdir/stats/ref.count", &ref_count),
        ],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let argv = vec![
        "".to_string(),
        "--mode".to_string(),
        "stats".to_string(),
        "--no-overpass".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 0);
    // Make sure that stats.json is updated.
    let mut guard = stats_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);

    let actual = ctx
        .get_file_system()
        .read_to_string(&ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap();
    // Same as in test_update_stats().
    assert_eq!(actual, "300\n");
}

/// Tests main(): the path when our_main() returns an error.
#[test]
fn test_main_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_rc: Rc<dyn context::Unit> = Rc::new(unit);
    ctx.set_unit(&unit_rc);
    let ref_count = context::tests::TestFileSystem::make_file();
    let stats_json = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/ref.count", &ref_count),
            ("workdir/stats/stats.json", &stats_json),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string("300", &ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap();
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let argv = vec![
        "".to_string(),
        "--mode".to_string(),
        "stats".to_string(),
        "--no-overpass".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());

    // main() catches the error returned by our_main().
    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 1);
}

/// Tests update_stats_count().
#[test]
fn test_update_stats_count() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_csv_value = context::tests::TestFileSystem::make_file();
    today_csv_value
        .borrow_mut()
        .write_all(
            r#"addr:postcode	addr:city	addr:street	addr:housenumber	@user	@id	@type	@timestamp	fixme
7677	Orfű	Dollár utca	1	mgpx	42	way	2020-05-10T22:02:25Z	
"#
            .as_bytes(),
        )
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/whole-country.csv", &today_csv_value)],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    {
        let conn = ctx.get_database_connection().unwrap();
        {
            let mut stmt = conn
                .prepare("select count from stats_counts where date = ?1")
                .unwrap();
            let mut counts = stmt.query(["2020-05-10"]).unwrap();
            assert!(counts.next().unwrap().is_some());
        }
        let mut stmt = conn
            .prepare("select date, count from stats_counts")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let date: String = row.get(0).unwrap();
            assert_eq!(date, "2020-05-10");
            let count: String = row.get(1).unwrap();
            let count: i64 = count.parse().unwrap();
            assert_eq!(count, 1);
        }
    }
    {
        let conn = ctx.get_database_connection().unwrap();
        let mut stmt = conn
            .prepare("select count from stats_citycounts where date = ?1")
            .unwrap();
        let mut citycounts = stmt.query(["2020-05-10"]).unwrap();
        let citycount = citycounts.next().unwrap();
        assert!(citycount.is_some());
    }
    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
        .prepare("select count from stats_zipcounts where date = ?1")
        .unwrap();
    let mut zipcounts = stmt.query(["2020-05-10"]).unwrap();
    let zipcount = zipcounts.next().unwrap();
    assert!(zipcount.is_some());
}

/// Tests update_stats_count(): the case then the .csv is missing.
#[test]
fn test_update_stats_count_no_csv() {
    let ctx = context::tests::make_test_context().unwrap();

    update_stats_count(&ctx, "2020-05-10").unwrap();

    // No .csv, no count or citycount.
    let conn = ctx.get_database_connection().unwrap();
    {
        let mut stmt = conn
            .prepare("select count from stats_counts where date = ?1")
            .unwrap();
        let mut counts = stmt.query(["2020-05-10"]).unwrap();
        assert!(counts.next().unwrap().is_none());
    }
    {
        let mut stmt = conn
            .prepare("select count from stats_zipcounts where date = ?1")
            .unwrap();
        let mut zipcounts = stmt.query(["2020-05-10"]).unwrap();
        assert!(zipcounts.next().unwrap().is_none());
    }
}

/// Tests update_stats_count(): the case when we ask for CSV but get XML.
#[test]
fn test_update_stats_count_xml_as_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_csv_value = context::tests::TestFileSystem::make_file();
    today_csv_value
        .borrow_mut()
        .write_all("<?xml\n".as_bytes())
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/whole-country.csv", &today_csv_value)],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
        .prepare("select count from stats_counts where date = ?1")
        .unwrap();
    let mut counts = stmt.query(["2020-05-10"]).unwrap();
    let count = counts.next().unwrap().unwrap();
    let actual: String = count.get(0).unwrap();
    assert_eq!(actual, "0");
}

/// Tests update_stats_topusers().
#[test]
fn test_update_stats_topusers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_csv_value = context::tests::TestFileSystem::make_file();
    today_csv_value
        .borrow_mut()
        .write_all(
            r#"addr:postcode	addr:city	addr:street	addr:housenumber	@user	@id	@type	@timestamp	fixme
1234	mycity	mystreet1	1	myuser1	42	way	2020-05-10T22:02:25Z	
1234	mycity	mystreet1	2	myuser1	43	way	2020-05-10T22:02:25Z	
1234	mycity	mystreet2	1	myuser2	44	way	2020-05-10T22:02:25Z	
"#
            .as_bytes(),
        )
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/whole-country.csv", &today_csv_value)],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    update_stats_topusers(&ctx, "2020-05-10").unwrap();

    {
        let conn = ctx.get_database_connection().unwrap();
        let mut stmt = conn
            .prepare(
                "select user, count from stats_topusers where date = ?1 order by cast(count as integer) desc",
            )
            .unwrap();
        let mut rows = stmt.query(["2020-05-10"]).unwrap();
        let row = rows.next().unwrap().unwrap();
        let user: String = row.get(0).unwrap();
        assert_eq!(user, "myuser1");
        let count: String = row.get(1).unwrap();
        assert_eq!(count, "2");
        let row = rows.next().unwrap().unwrap();
        let user: String = row.get(0).unwrap();
        assert_eq!(user, "myuser2");
        let count: String = row.get(1).unwrap();
        assert_eq!(count, "1");
    }
    {
        let conn = ctx.get_database_connection().unwrap();
        let mut stmt = conn
            .prepare("select count from stats_usercounts where date = ?1")
            .unwrap();
        let mut usercounts = stmt.query(["2020-05-10"]).unwrap();
        let usercount = usercounts.next().unwrap();
        assert!(usercount.is_some());
    }

    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
        .prepare("select date, count from stats_usercounts")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        let date: String = row.get(0).unwrap();
        assert_eq!(date, "2020-05-10");
        let count: String = row.get(1).unwrap();
        let count: i64 = count.parse().unwrap();
        assert_eq!(count, 2);
    }
}

/// Tests update_stats_topusers(): the case then the .csv is missing.
#[test]
fn test_update_stats_topusers_no_csv() {
    let ctx = context::tests::make_test_context().unwrap();

    update_stats_topusers(&ctx, "2020-05-10").unwrap();

    // No .csv, no topusers or usercount.
    {
        let conn = ctx.get_database_connection().unwrap();
        let mut stmt = conn
            .prepare("select count from stats_topusers where date = ?1")
            .unwrap();
        let mut topusers = stmt.query(["2020-05-10"]).unwrap();
        let row = topusers.next().unwrap();
        assert!(row.is_none());
    }
    {
        let conn = ctx.get_database_connection().unwrap();
        let mut stmt = conn
            .prepare("select count from stats_usercounts where date = ?1")
            .unwrap();
        let mut usercounts = stmt.query(["2020-05-10"]).unwrap();
        let usercount = usercounts.next().unwrap();
        assert!(usercount.is_none());
    }
}

/// Tests write_city_count_path().
#[test]
fn test_write_city_count_path() {
    let ctx = context::tests::make_test_context().unwrap();
    let city1: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let city2: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let cities: HashMap<String, HashSet<String>> = [
        ("mycity2".to_string(), city2),
        ("mycity1".to_string(), city1),
    ]
    .into_iter()
    .collect();

    write_city_count_path(&ctx, &cities).unwrap();

    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
                .prepare("select city, count from stats_citycounts where date = ?1 order by cast(count as integer) desc")
                .unwrap();
    let mut rows = stmt.query(["2020-05-10"]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let zip: String = row.get(0).unwrap();
    assert_eq!(zip, "mycity1");
    let count: String = row.get(1).unwrap();
    assert_eq!(count, "2");
    let row = rows.next().unwrap().unwrap();
    let zip: String = row.get(0).unwrap();
    assert_eq!(zip, "mycity2");
    let count: String = row.get(1).unwrap();
    assert_eq!(count, "2");
    assert!(rows.next().unwrap().is_none());
}

/// Tests write_zip_count_path().
#[test]
fn test_write_zip_count_path() {
    let ctx = context::tests::make_test_context().unwrap();
    let zip1: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let zip2: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let cities: HashMap<String, HashSet<String>> =
        [("myzip2".to_string(), zip2), ("myzip1".to_string(), zip1)]
            .into_iter()
            .collect();

    write_zip_count_path(&ctx, &cities).unwrap();

    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
                .prepare("select zip, count from stats_zipcounts where date = ?1 order by cast(count as integer) desc")
                .unwrap();
    let mut rows = stmt.query(["2020-05-10"]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let zip: String = row.get(0).unwrap();
    assert_eq!(zip, "myzip1");
    let count: String = row.get(1).unwrap();
    assert_eq!(count, "2");
    let row = rows.next().unwrap().unwrap();
    let zip: String = row.get(0).unwrap();
    assert_eq!(zip, "myzip2");
    let count: String = row.get(1).unwrap();
    assert_eq!(count, "2");
    assert!(rows.next().unwrap().is_none());
}
