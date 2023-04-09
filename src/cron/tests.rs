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
use std::sync::Arc;

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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);

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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);

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
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
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
            ("workdir/refs/utcak_20190514.tsv.cache", &ref_streets_cache),
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
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let count_file1 = context::tests::TestFileSystem::make_file();
    let count_file2 = context::tests::TestFileSystem::make_file();
    let json_cache = context::tests::TestFileSystem::make_file();
    let ref_housenumbers = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.percent", &count_file1),
            ("workdir/ujbuda.percent", &count_file2),
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
    let path1 = ctx.get_abspath("workdir/gazdagret.percent");
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        path1.to_string(),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/cache-gazdagret.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    // Only one housenumber and it's missing.
    let expected: String = "0.00".into();

    update_missing_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let expected_mtime = file_system_arc.getmtime(&path1).unwrap();
    assert!(expected_mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_missing_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    let actual_mtime = file_system_arc.getmtime(&path1).unwrap();
    assert_eq!(actual_mtime, expected_mtime);
    let actual = context::tests::TestFileSystem::get_content(&count_file1);
    assert_eq!(actual, expected);
    // Make sure housenumber stat is not created for the streets=only case.
    let mut guard = count_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, false);
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
    let count_file1 = context::tests::TestFileSystem::make_file();
    let count_file2 = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-streets.percent", &count_file1),
            ("workdir/gellerthegy-streets.percent", &count_file2),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path1 = ctx.get_abspath("workdir/gazdagret-streets.percent");
    mtimes.insert(
        path1.to_string(),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "50.00".into();

    update_missing_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let expected_mtime = ctx.get_file_system().getmtime(&path1).unwrap();
    assert!(expected_mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_missing_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    let actual_mtime = ctx.get_file_system().getmtime(&path1).unwrap();
    assert_eq!(actual_mtime, expected_mtime);
    let actual = context::tests::TestFileSystem::get_content(&count_file1);
    assert_eq!(actual, expected);
    // Make sure street stat is not created for the streets=no case.
    let mut guard = count_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, false);
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
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "1".into();
    update_additional_streets(&ctx, &mut relations, /*update=*/ true).unwrap();
    let mtime = file_system_arc.getmtime(&path1).unwrap();

    update_additional_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(file_system_arc.getmtime(&path1).unwrap(), mtime);
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
    let osm_housenumbers_value = context::tests::TestFileSystem::make_file();
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
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &osm_housenumbers_value,
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
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv");
    let expected = std::fs::read_to_string(&path).unwrap();

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();
    // Make sure that in case we keep getting errors we give up at some stage and
    // leave the last state unchanged.
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_housenumbers(): the case when we ask for CSV but get XML.
#[test]
fn test_update_osm_housenumbers_xml_as_csv() {
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
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
            /*result_path=*/ "src/fixtures/network/overpass-streets-gazdagret.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let osm_streets_value = context::tests::TestFileSystem::make_file();
    let template_value = context::tests::TestFileSystem::make_file();
    template_value
        .borrow_mut()
        .write_all(b"aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-gazdagret.csv", &osm_streets_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/streets-gazdagret.csv");
    mtimes.insert(
        path.to_string(),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();
    assert!(mtime > time::OffsetDateTime::UNIX_EPOCH);

    update_osm_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);

    let actual = context::tests::TestFileSystem::get_content(&osm_streets_value);
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected);
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/streets-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    // Make sure that in case we keep getting errors we give up at some stage and
    // leave the last state unchanged.
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_streets(): the case when we ask for CSV but get XML.
#[test]
fn test_update_osm_streets_xml_as_csv() {
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/streets-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);

    let citycount_value = context::tests::TestFileSystem::make_file();
    let zipcount_value = context::tests::TestFileSystem::make_file();
    let count_value = context::tests::TestFileSystem::make_file();
    let topusers_value = context::tests::TestFileSystem::make_file();
    let csv_value = context::tests::TestFileSystem::make_file();
    let usercount_value = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let stats_json = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let old_csv = context::tests::TestFileSystem::make_file();
    let old_path = "workdir/stats/old.csv";
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.citycount", &citycount_value),
            ("workdir/stats/2020-05-10.zipcount", &zipcount_value),
            ("workdir/stats/2020-05-10.count", &count_value),
            ("workdir/stats/2020-05-10.topusers", &topusers_value),
            ("workdir/stats/2020-05-10.csv", &csv_value),
            ("workdir/stats/2020-05-10.usercount", &usercount_value),
            ("workdir/stats/ref.count", &ref_count),
            ("workdir/stats/stats.json", &stats_json),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
            (old_path, &old_csv),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/stats/2020-05-10.csv");
    mtimes.insert(path, Rc::new(RefCell::new(ctx.get_time().now())));
    let path = ctx.get_abspath("workdir/stats/old.csv");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]").unwrap();
    let today = now.format(&format).unwrap();
    let path = ctx.get_abspath(&format!("workdir/stats/{today}.csv"));

    update_stats(&ctx, /*overpass=*/ true).unwrap();

    let actual = ctx.get_file_system().read_to_string(&path).unwrap();
    assert_eq!(
        actual,
        String::from_utf8(std::fs::read("src/fixtures/network/overpass-stats.csv").unwrap())
            .unwrap()
    );

    // Make sure that the old CSV is removed.
    assert_eq!(
        ctx.get_file_system()
            .path_exists(&ctx.get_abspath(old_path)),
        false
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);

    let citycount_value = context::tests::TestFileSystem::make_file();
    let count_value = context::tests::TestFileSystem::make_file();
    count_value
        .borrow_mut()
        .write_all("254651\n".as_bytes())
        .unwrap();
    let topusers_value = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let stats_json = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.citycount", &citycount_value),
            ("workdir/stats/2020-05-10.count", &count_value),
            ("workdir/stats/2020-05-10.topusers", &topusers_value),
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);

    let citycount_value = context::tests::TestFileSystem::make_file();
    let zipcount_value = context::tests::TestFileSystem::make_file();
    let count_value = context::tests::TestFileSystem::make_file();
    let topusers_value = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let today_count = context::tests::TestFileSystem::make_file();
    today_count
        .borrow_mut()
        .write_all("254651\n".as_bytes())
        .unwrap();
    let stats_json = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.citycount", &citycount_value),
            ("workdir/stats/2020-05-10.zipcount", &zipcount_value),
            ("workdir/stats/2020-05-10.count", &count_value),
            ("workdir/stats/2020-05-10.topusers", &topusers_value),
            ("workdir/stats/2020-05-10.count", &today_count),
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
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-streets-gazdagret.csv",
        ),
        // For update_osm_housenumbers().
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
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
    let osm_streets_value = context::tests::TestFileSystem::make_file();
    let osm_housenumbers_value = context::tests::TestFileSystem::make_file();
    let ref_streets_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_value = context::tests::TestFileSystem::make_file();
    let missing_streets_value = context::tests::TestFileSystem::make_file();
    let missing_housenumbers_value = context::tests::TestFileSystem::make_file();
    let additional_streets_value = context::tests::TestFileSystem::make_file();
    let ref_streets_cache_value = context::tests::TestFileSystem::make_file();
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
            ("workdir/streets-gazdagret.csv", &osm_streets_value),
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &osm_housenumbers_value,
            ),
            (
                "workdir/streets-reference-gazdagret.lst",
                &ref_streets_value,
            ),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers_value,
            ),
            ("workdir/gazdagret-streets.percent", &missing_streets_value),
            ("workdir/gazdagret.percent", &missing_housenumbers_value),
            (
                "workdir/gazdagret-additional-streets.count",
                &additional_streets_value,
            ),
            (
                "workdir/refs/utcak_20190514.tsv.cache",
                &ref_streets_cache_value,
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
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

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
        let mut guard = osm_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_osm_housenumbers() is called.
    {
        let mut guard = osm_housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
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
    {
        let mut guard = missing_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_missing_housenumbers() is called.
    {
        let mut guard = missing_housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(network_arc);
    let mut file_system = context::tests::TestFileSystem::new();
    let stats_value = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let today_csv = context::tests::TestFileSystem::make_file();
    let today_count = context::tests::TestFileSystem::make_file();
    let today_citycount = context::tests::TestFileSystem::make_file();
    let today_zipcount = context::tests::TestFileSystem::make_file();
    let today_topusers = context::tests::TestFileSystem::make_file();
    let today_usercount = context::tests::TestFileSystem::make_file();
    let ref_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/stats.json", &stats_value),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
            ("workdir/stats/2020-05-10.csv", &today_csv),
            ("workdir/stats/2020-05-10.count", &today_count),
            ("workdir/stats/2020-05-10.citycount", &today_citycount),
            ("workdir/stats/2020-05-10.zipcount", &today_zipcount),
            ("workdir/stats/2020-05-10.topusers", &today_topusers),
            ("workdir/stats/2020-05-10.usercount", &today_usercount),
            ("workdir/stats/ref.count", &ref_count),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/stats/2020-05-10.csv");
    mtimes.insert(path, Rc::new(RefCell::new(ctx.get_time().now())));
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let unit_arc: Arc<dyn context::Unit> = Arc::new(unit);
    ctx.set_unit(&unit_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
            r#"addr:postcode	addr:city	addr:street	addr:housenumber	@user
7677	Orfű	Dollár utca	1	mgpx
"#
            .as_bytes(),
        )
        .unwrap();
    let today_count_value = context::tests::TestFileSystem::make_file();
    let today_citycount_value = context::tests::TestFileSystem::make_file();
    let today_zipcount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.csv", &today_csv_value),
            ("workdir/stats/2020-05-10.count", &today_count_value),
            ("workdir/stats/2020-05-10.citycount", &today_citycount_value),
            ("workdir/stats/2020-05-10.zipcount", &today_zipcount_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    {
        let mut guard = today_count_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    {
        let mut guard = today_citycount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    let mut guard = today_zipcount_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests update_stats_count(): the case then the .csv is missing.
#[test]
fn test_update_stats_count_no_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_count_value = context::tests::TestFileSystem::make_file();
    let today_citycount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.count", &today_count_value),
            ("workdir/stats/2020-05-10.citycount", &today_citycount_value),
        ],
    );
    file_system.set_files(&files);
    file_system.set_hide_paths(&[ctx.get_abspath("workdir/stats/2020-05-10.csv")]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    // No .csv, no .count or .citycount.
    {
        let mut guard = today_count_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
    {
        let mut guard = today_citycount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
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
    let today_count_value = context::tests::TestFileSystem::make_file();
    let today_citycount_value = context::tests::TestFileSystem::make_file();
    let today_zipcount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.csv", &today_csv_value),
            ("workdir/stats/2020-05-10.count", &today_count_value),
            ("workdir/stats/2020-05-10.citycount", &today_citycount_value),
            ("workdir/stats/2020-05-10.zipcount", &today_zipcount_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    let path = ctx.get_abspath("workdir/stats/2020-05-10.count");
    let actual = ctx.get_file_system().read_to_string(&path).unwrap();
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
            r#"addr:postcode	addr:city	addr:street	addr:housenumber	@user
1234	mycity	mystreet1	1	myuser1
1234	mycity	mystreet1	2	myuser1
1234	mycity	mystreet2	1	myuser2
"#
            .as_bytes(),
        )
        .unwrap();
    let today_topusers_value = context::tests::TestFileSystem::make_file();
    let today_usercount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.csv", &today_csv_value),
            ("workdir/stats/2020-05-10.topusers", &today_topusers_value),
            ("workdir/stats/2020-05-10.usercount", &today_usercount_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_topusers(&ctx, "2020-05-10").unwrap();

    {
        let abspath = ctx.get_abspath("workdir/stats/2020-05-10.topusers");
        let content = ctx.get_file_system().read_to_string(&abspath).unwrap();
        assert_eq!(content, "CNT\tUSER\n2\tmyuser1\n1\tmyuser2\n");
    }
    {
        let mut guard = today_usercount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
}

/// Tests update_stats_topusers(): the case then the .csv is missing.
#[test]
fn test_update_stats_topusers_no_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_topusers_value = context::tests::TestFileSystem::make_file();
    let today_usercount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.topusers", &today_topusers_value),
            ("workdir/stats/2020-05-10.usercount", &today_usercount_value),
        ],
    );
    file_system.set_files(&files);
    file_system.set_hide_paths(&[ctx.get_abspath("workdir/stats/2020-05-10.csv")]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_topusers(&ctx, "2020-05-10").unwrap();

    // No .csv, no .topusers or .usercount.
    {
        let mut guard = today_topusers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
    {
        let mut guard = today_usercount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
}

/// Tests write_city_count_path().
#[test]
fn test_write_city_count_path() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let file = context::tests::TestFileSystem::make_file();
    let relpath = "workdir/stats/2020-05-10.citycount";
    let abspath = ctx.get_abspath(relpath);
    let files = context::tests::TestFileSystem::make_files(&ctx, &[(relpath, &file)]);
    file_system.set_files(&files);
    let file_system: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system);
    let city1: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let city2: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let cities: HashMap<String, HashSet<String>> = [
        ("mycity2".to_string(), city2),
        ("mycity1".to_string(), city1),
    ]
    .into_iter()
    .collect();

    write_city_count_path(&ctx, &abspath, &cities).unwrap();

    let content = ctx.get_file_system().read_to_string(&abspath).unwrap();
    assert_eq!(content, "VAROS\tCNT\nmycity1\t2\nmycity2\t2\n");
}

/// Tests write_zip_count_path().
#[test]
fn test_write_zip_count_path() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let file = context::tests::TestFileSystem::make_file();
    let relpath = "workdir/stats/2020-05-10.zipcount";
    let abspath = ctx.get_abspath(relpath);
    let files = context::tests::TestFileSystem::make_files(&ctx, &[(relpath, &file)]);
    file_system.set_files(&files);
    let file_system: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system);
    let zip1: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let zip2: HashSet<String> = ["mystreet 1".to_string(), "mystreet 2".to_string()].into();
    let cities: HashMap<String, HashSet<String>> =
        [("myzip2".to_string(), zip2), ("myzip1".to_string(), zip1)]
            .into_iter()
            .collect();

    write_zip_count_path(&ctx, &abspath, &cities).unwrap();

    let content = ctx.get_file_system().read_to_string(&abspath).unwrap();
    assert_eq!(content, "IRSZ\tCNT\nmyzip1\t2\nmyzip2\t2\n");
}

/// Tests update_ref_housenumbers(): the case when we ask for CSV but get XML.
#[test]
fn test_update_ref_housenumbers_xml_as_csv() {
    // Given a junk osm_streets_value:
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let osm_streets_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_value = context::tests::TestFileSystem::make_file();
    osm_streets_value
        .borrow_mut()
        .write_all(b"@id\n42\n")
        .unwrap();
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
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-gazdagret.csv", &osm_streets_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers_value,
            ),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    // When updating ref housenumbers:
    update_ref_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    // Then make sure that the problematic relation is just skipped instead of failing:
    let mut guard = ref_housenumbers_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
}
