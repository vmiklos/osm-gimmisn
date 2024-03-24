/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the webframe module.

use super::*;
use crate::context::Unit;
use crate::wsgi;
use std::io::Write;

/// Tests handle_static().
#[test]
fn test_handle_static() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let css = context::tests::TestFileSystem::make_file();
    {
        let mut guard = css.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"/* comment */").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[("target/browser/osm.min.css", &css)]);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("target/browser/osm.min.css");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/osm.min.css")).unwrap();

    assert_eq!(content.is_empty(), false);
    assert_eq!(content_type, "text/css; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the generated javascript case.
#[test]
fn test_handle_static_generated_javascript() {
    let ctx = context::tests::make_test_context().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/bundle.js")).unwrap();
    assert_eq!("// bundle.js\n".as_bytes(), content);
    assert_eq!(content_type, "application/x-javascript; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the json case.
#[test]
fn test_handle_static_json() {
    let ctx = context::tests::make_test_context().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/stats-empty.json")).unwrap();
    assert_eq!(content.starts_with(b"{"), true);
    assert_eq!(content_type, "application/json; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the ico case.
#[test]
fn test_handle_static_ico() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let ico = context::tests::TestFileSystem::make_file();
    {
        let mut guard = ico.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"\0").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[("favicon.ico", &ico)]);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("favicon.ico");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let (content, content_type, extra_headers) = handle_static(&ctx, "/favicon.ico").unwrap();

    assert_eq!(content.is_empty(), false);
    assert_eq!(content_type, "image/x-icon");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the svg case.
#[test]
fn test_handle_static_svg() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let svg = context::tests::TestFileSystem::make_file();
    {
        let mut guard = svg.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"<svg").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[("favicon.svg", &svg)]);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("favicon.svg");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let (content, content_type, extra_headers) = handle_static(&ctx, "/favicon.svg").unwrap();

    assert_eq!(content.is_empty(), false);
    assert_eq!(content_type, "image/svg+xml; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests the case when the content type is not recognized.
#[test]
fn test_handle_static_else() {
    let ctx = context::tests::make_test_context().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/test.xyz")).unwrap();
    assert_eq!(content.is_empty(), true);
    assert_eq!(content_type.is_empty(), true);
    // No last modified non-existing file.
    assert_eq!(extra_headers.is_empty(), true);
}

/// Tests fill_missing_header_items().
#[test]
fn test_fill_missing_header_items() {
    let streets = "no";
    let relation_name = "gazdagret";
    let mut items: Vec<yattag::Doc> = Vec::new();
    let additional_housenumbers = true;
    let ctx = context::tests::make_test_context().unwrap();
    items = fill_missing_header_items(
        &ctx,
        streets,
        additional_housenumbers,
        relation_name,
        &items,
    )
    .unwrap();
    let html = items[0].get_value();
    assert_eq!(html.contains("Missing house numbers"), true);
    assert_eq!(html.contains("Missing streets"), false);
}

/// Tests handle_error().
#[test]
fn test_handle_error() {
    let headers = vec![("Accept-Language".to_string(), ",".to_string())];
    let request = rouille::Request::fake_http("GET", "/", headers, vec![]);

    let unit = context::tests::TestUnit::new();
    let err = unit.make_error();

    let response = handle_error(&request, &format!("{err:?}"));
    let mut data = Vec::new();
    let (mut reader, _size) = response.data.into_reader_and_size();
    reader.read_to_end(&mut data).unwrap();

    assert_eq!(response.status_code, 500);

    let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
    assert_eq!(headers_map["Content-type"], "text/html; charset=utf-8");
    assert_eq!(data.is_empty(), false);

    let output = String::from_utf8(data).unwrap();
    assert_eq!(output.contains("TestError"), true);
}

/// Tests get_toolbar().
#[test]
fn test_get_toolbar() {
    let ctx = context::tests::make_test_context().unwrap();

    let ret = get_toolbar(&ctx, None, "myfunc", "myrel", 42).unwrap();

    assert_eq!(ret.get_value().is_empty(), false);
}

/// Tests handle_invalid_addr_cities().
#[test]
fn test_handle_invalid_addr_cities() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute("insert into stats_invalid_addr_cities (osm_id, osm_type, postcode, city, street, housenumber, user) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                   ["42", "type", "1111", "mycity", "mystreet", "myhousenumber", "myuser"]).unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["whole-country/osm-base", "0"],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["whole-country/areas-base", "0"],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/lints/whole-country/invalid-addr-cities");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/table/tr");
    // header + 1 row.
    assert_eq!(results.len(), 2);
}

/// Tests handle_invalid_refstreets(), the case when a relation has no errors.
#[test]
fn test_handle_invalid_refstreets_no_errors() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
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
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
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

    let root = test_wsgi.get_dom_for_path("/lints/whole-country/invalid-relations");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/h1/a");
    assert_eq!(results.is_empty(), true);
}

/// Tests handle_lints().
#[test]
fn test_handle_lints() {
    let mut test_wsgi = wsgi::tests::TestWsgi::new();

    let root = test_wsgi.get_dom_for_path("/lints/whole-country/");

    let results = wsgi::tests::TestWsgi::find_all(&root, "body/ul/li");
    // 2 lint types.
    assert_eq!(results.len(), 2);
}

/// Tests handle_invalid_addr_cities_update().
#[test]
fn test_handle_invalid_addr_cities_update() {
    // Given a context to get /invalid-addr-cities/update-result:
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
            /*result_path=*/ "src/fixtures/network/overpass-stats.csv",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-stats.json",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
    let csv_value = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.get_ctx(),
        &[
            ("workdir/stats/whole-country.csv", &csv_value),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

    // When getting that page:
    let root = test_wsgi.get_dom_for_path("/lints/whole-country/invalid-addr-cities/update-result");

    // Then make sure the whole-country.csv is updated:
    let path = test_wsgi
        .get_ctx()
        .get_abspath(&format!("workdir/stats/whole-country.csv"));
    let actual = test_wsgi
        .get_ctx()
        .get_file_system()
        .read_to_string(&path)
        .unwrap();
    assert_eq!(
        actual,
        String::from_utf8(std::fs::read("src/fixtures/network/overpass-stats.csv").unwrap())
            .unwrap()
    );
    // SQL is updated:
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        let last_modified: String = conn
            .query_row(
                "select last_modified from mtimes where page = ?1",
                ["whole-country/osm-base"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!last_modified.is_empty());
    }
    // Output is well-formed:
    let results = wsgi::tests::TestWsgi::find_all(&root, "body");
    assert_eq!(results.len(), 1);
}

/// Tests handle_invalid_addr_cities_update_json().
#[test]
fn test_handle_invalid_addr_cities_update_json() {
    // Given a context to get /invalid-addr-cities/update-result.json:
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
            /*result_path=*/ "src/fixtures/network/overpass-stats.csv",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "src/fixtures/network/overpass-stats.json",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.get_ctx().set_network(network_rc);
    let csv_value = context::tests::TestFileSystem::make_file();
    let overpass_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.get_ctx(),
        &[
            ("workdir/stats/whole-country.csv", &csv_value),
            (
                "data/street-housenumbers-hungary.overpassql",
                &overpass_template,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.get_ctx().set_file_system(&file_system_rc);

    // When getting that page:
    let root =
        test_wsgi.get_json_for_path("/lints/whole-country/invalid-addr-cities/update-result.json");
    assert_eq!(root.as_object().unwrap()["error"], "");

    // Then make sure the whole-country.csv is updated:
    let path = test_wsgi
        .get_ctx()
        .get_abspath(&format!("workdir/stats/whole-country.csv"));
    let actual = test_wsgi
        .get_ctx()
        .get_file_system()
        .read_to_string(&path)
        .unwrap();
    assert_eq!(
        actual,
        String::from_utf8(std::fs::read("src/fixtures/network/overpass-stats.csv").unwrap())
            .unwrap()
    );
    // SQL is updated:
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        let last_modified: String = conn
            .query_row(
                "select last_modified from mtimes where page = ?1",
                ["whole-country/osm-base"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!last_modified.is_empty());
    }
}
