/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi module.

use super::*;
use std::cell::RefCell;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::DerefMut;
use std::rc::Rc;

/// Shared struct for wsgi tests.
pub struct TestWsgi {
    gzip_compress: bool,
    ctx: context::Context,
    headers: Vec<(String, String)>,
    bytes: Vec<u8>,
    absolute_path: bool,
    expected_status: u16,
    content_type: String,
}

impl TestWsgi {
    pub fn new() -> Self {
        let gzip_compress = false;
        let ctx = context::tests::make_test_context().unwrap();
        let headers: Vec<(String, String)> = Vec::new();
        let bytes: Vec<u8> = Vec::new();
        let absolute_path = false;
        let expected_status = 200_u16;
        let content_type = "text/html; charset=utf-8".into();
        TestWsgi {
            gzip_compress,
            ctx,
            headers,
            bytes,
            absolute_path,
            expected_status,
            content_type,
        }
    }

    pub fn get_ctx(&mut self) -> &mut context::Context {
        &mut self.ctx
    }

    /// Finds all matching subelements, by tag name or path.
    pub fn find_all(package: &sxd_document::Package, path: &str) -> Vec<String> {
        let document = package.as_document();
        let value = sxd_xpath::evaluate_xpath(&document, &format!("/html/{path}")).unwrap();
        let mut ret: Vec<String> = Vec::new();
        if let sxd_xpath::Value::Nodeset(nodeset) = value {
            ret = nodeset.iter().map(|i| i.string_value()).collect();
        };
        ret
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        self.content_type = content_type.to_string();
    }

    /// Generates an XML DOM for a given wsgi path.
    pub fn get_dom_for_path(&mut self, path: &str) -> sxd_document::Package {
        let prefix = self.ctx.get_ini().get_uri_prefix();
        let abspath: String;
        if self.absolute_path {
            abspath = path.into();
        } else {
            abspath = format!("{prefix}{path}");
        }
        if self.gzip_compress {
            self.headers
                .push(("Accept-Encoding".into(), "gzip, deflate".into()));
        }
        let request =
            rouille::Request::fake_http("GET", abspath, self.headers.clone(), self.bytes.clone());
        let response = application(&request, &self.ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();
        let mut headers_map = HashMap::new();
        for (key, value) in response.headers {
            headers_map.insert(key, value);
        }
        assert_eq!(headers_map["Content-type"], self.content_type);
        assert_eq!(data.is_empty(), false);
        let mut output: Vec<u8> = Vec::new();
        if self.gzip_compress {
            let mut gz = flate2::read::GzDecoder::new(data.as_slice());
            gz.read_to_end(&mut output).unwrap();
        } else {
            output = data;
        }
        let output_xml = String::from_utf8(output)
            .unwrap()
            .to_string()
            .replace("<!DOCTYPE html>", "");
        println!("get_dom_for_path: output_xml is '{output_xml}'");
        // Make sure the built-in error catcher is not kicking in.
        assert_eq!(response.status_code, self.expected_status);

        sxd_document::parser::parse(&output_xml).unwrap()
    }

    /// Generates a string for a given wsgi path.
    pub fn get_txt_for_path(&mut self, path: &str) -> String {
        let prefix = self.ctx.get_ini().get_uri_prefix();
        let abspath = format!("{prefix}{path}");
        let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
        let response = application(&request, &self.ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();
        let output = String::from_utf8(data).unwrap();
        println!("get_txt_for_path: output is '{output}'");
        // Make sure the built-in exception catcher is not kicking in.
        assert_eq!(response.status_code, 200);
        let mut headers_map = HashMap::new();
        for (key, value) in response.headers {
            headers_map.insert(key, value);
        }
        if path.ends_with(".chkl") {
            assert_eq!(headers_map["Content-type"], "application/octet-stream");
        } else {
            assert_eq!(headers_map["Content-type"], "text/plain; charset=utf-8");
        }
        assert_eq!(output.is_empty(), false);
        output
    }

    /// Generates an json value for a given wsgi path.
    pub fn get_json_for_path(&mut self, path: &str) -> serde_json::Value {
        let prefix = self.ctx.get_ini().get_uri_prefix();
        let abspath = format!("{prefix}{path}");
        let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
        let response = application(&request, &self.ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();
        assert_eq!(data.is_empty(), false);
        let output = String::from_utf8(data).unwrap();
        println!("get_json_for_path: output is '{output}'");
        // Make sure the built-in exception catcher is not kicking in.
        assert_eq!(response.status_code, 200);
        let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
        assert_eq!(
            headers_map["Content-type"],
            "application/json; charset=utf-8"
        );
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        value
    }

    /// Generates a CSS string for a given wsgi path.
    fn get_css_for_path(&mut self, path: &str) -> String {
        let prefix = self.ctx.get_ini().get_uri_prefix();
        let abspath = format!("{prefix}{path}");
        let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
        let response = application(&request, &self.ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();
        let css = String::from_utf8(data).unwrap();
        // println!("get_css_for_path: css is '{}'", css);
        // Make sure the built-in exception catcher is not kicking in.
        assert_eq!(response.status_code, 200);
        let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
        assert_eq!(headers_map["Content-type"], "text/css; charset=utf-8");
        assert_eq!(css.is_empty(), false);
        css
    }
}

/// Tests handle_streets(): if the output is well-formed.
#[test]
fn test_handle_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let root = test_wsgi.get_dom_for_path("/streets/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests handle_streets(): if the view-query output is well-formed.
#[test]
fn test_handle_streets_view_query_well_formed() {
    let mut test_wsgi = TestWsgi::new();
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/gazdagret/view-query");

    let results = TestWsgi::find_all(&root, "body/pre");
    assert_eq!(results.len(), 1);
}

/// Tests handle_streets(): if the update-result output is well-formed.
#[test]
fn test_handle_streets_update_result_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-streets-gazdagret.json",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.ctx.set_network(network_rc);
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/gazdagret/update-result");

    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    assert_eq!(
        relations
            .get_relation("gazdagret")
            .unwrap()
            .get_files()
            .get_osm_json_streets(&test_wsgi.ctx)
            .unwrap()
            .len(),
        4
    );
    let results = TestWsgi::find_all(&root, "body");
    assert_eq!(results.len(), 1);
}

/// Tests handle_streets(): if the update-result output on error is well-formed.
#[test]
fn test_handle_streets_update_result_error_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "", // no result -> error
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.ctx.set_network(network_rc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/gazdagret/update-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='overpass-error']");
    // Error during JSON query.
    assert_eq!(results.len(), 1);
}

/// Tests handle_streets(): if the update-result output is well-formed for
/// should_check_missing_streets() == "only".
#[test]
fn test_handle_streets_update_result_missing_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-streets-ujbuda.json",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.ctx.set_network(network_rc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "ujbuda": {
                "osmrelation": 42,
            },
        },
        "relation-ujbuda.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let template_value = context::tests::TestFileSystem::make_file();
    template_value
        .borrow_mut()
        .write_all(b"aaa @RELATION@ bbb @AREA@ ccc\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("data/streets-template.overpassql", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/ujbuda/update-result");

    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    assert_eq!(
        relations
            .get_relation("ujbuda")
            .unwrap()
            .get_files()
            .get_osm_json_streets(&test_wsgi.ctx)
            .unwrap()
            .len(),
        3
    );
    let results = TestWsgi::find_all(&root, "body");
    assert_eq!(results.len(), 1);
}

/// Tests the per-relation lints page.
#[test]
fn test_per_relation_lints() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Tűzkő utca": {
                    "ranges": [
                        {
                            "start": "9",
                            "end": "9",
                        }
                    ],
                },
                "Törökugrató utca": {
                    "invalid": [ "1", "11", "12", "42" ],
                }
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi
            .ctx
            .get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
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
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');"
        )
        .unwrap();
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
    {
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let mut relation = relations.get_relation("gazdagret").unwrap();
        relation.write_ref_housenumbers().unwrap();
        cache::get_missing_housenumbers_json(&mut relation).unwrap();
        relation.write_lints().unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-lints");

    // 2 are created-in-osm (1 range, 1 invalid)
    assert_eq!(
        TestWsgi::find_all(&root, "body/table/tr/td/div[@data-value='created-in-osm']").len(),
        2
    );
    // 42 is deleted-from-ref
    assert_eq!(
        TestWsgi::find_all(
            &root,
            "body/table/tr/td/div[@data-value='deleted-from-ref']"
        )
        .len(),
        1
    );
}

/// Tests the per-relation lints page, the out-of-range case.
#[test]
fn test_per_relation_lints_out_of_range() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh3073": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
        "relation-gh3073.yaml": {
            "filters": {
                "Hadak útja": {
                    "invalid": [ "3" ],
                    "ranges": [
                        {
                            "start": "5",
                            "end": "7",
                        }
                    ],
                },
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/cache-gh3073.json", &json_cache_value),
            (
                "workdir/street-housenumbers-reference-gh3073.lst",
                &ref_file,
            ),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/cache-gh3073.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    let now = test_wsgi.ctx.get_time().now();
    mtimes.insert(
        test_wsgi
            .ctx
            .get_abspath("workdir/street-housenumbers-reference-gh3073.lst"),
        Rc::new(RefCell::new(now)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        conn.execute_batch("insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Hadak útja', '3', '');").unwrap();
        conn.execute(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            ["gh3073", "15812165", "Hadak útja", "5", "1119", "", "", "", "", "", "", "", "Trendo 11 lakópark", "relation"],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gh3073", &mtime],
        )
        .unwrap();
    }
    {
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let mut relation = relations.get_relation("gh3073").unwrap();
        relation.write_ref_housenumbers().unwrap();
        cache::get_missing_housenumbers_json(&mut relation).unwrap();
        relation.write_lints().unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gh3073/view-lints");

    // 1 is out-of-range
    assert_eq!(
        TestWsgi::find_all(&root, "body/table/tr/td/div[@data-value='out-of-range']").len(),
        1
    );
}

/// Tests the missing house numbers page: if the output is well-formed.
#[test]
fn test_missing_housenumbers_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
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
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');"
        )
        .unwrap();
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
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    relation.write_ref_housenumbers().unwrap();

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");

    let mut results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);

    // refstreets: >0 invalid osm name
    results = TestWsgi::find_all(&root, "body/div[@id='osm-invalids-container']");
    assert_eq!(results.len(), 1);
    // refstreets: >0 invalid ref name
    results = TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: the output for a non-existing relation.
#[test]
fn test_missing_housenumbers_no_such_relation() {
    let mut test_wsgi = TestWsgi::new();
    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret42/view-result");
    let results = TestWsgi::find_all(&root, "body/div[@id='no-such-relation-error']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed (URL rewrite).
#[test]
fn test_missing_housenumbers_compat() {
    let mut test_wsgi = TestWsgi::new();
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
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');"
        )
        .unwrap();
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

        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/suspicious-streets/gazdagret/view-result");

    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    assert_eq!(relation.has_osm_housenumber_coverage().unwrap(), true);
    {
        let conn = test_wsgi.get_ctx().get_database_connection().unwrap();
        let json: String = conn
            .query_row(
                "select json from missing_housenumbers_cache where relation = ?1",
                ["gazdagret"],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!json.is_empty());
    }
    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed (URL rewrite for relation name).
#[test]
fn test_missing_housenumbers_compat_relation() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
        "relation-budafok.yaml": {
            "alias": [
                "budapest_22",
            ],
        }
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let jsoncache_value = context::tests::TestFileSystem::make_file();
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/cache-budafok.json", &jsoncache_value),
            (
                "workdir/street-housenumbers-reference-budafok.lst",
                &ref_file,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/cache-budafok.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '34', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '36', ' ');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '12', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '2', '');",
         )
         .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["budafok", "458338075", "Vöröskúti határsor", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/budafok", &mtime],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/budafok", &mtime],
        )
        .unwrap();
    }
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("budafok").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/suspicious-streets/budapest_22/view-result");

    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed, no osm streets case.
#[test]
fn test_missing_housenumbers_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed, no osm housenumbers case.
#[test]
fn test_missing_housenumbers_no_osm_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed, no ref housenumbers case.
#[test]
fn test_missing_housenumbers_no_ref_housenumbers_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_ref_housenumbers_path();
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
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/gazdagret", &mtime],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-ref-housenumbers']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: the txt output.
#[test]
fn test_missing_housenumbers_view_result_txt() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let json_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("workdir/cache-budafok.json", &json_cache)],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/cache-budafok.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["budafok", "458338075", "Vöröskúti határsor", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/budafok", &mtime],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/budafok", &mtime],
        )
        .unwrap();
    }

    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/budafok/view-result.txt");

    // Note how 12 is ordered after 2.
    assert_eq!(result, "Vöröskúti határsor\t[2, 12, 34, 36*]");
}

/// Tests the missing house numbers page: the txt output (even-odd streets).
#[test]
fn test_missing_housenumbers_view_result_txt_even_odd() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": ["11", "12"],
                },
                "Tűzkő utca": {
                    "interpolation": "all",
                },
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    file_system.set_files(&files);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');"
        )
        .unwrap();
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
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");

    let expected = r#"Hamzsabégi út	[1]
Törökugrató utca	[7], [10]
Tűzkő utca	[1, 2]"#;
    assert_eq!(result, expected);
}

/// Tests the missing house numbers page: the chkl output.
#[test]
fn test_missing_housenumbers_view_result_chkl() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
        "relation-budafok.yaml": {
            "filters": {
                "Vöröskúti határsor": {
                    "interpolation": "all",
                }
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-budafok.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '34', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '36', ' ');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '12', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Vöröskúti határsor', '2', '');",
         )
         .unwrap();
        conn.execute(
            r#"insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            ["budafok", "458338075", "Vöröskúti határsor", "", "", "", "", ""],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/budafok", &mtime],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/budafok", &mtime],
        )
        .unwrap();
    }
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("budafok").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/budafok/view-result.chkl");

    // Note how 12 is ordered after 2.
    assert_eq!(result, "[ ] Vöröskúti határsor [2, 12, 34, 36*]");
}

/// Tests the missing house numbers page: the chkl output (even-odd streets).
#[test]
fn test_missing_housenumbers_view_result_chkl_even_odd() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": ["11", "12"],
                },
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');"
        )
        .unwrap();
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
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");

    let expected = r#"[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [7], [10]
[ ] Tűzkő utca [1], [2]"#;
    assert_eq!(result, expected);
}

/// Tests the missing house numbers page: the chkl output (even-odd streets).
#[test]
fn test_missing_housenumbers_view_result_chkl_even_odd_split() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": ["11", "12"],
                },
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let hoursnumbers_ref = r#"Hamzsabégi út	1
Ref Name 1	1
Ref Name 1	2
Törökugrató utca	1	comment
Törökugrató utca	10
Törökugrató utca	11
Törökugrató utca	12
Törökugrató utca	2
Törökugrató utca	7
Tűzkő utca	1
Tűzkő utca	2
Tűzkő utca	9
Tűzkő utca	10
Tűzkő utca	12
Tűzkő utca	13
Tűzkő utca	14
Tűzkő utca	15
Tűzkő utca	16
Tűzkő utca	17
Tűzkő utca	18
Tűzkő utca	19
Tűzkő utca	20
Tűzkő utca	21
Tűzkő utca	22
Tűzkő utca	22
Tűzkő utca	24
Tűzkő utca	25
Tűzkő utca	26
Tűzkő utca	27
Tűzkő utca	28
Tűzkő utca	29
Tűzkő utca	30
Tűzkő utca	31
"#;
    let housenumbers_ref_value = context::tests::TestFileSystem::make_file();
    housenumbers_ref_value
        .borrow_mut()
        .write_all(hoursnumbers_ref.as_bytes())
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &housenumbers_ref_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");

    let expected = r#"[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [7], [10]
[ ] Tűzkő utca [1, 13, 15, 17, 19, 21, 25, 27, 29, 31]
[ ] Tűzkő utca [2, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]"#;
    assert_eq!(result, expected);
}

/// Tests the missing house numbers page: the chkl output, no osm streets case.
#[test]
fn test_missing_housenumbers_view_result_chkl_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
    assert_eq!(result, "No existing streets");
}

/// Tests the missing house numbers page: the chkl output, no osm housenumbers case.
#[test]
fn test_missing_housenumbers_view_result_chkl_no_osm_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/gazdagret", &mtime],
        )
        .unwrap();
    }
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
    assert_eq!(result, "No existing house numbers");
}

/// Tests the missing house numbers page: the chkl output, no ref housenumbers case.
#[test]
fn test_missing_housenumbers_view_result_chkl_no_ref_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_ref_housenumbers_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/gazdagret", &mtime],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
    assert_eq!(result, "No reference house numbers");
}

/// Tests the missing house numbers page: the txt output, no osm streets case.
#[test]
fn test_missing_housenumbers_view_result_txt_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
    assert_eq!(result, "No existing streets");
}

/// Tests the missing house numbers page: the txt output, no osm housenumbers case.
#[test]
fn test_missing_housenumbers_view_result_txt_no_osm_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/gazdagret", &mtime],
        )
        .unwrap();
    }
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
    assert_eq!(result, "No existing house numbers");
}

/// Tests the missing house numbers page: the txt output, no ref housenumbers case.
#[test]
fn test_missing_housenumbers_view_result_txt_no_ref_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_ref_housenumbers_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/gazdagret", &mtime],
        )
        .unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
    assert_eq!(result, "No reference house numbers");
}

/// Tests the missing house numbers page: if the view-turbo output is well-formed.
#[test]
fn test_missing_housenumbers_view_turbo_well_formed() {
    let mut test_wsgi = TestWsgi::new();
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
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
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
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');"
        )
        .unwrap();
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
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-turbo");

    let results = TestWsgi::find_all(&root, "body/pre");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the view-query output is well-formed.
#[test]
fn test_missing_housenumbers_view_query_well_formed() {
    let mut test_wsgi = TestWsgi::new();
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
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
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
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tűzkő utca', '9', '');"
        )
        .unwrap();
    }
    {
        let ctx = test_wsgi.get_ctx();
        let mut relations = areas::Relations::new(ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-query");

    let results = TestWsgi::find_all(&root, "body/pre");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the update-result output links back to the correct page.
#[test]
fn test_missing_housenumbers_update_result_link() {
    let mut test_wsgi = TestWsgi::new();
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &housenumbers_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let references = test_wsgi
        .ctx
        .get_ini()
        .get_reference_housenumber_paths()
        .unwrap();
    util::build_reference_index(&test_wsgi.ctx, &references).unwrap();
    let mtime = test_wsgi.get_ctx().get_time().now_string();
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
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/update-result");

    let mut guard = housenumbers_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let prefix = test_wsgi.ctx.get_ini().get_uri_prefix();
    let results = TestWsgi::find_all(
        &root,
        &format!("body/a[@href='{prefix}/missing-housenumbers/gazdagret/view-result']"),
    );
    assert_eq!(results.len(), 1);
}

/// Tests handle_street_housenumbers(): view result: the update-result link.
#[test]
fn test_housenumbers_view_result_update_result_link() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
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
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["housenumbers/gazdagret", &mtime],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-result");

    let uri = format!(
        "{}/missing-housenumbers/gazdagret/view-result",
        test_wsgi.ctx.get_ini().get_uri_prefix()
    );
    let results = TestWsgi::find_all(&root, &format!("body/div[@id='toolbar']/a[@href='{uri}']"));
    assert_eq!(results.len(), 1);
}

/// Tests handle_street_housenumbers(): if the view-query output is well-formed.
#[test]
fn test_housenumbers_view_query_well_formed() {
    let mut test_wsgi = TestWsgi::new();
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-query");

    let results = TestWsgi::find_all(&root, "body/pre");
    assert_eq!(results.len(), 1);
}

/// Tests handle_street_housenumbers(): if the update-result output is well-formed.
#[test]
fn test_housenumbers_update_result_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-housenumbers-gazdagret.json",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.ctx.set_network(network_rc);
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/update-result");

    let results = TestWsgi::find_all(&root, "body");
    assert_eq!(results.len(), 1);
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    assert_eq!(
        relations
            .get_relation("gazdagret")
            .unwrap()
            .get_files()
            .get_osm_json_streets(&test_wsgi.ctx)
            .unwrap()
            .len(),
        4
    );
}

/// Tests handle_street_housenumbers(): if the update-result output on error is well-formed.
#[test]
fn test_housenumbers_update_result_error_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    test_wsgi.ctx.set_network(network_rc);
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
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "data/street-housenumbers-template.overpassql",
                &overpass_template,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/update-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='overpass-error']");
    assert_eq!(results.len(), 1);
}

/// Tests handle_street_housenumbers(): if the output is well-formed, no osm streets case.
#[test]
fn test_housenumbers_no_osm_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-result");
    let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: if the output is well-formed.
#[test]
fn test_missing_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "01",
                "refsettlement": "011",
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
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");

    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    assert_eq!(relation.has_osm_street_coverage().unwrap(), true);
    let mut results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
    // refstreets: >0 invalid osm name
    results = TestWsgi::find_all(&root, "body/div[@id='osm-invalids-container']");
    assert_eq!(results.len(), 1);
    // refstreets: >0 invalid ref name
    results = TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: if the output is well-formed (URL rewrite).
#[test]
fn test_missing_streets_well_formed_compat() {
    let mut test_wsgi = TestWsgi::new();
    {
        let ref_streets = test_wsgi.ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&test_wsgi.ctx, &ref_streets).unwrap();
    }
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
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let root = test_wsgi.get_dom_for_path("/suspicious-relations/gazdagret/view-result");

    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    assert_eq!(relation.has_osm_street_coverage().unwrap(), true);
    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: if the output is well-formed, no osm streets case.
#[test]
fn test_missing_streets_no_osm_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: the txt output.
#[test]
fn test_missing_streets_view_result_txt() {
    let mut test_wsgi = TestWsgi::new();
    {
        let ref_streets = test_wsgi.ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&test_wsgi.ctx, &ref_streets).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "01",
                "refsettlement": "011",
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
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

    assert_eq!(result, "Only In Ref Nonsense utca\nOnly In Ref utca\n");
}

/// Tests the missing streets page: the chkl output.
#[test]
fn test_missing_streets_view_result_chkl() {
    let mut test_wsgi = TestWsgi::new();
    {
        let ref_streets = test_wsgi.ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&test_wsgi.ctx, &ref_streets).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "01",
                "refsettlement": "011",
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "street-filters": ["Only In Ref Nonsense utca"],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.chkl");

    assert_eq!(result, "[ ] Only In Ref utca\n");
}

/// Tests the missing streets page: the txt output, no osm streets case.
#[test]
fn test_missing_streets_view_result_txt_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

    assert_eq!(result, "No existing streets");
}

/// Tests the missing streets page: if the view-query output is well-formed.
#[test]
fn test_missing_streets_view_query_well_formed() {
    let mut test_wsgi = TestWsgi::new();
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
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-query");

    let results = TestWsgi::find_all(&root, "body/pre");

    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: the view-turbo output.
#[test]
fn test_missing_streets_view_turbo() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "OSM Name 2": {
                    "show-refstreet": false,
                },
            },
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
                "OSM Name 2": "Ref Name 2",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-turbo");

    let results = TestWsgi::find_all(&root, "body/pre");

    assert_eq!(results.len(), 1);

    assert_eq!(results[0].contains("OSM Name 1"), true);
    // This is silenced with `show-refstreet: false`.
    assert_eq!(results[0].contains("OSM Name 2"), false);
}

/// Tests handle_main(): if the output is well-formed.
#[test]
fn test_main_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let root = test_wsgi.get_dom_for_path("/");
    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests handle_main(): the case when the URL is empty (should give the main page).
#[test]
fn test_main_no_path() {
    let request = rouille::Request::fake_http("GET", "", vec![], vec![]);
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let ret = webframe::get_request_uri(&request, &ctx, &mut relations).unwrap();
    assert_eq!(ret, "");
}

/// Tests handle_main(): if the /osm/filter-for/incomplete output is well-formed.
#[test]
fn test_main_filter_for_incomplete() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/filter-for/incomplete");

    let results = TestWsgi::find_all(&root, "body/table/tr");
    // header + 1 relation
    assert_eq!(results.len(), 2);
}

/// Tests handle_main(): if the /osm/filter-for/everything output is well-formed.
#[test]
fn test_main_filter_for_everything_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let root = test_wsgi.get_dom_for_path("/filter-for/everything");
    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests handle_main(): the /osm/filter-for/refcounty output.
#[test]
fn test_main_filter_for_refcounty() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "refcounty-names.yaml": {
            "01": "Budapest",
            "67": "Sixtyseven",
        },
        "refsettlement-names.yaml": {
            "01": {
                "011": "Ujbuda",
                "012": "Hegyvidek",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/filter-for/refcounty/01/whole-county");

    let results = TestWsgi::find_all(&root, "body/table/tr");
    // header + 1 relation
    assert_eq!(results.len(), 2);
}

/// Tests handle_main(): if the /osm/filter-for/refcounty output is well-formed.
#[test]
fn test_main_filter_for_refcounty_no_refsettlement() {
    let mut test_wsgi = TestWsgi::new();
    let root = test_wsgi.get_dom_for_path("/filter-for/refcounty/67/whole-county");
    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests handle_main(): the /osm/filter-for/refcounty/<value>/refsettlement/<value> output.
#[test]
fn test_main_filter_for_refcounty_refsettlement() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation1": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "myrelation2": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "012",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/filter-for/refcounty/01/refsettlement/011");

    let mut results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
    results = TestWsgi::find_all(&root, "body/table/tr");
    // header + myrelation1 (but not myrelation2) was just 1 when the filter was buggy
    assert_eq!(results.len(), 2);
}

/// Tests handle_main(): the /osm/filter-for/relations/... output
#[test]
fn test_main_filter_for_relations() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation1": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "myrelation2": {
                "osmrelation": 43,
                "refcounty": "01",
                "refsettlement": "012",
            },
            "myrelation3": {
                "osmrelation": 44,
                "refcounty": "01",
                "refsettlement": "013",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/filter-for/relations/42,43");

    let mut results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
    results = TestWsgi::find_all(&root, "body/table/tr");
    // header + the two requested relations
    assert_eq!(results.len(), 3);
}

/// Tests handle_main(): the /osm/filter-for/relations/ output.
#[test]
fn test_main_filter_for_relations_empty() {
    let mut test_wsgi = TestWsgi::new();

    let root = test_wsgi.get_dom_for_path("/filter-for/relations/");

    let mut results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
    results = TestWsgi::find_all(&root, "body/table/tr");
    // header + no requested relations
    assert_eq!(results.len(), 1);
}

/// Tests application(): the error handling case.
#[test]
fn test_application_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_rc: Rc<dyn context::Unit> = Rc::new(unit);
    ctx.set_unit(&unit_rc);
    let css = context::tests::TestFileSystem::make_file();
    {
        let mut guard = css.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"/* comment */").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[("target/browser/osm.min.css", &css)]);
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let bytes: Vec<u8> = Vec::new();

    let abspath: String = "/".into();
    let rouille_headers: Vec<(String, String)> = Vec::new();
    let request = rouille::Request::fake_http("GET", abspath, rouille_headers, bytes);
    let response = application(&request, &ctx);
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

/// Tests /osm/webhooks/: /osm/webhooks/github.
#[test]
fn test_webhooks_github() {
    let root = serde_json::json!({"ref": "refs/heads/master"});
    let payload = serde_json::to_string(&root).unwrap();
    let query_string: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("payload", &payload)
        .finish();
    let mut test_wsgi = TestWsgi::new();
    test_wsgi.bytes = query_string.as_bytes().to_vec();
    let expected_args = format!("make -C {} deploy", test_wsgi.ctx.get_abspath(""));
    let outputs: HashMap<_, _> = vec![(expected_args, "".to_string())].into_iter().collect();
    let subprocess = context::tests::TestSubprocess::new(&outputs);
    let subprocess_rc: Rc<dyn context::Subprocess> = Rc::new(subprocess);
    test_wsgi.ctx.set_subprocess(&subprocess_rc);

    test_wsgi.get_dom_for_path("/webhooks/github");

    let subprocess = subprocess_rc
        .as_any()
        .downcast_ref::<context::tests::TestSubprocess>()
        .unwrap();
    assert_eq!(subprocess.get_runs().is_empty(), false);
    assert_eq!(subprocess.get_exits(), &[1]);
}

/// Tests /osm/webhooks/: /osm/webhooks/github, the case when a non-master branch is updated.
#[test]
fn test_webhooks_github_branch() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let outputs: HashMap<String, String> = HashMap::new();
    let subprocess = context::tests::TestSubprocess::new(&outputs);
    let subprocess_rc: Rc<dyn context::Subprocess> = Rc::new(subprocess);
    ctx.set_subprocess(&subprocess_rc);
    let root = serde_json::json!({"ref": "refs/heads/stable"});
    let payload = serde_json::to_string(&root).unwrap();
    let query_string: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("payload", &payload)
        .finish();
    let buf = query_string.as_bytes().to_vec();
    let request = rouille::Request::fake_http("GET", "/", vec![], buf);

    webframe::handle_github_webhook(&request, &ctx).unwrap();

    let subprocess = subprocess_rc
        .as_any()
        .downcast_ref::<context::tests::TestSubprocess>()
        .unwrap();
    assert_eq!(subprocess.get_runs().is_empty(), true);
}

/// Tests handle_stats().
#[test]
fn test_handle_stats() {
    let mut test_wsgi = TestWsgi::new();

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/whole-country/");

    let results = TestWsgi::find_all(&root, "body/h2");
    // 10 chart types + note
    assert_eq!(results.len(), 11);
}

/// Tests /osm/static/: the css case.
#[test]
fn test_static_css() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let css_value = context::tests::TestFileSystem::make_file();
    css_value.borrow_mut().write_all(b"{}").unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("target/browser/osm.min.css", &css_value)],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("target/browser/osm.min.css"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);

    let result = test_wsgi.get_css_for_path("/static/osm.min.css");

    assert_eq!(result.ends_with('}'), true);
}

/// Tests /osm/static/: the plain text case.
#[test]
fn test_static_text() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let txt_value = context::tests::TestFileSystem::make_file();
    txt_value
        .borrow_mut()
        .write_all(b"User-agent: *\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/robots.txt", &txt_value)],
    );
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_rc);

    let result = test_wsgi.get_txt_for_path("/robots.txt");

    assert_eq!(result, "User-agent: *\n");
}

/// Tests handle_stats_cityprogress(): if the output is well-formed.
#[test]
fn test_handle_stats_cityprogress_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_citycounts (date, city, count) values (?1, ?2, ?3)"#,
            ["2020-05-10", "budapest_11", "11"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_citycounts (date, city, count) values (?1, ?2, ?3)"#,
            ["2020-05-10", "budapest_12", "12"],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/whole-country/cityprogress");

    let results = TestWsgi::find_all(&root, "body/table/tr");
    // header; also budapest_11/budapest_12 are both in ref and osm
    assert_eq!(results.len(), 3);
}

/// Tests handle_stats_zipprogress(): if the output is well-formed.
#[test]
fn test_handle_stats_zipprogress_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_zipcounts (date, zip, count) values (?1, ?2, ?3)"#,
            ["2020-05-10", "1111", "10"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_zipcounts (date, zip, count) values (?1, ?2, ?3)"#,
            ["2020-05-10", "1121", "20"],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/whole-country/zipprogress");

    let results = TestWsgi::find_all(&root, "body/table/tr");
    // header; also 1111/1121 is both in ref and osm
    assert_eq!(results.len(), 3);
}

/// Tests handle_invalid_refstreets().
#[test]
fn test_handle_invalid_refstreets() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
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

    let mut results = TestWsgi::find_all(&root, "body/h1/a");
    assert_eq!(results.is_empty(), false);

    // 2 matches: 'OSM Name 1' is in the OSM street list + it's not in the reference.
    results = TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']/ul/li");
    assert_eq!(results.len(), 2);
}

/// Tests handle_invalid_refstreets(): error handling when osm street list is missing for a relation.
#[test]
fn test_handle_invalid_refstreets_no_osm_sreets() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/lints/whole-country/invalid-relations");

    let results = TestWsgi::find_all(&root, "body");
    assert_eq!(results.is_empty(), false);
}

/// Tests handle_invalid_refstreets(): ignore relations which have empty invalid lists.
#[test]
fn test_handle_invalid_refstreets_no_invalids() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
    let mtime = test_wsgi.get_ctx().get_time().now_string();
    {
        let conn = test_wsgi.ctx.get_database_connection().unwrap();
        conn.execute(
            "insert into mtimes (page, last_modified) values (?1, ?2)",
            ["streets/myrelation", &mtime],
        )
        .unwrap();
    }

    let root = test_wsgi.get_dom_for_path("/lints/whole-country/invalid-relations");

    let results = TestWsgi::find_all(&root, "body/h1/a");
    assert_eq!(results.is_empty(), true);
}

/// Tests the not-found page: if the output is well-formed.
#[test]
fn test_not_found_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    test_wsgi.absolute_path = true;
    test_wsgi.expected_status = 404;

    let root = test_wsgi.get_dom_for_path("/asdf");

    let results = TestWsgi::find_all(&root, "body/h1");

    assert_eq!(results.is_empty(), false);
}

/// Tests gzip compress case.
#[test]
fn test_compress() {
    let mut test_wsgi = TestWsgi::new();
    test_wsgi.gzip_compress = true;

    let root = test_wsgi.get_dom_for_path("/");

    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Test get_housenr_additional_count().
#[test]
fn test_get_housenr_additional_count() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
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
    let relation = relations.get_relation("myrelation").unwrap();

    let ret = get_housenr_additional_count(&ctx, &relation).unwrap();

    // Not a failure, just no pre-calculated result.
    assert_eq!(ret, "");
}

/// Tests handle_main_housenr_additional_count().
#[test]
fn test_handle_main_housenr_additional_count() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
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
    let relation = relations.get_relation("myrelation").unwrap();

    let ret = handle_main_housenr_additional_count(&ctx, &relation).unwrap();

    // No pre-calculated data, so no numbers.
    assert_eq!(
        ret.get_value(),
        r#"<strong><a href="/osm/additional-housenumbers/myrelation/view-result">additional house numbers</a></strong>"#
    );
}

/// Tests handle_main_relation() for the missing-streets=no case.
#[test]
fn test_handle_main_relation() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "missing-streets": "no",
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
    let filter_for = Box::new(filter_for_everything);

    let ret = handle_main_relation(&ctx, &mut relations, &filter_for, "myrelation").unwrap();

    // area, missing housenumbers, additional housenumbers, missing streets, additional streets,
    // relation link
    assert_eq!(ret.len(), 6);
    // missing-streets=no, so 'missing streets' should be empty.
    assert_eq!(ret[3].get_value(), "");
    // same for additional streets.
    assert_eq!(ret[4].get_value(), "");
}

/// Tests handle_main_relation() for the missing-streets=only case.
#[test]
fn test_handle_main_relation_streets_only() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "missing-streets": "only",
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
    let filter_for = Box::new(filter_for_everything);

    let ret = handle_main_relation(&ctx, &mut relations, &filter_for, "myrelation").unwrap();

    // area, missing housenumbers, additional housenumbers, missing streets, additional streets,
    // relation link
    assert_eq!(ret.len(), 6);
    // missing-streets=only, so 'missing housenumbers' should be empty.
    assert_eq!(ret[1].get_value(), "");
    // same for additional housenumbers.
    assert_eq!(ret[2].get_value(), "");
}

/// Tests handle_main_housenr_percent().
#[test]
fn test_handle_main_housenr_percent() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_housenumber_coverages (relation_name, coverage, last_modified) values (?1, ?2, ?3)"#,
            ["gazdagret", "4.2", "0"],
        ).unwrap();
    }
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let (doc, percent) = handle_main_housenr_percent(&ctx, &relation).unwrap();

    assert_eq!(doc.get_value().is_empty(), false);
    assert_eq!(percent, 4.2_f64);
}

/// Tests handle_main_street_percent().
#[test]
fn test_handle_main_street_percent() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into osm_street_coverages (relation_name, coverage, last_modified) values (?1, ?2, ?3)"#,
            ["gazdagret", "80.00", "0"],
        ).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let (doc, percent) = handle_main_street_percent(&ctx, &relation).unwrap();

    assert_eq!(doc.get_value().is_empty(), false);
    assert_eq!(percent, 80.0_f64);
}

/// Tests handle_main_street_additional_count().
#[test]
fn test_handle_main_street_additional_count() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into additional_streets_counts (relation, count) values ('gazdagret', '42');
            insert into osm_street_coverages (relation_name, coverage, last_modified) values ('gazdagret', '0', '0');",
        )
        .unwrap();
    }
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let doc = handle_main_street_additional_count(&ctx, &relation).unwrap();

    assert_eq!(doc.get_value().contains("42 streets"), true);
}

/// Tests handle_main_filters_refcounty().
#[test]
fn test_handle_main_filters_refcounty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "refcounty-names.yaml": {
            "01": "Budapest",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = areas::Relations::new(&ctx).unwrap();

    let ret = handle_main_filters_refcounty(&ctx, &relations, "", "01").unwrap();

    assert_eq!(
        ret.get_value(),
        "<a href=\"/osm/filter-for/refcounty/01/whole-county\">Budapest</a>"
    );
}

/// Tests handle_main_filters_refcounty(), the case when refcounty_id is non-empty.
#[test]
fn test_handle_main_filters_refcounty_filter() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "refcounty-names.yaml": {
            "01": "Budapest",
        },
        "refsettlement-names.yaml": {
            "01": {
                "011": "Ujbuda",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = areas::Relations::new(&ctx).unwrap();

    let ret = handle_main_filters_refcounty(&ctx, &relations, "01", "01").unwrap();

    assert_eq!(ret.get_value(), "<a href=\"/osm/filter-for/refcounty/01/whole-county\">Budapest</a> (<a href=\"/osm/filter-for/refcounty/01/refsettlement/011\">Ujbuda</a>)");
}

/// Tests handle_main_filters_refcounty(), the case when refcounty_id is non-empty, but the county
/// has no settlements.
#[test]
fn test_handle_main_filters_refcounty_filter_no_settlements() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "refcounty-names.yaml": {
            "01": "Budapest",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = areas::Relations::new(&ctx).unwrap();

    let ret = handle_main_filters_refcounty(&ctx, &relations, "01", "01").unwrap();

    assert_eq!(
        ret.get_value(),
        "<a href=\"/osm/filter-for/refcounty/01/whole-county\">Budapest</a>"
    );
}
