/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the wsgi module.

use super::*;
use context::FileSystem;
use std::cell::RefCell;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::rc::Rc;

/// Shared struct for wsgi tests.
pub struct TestWsgi {
    gzip_compress: bool,
    ctx: context::Context,
    headers: Vec<(String, String)>,
    bytes: Vec<u8>,
    absolute_path: bool,
    expected_status: u16,
}

impl TestWsgi {
    pub fn new() -> Self {
        let gzip_compress = false;
        let ctx = context::tests::make_test_context().unwrap();
        let headers: Vec<(String, String)> = Vec::new();
        let bytes: Vec<u8> = Vec::new();
        let absolute_path = false;
        let expected_status = 200_u16;
        TestWsgi {
            gzip_compress,
            ctx,
            headers,
            bytes,
            absolute_path,
            expected_status,
        }
    }

    pub fn get_ctx(&mut self) -> &mut context::Context {
        &mut self.ctx
    }

    /// Finds all matching subelements, by tag name or path.
    pub fn find_all(package: &sxd_document::Package, path: &str) -> Vec<String> {
        let document = package.as_document();
        let value = sxd_xpath::evaluate_xpath(&document, &format!("/html/{}", path)).unwrap();
        let mut ret: Vec<String> = Vec::new();
        if let sxd_xpath::Value::Nodeset(nodeset) = value {
            ret = nodeset.iter().map(|i| i.string_value()).collect();
        };
        ret
    }

    /// Generates an XML DOM for a given wsgi path.
    pub fn get_dom_for_path(&mut self, path: &str) -> sxd_document::Package {
        let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
        let abspath: String;
        if self.absolute_path {
            abspath = path.into();
        } else {
            abspath = format!("{}{}", prefix, path);
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
        assert_eq!(headers_map["Content-type"], "text/html; charset=utf-8");
        assert_eq!(data.is_empty(), false);
        let mut output: Vec<u8> = Vec::new();
        if self.gzip_compress {
            let mut gz = flate2::read::GzDecoder::new(data.as_slice());
            gz.read_to_end(&mut output).unwrap();
        } else {
            output = data;
        }
        let output_xml =
            format!("{}", String::from_utf8(output).unwrap()).replace("<!DOCTYPE html>", "");
        println!("get_dom_for_path: output_xml is '{}'", output_xml);
        // Make sure the built-in error catcher is not kicking in.
        assert_eq!(response.status_code, self.expected_status);
        let package = sxd_document::parser::parse(&output_xml).unwrap();
        package
    }

    /// Generates a string for a given wsgi path.
    pub fn get_txt_for_path(&mut self, path: &str) -> String {
        let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
        let abspath = format!("{}{}", prefix, path);
        let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
        let response = application(&request, &self.ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();
        let output = String::from_utf8(data).unwrap();
        println!("get_txt_for_path: output is '{}'", output);
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
        let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
        let abspath = format!("{}{}", prefix, path);
        let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
        let response = application(&request, &self.ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();
        // Make sure the built-in exception catcher is not kicking in.
        assert_eq!(response.status_code, 200);
        let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
        assert_eq!(
            headers_map["Content-type"],
            "application/json; charset=utf-8"
        );
        assert_eq!(data.is_empty(), false);
        let value: serde_json::Value =
            serde_json::from_str(&String::from_utf8(data).unwrap()).unwrap();
        value
    }

    /// Generates a CSS string for a given wsgi path.
    fn get_css_for_path(&mut self, path: &str) -> String {
        let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
        let abspath = format!("{}{}", prefix, path);
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
            ("data/streets-template.txt", &template_value),
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
        /*result_path=*/ "tests/network/overpass-streets-gazdagret.csv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.ctx.set_network(&network_arc);
    let streets_value = context::tests::TestFileSystem::make_file();
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
            ("workdir/streets-gazdagret.csv", &streets_value),
            ("data/streets-template.txt", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/gazdagret/update-result");

    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
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
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.ctx.set_network(&network_arc);
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
            ("data/streets-template.txt", &template_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/gazdagret/update-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='overpass-error']");
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
        /*result_path=*/ "tests/network/overpass-streets-ujbuda.csv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.ctx.set_network(&network_arc);
    let streets_value = context::tests::TestFileSystem::make_file();
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
            ("data/streets-template.txt", &template_value),
            ("workdir/streets-ujbuda.csv", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/streets/ujbuda/update-result");

    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let results = TestWsgi::find_all(&root, "body");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed.
#[test]
fn test_missing_housenumbers_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
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
    let percent_value = context::tests::TestFileSystem::make_file();
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.percent", &percent_value),
            ("workdir/gazdagret.cache.json", &json_cache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/gazdagret.cache.json"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

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
    let streets_value = context::tests::TestFileSystem::make_file();
    let jsoncache_value = context::tests::TestFileSystem::make_file();
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
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.percent", &streets_value),
            ("workdir/gazdagret.cache.json", &jsoncache_value),
        ],
    );
    file_system.set_files(&files);
    // Make sure the cache is outdated.
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/gazdagret.cache.json"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/suspicious-streets/gazdagret/view-result");

    {
        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    {
        let mut guard = jsoncache_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
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
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/budafok.cache.json", &jsoncache_value),
            ("workdir/budafok.percent", &percent_value),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/budafok.cache.json"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/suspicious-streets/budapest_22/view-result");

    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed, no osm streets case.
#[test]
fn test_missing_housenumbers_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_streets_path();
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: if the output is well-formed, no osm housenumbers case.
#[test]
fn test_missing_housenumbers_no_osm_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_housenumbers_path();
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-ref-housenumbers']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing house numbers page: the txt output.
#[test]
fn test_missing_housenumbers_view_result_txt() {
    let mut test_wsgi = TestWsgi::new();
    let mut file_system = context::tests::TestFileSystem::new();
    let txt_cache = context::tests::TestFileSystem::make_file();
    let json_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("workdir/budafok.txtcache", &txt_cache),
            ("workdir/budafok.cache.json", &json_cache),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/budafok.txtcache"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/budafok.cache.json"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

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
    let txt_cache_value = context::tests::TestFileSystem::make_file();
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.txtcache", &txt_cache_value),
            ("workdir/gazdagret.cache.json", &json_cache_value),
        ],
    );
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/gazdagret.txtcache"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        test_wsgi.ctx.get_abspath("workdir/gazdagret.cache.json"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");

    let expected = r#"Hamzsabégi út	[1]
Törökugrató utca	[7], [10]
Tűzkő utca	[1], [2]"#;
    assert_eq!(result, expected);
}

/// Tests the missing house numbers page: the chkl output.
#[test]
fn test_missing_housenumbers_view_result_chkl() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
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
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);
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
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

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
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
    assert_eq!(result, "No existing streets");
}

/// Tests the missing house numbers page: the chkl output, no osm housenumbers case.
#[test]
fn test_missing_housenumbers_view_result_chkl_no_osm_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_housenumbers_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
    assert_eq!(result, "No reference house numbers");
}

/// Tests the missing house numbers page: the txt output, no osm streets case.
#[test]
fn test_missing_housenumbers_view_result_txt_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
    let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
    assert_eq!(result, "No existing streets");
}

/// Tests the missing house numbers page: the txt output, no osm housenumbers case.
#[test]
fn test_missing_housenumbers_view_result_txt_no_osm_housenumbers() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_housenumbers_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
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
    let ref_housenumbers_cache = context::tests::TestFileSystem::make_file();
    let ref_housenumbers2_cache = context::tests::TestFileSystem::make_file();
    let housenumbers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
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
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/update-result");

    let mut guard = housenumbers_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let prefix = test_wsgi.ctx.get_ini().get_uri_prefix().unwrap();
    let results = TestWsgi::find_all(
        &root,
        &format!(
            "body/a[@href='{}/missing-housenumbers/gazdagret/view-result']",
            prefix
        ),
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

    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-result");

    let uri = format!(
        "{}/missing-housenumbers/gazdagret/view-result",
        test_wsgi.ctx.get_ini().get_uri_prefix().unwrap()
    );
    let results = TestWsgi::find_all(
        &root,
        &format!("body/div[@id='toolbar']/a[@href='{}']", uri),
    );
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
            ("data/street-housenumbers-template.txt", &overpass_template),
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
        /*result_path=*/ "tests/network/overpass-housenumbers-gazdagret.csv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.ctx.set_network(&network_arc);
    let streets_value = context::tests::TestFileSystem::make_file();
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
            ("data/street-housenumbers-template.txt", &overpass_template),
            ("workdir/street-housenumbers-gazdagret.csv", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/update-result");

    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let results = TestWsgi::find_all(&root, "body");
    assert_eq!(results.len(), 1);
}

/// Tests handle_street_housenumbers(): if the update-result output on error is well-formed.
#[test]
fn test_housenumbers_update_result_error_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "",
        /*result_path=*/ "",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    test_wsgi.ctx.set_network(&network_arc);
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
            ("data/street-housenumbers-template.txt", &overpass_template),
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
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_housenumbers_path();
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);
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
    let streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-streets.percent", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");

    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
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
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-streets.percent", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/suspicious-relations/gazdagret/view-result");

    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let results = TestWsgi::find_all(&root, "body/table");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: if the output is well-formed, no osm streets case.
#[test]
fn test_missing_streets_no_osm_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
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
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: if the output is well-formed, no ref streets case.
#[test]
fn test_missing_streets_no_ref_streets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_ref_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
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
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");

    let results = TestWsgi::find_all(&root, "body/div[@id='no-ref-streets']");
    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: the txt output.
#[test]
fn test_missing_streets_view_result_txt() {
    let mut test_wsgi = TestWsgi::new();
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
        &test_wsgi.ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

    assert_eq!(result, "Only In Ref Nonsense utca\nOnly In Ref utca\n");
}

/// Tests the missing streets page: the chkl output.
#[test]
fn test_missing_streets_view_result_chkl() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
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

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.chkl");

    assert_eq!(result, "[ ] Only In Ref utca\n");
}

/// Tests the missing streets page: the txt output, no osm streets case.
#[test]
fn test_missing_streets_view_result_txt_no_osm_streets() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_osm_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

    assert_eq!(result, "No existing streets");
}

/// Tests the missing streets page: the txt output, no ref streets case.
#[test]
fn test_missing_streets_view_result_txt_no_ref_streets() {
    let mut test_wsgi = TestWsgi::new();
    let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let hide_path = relation.get_files().get_ref_streets_path();
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

    assert_eq!(result, "No reference streets");
}

/// Tests the missing streets page: if the view-query output is well-formed.
#[test]
fn test_missing_streets_view_query_well_formed() {
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

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-query");

    let results = TestWsgi::find_all(&root, "body/pre");

    assert_eq!(results.len(), 1);
}

/// Tests the missing streets page: the update-result output.
#[test]
fn test_missing_streets_update_result() {
    let mut test_wsgi = TestWsgi::new();
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
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
            ("workdir/streets-reference-gazdagret.lst", &streets_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/update-result");

    let mut guard = streets_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);

    let results = TestWsgi::find_all(&root, "body/div[@id='update-success']");
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
    let unit_arc: Arc<dyn context::Unit> = Arc::new(unit);
    ctx.set_unit(&unit_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let subprocess_arc: Arc<dyn context::Subprocess> = Arc::new(subprocess);
    test_wsgi.ctx.set_subprocess(&subprocess_arc);

    test_wsgi.get_dom_for_path("/webhooks/github");

    let subprocess = subprocess_arc
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
    let subprocess_arc: Arc<dyn context::Subprocess> = Arc::new(subprocess);
    ctx.set_subprocess(&subprocess_arc);
    let root = serde_json::json!({"ref": "refs/heads/stable"});
    let payload = serde_json::to_string(&root).unwrap();
    let query_string: String = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("payload", &payload)
        .finish();
    let buf = query_string.as_bytes().to_vec();
    let request = rouille::Request::fake_http("GET", "/", vec![], buf);

    webframe::handle_github_webhook(&request, &ctx).unwrap();

    let subprocess = subprocess_arc
        .as_any()
        .downcast_ref::<context::tests::TestSubprocess>()
        .unwrap();
    assert_eq!(subprocess.get_runs().is_empty(), true);
}

/// Tests handle_stats().
#[test]
fn test_handle_stats() {
    let mut test_wsgi = TestWsgi::new();

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/");

    let results = TestWsgi::find_all(&root, "body/h2");
    // 9 chart types + note
    assert_eq!(results.len(), 10);
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
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        test_wsgi.ctx.get_abspath("target/browser/osm.min.css"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let result = test_wsgi.get_css_for_path("/static/osm.min.css");

    assert_eq!(result.ends_with("}"), true);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let result = test_wsgi.get_txt_for_path("/robots.txt");

    assert_eq!(result, "User-agent: *\n");
}

/// Tests handle_stats_cityprogress(): if the output is well-formed.
#[test]
fn test_handle_stats_cityprogress_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let citycount_value = context::tests::TestFileSystem::make_file();
    citycount_value
        .borrow_mut()
        .write_all(b"budapest_11\t11\nbudapest_12\t12\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("workdir/stats/2020-05-10.citycount", &citycount_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/cityprogress");

    let results = TestWsgi::find_all(&root, "body/table/tr");
    // header; also budapest_11/budapest_12 are both in ref and osm
    assert_eq!(results.len(), 3);
}

/// Tests handle_stats_zipprogress(): if the output is well-formed.
#[test]
fn test_handle_stats_zipprogress_well_formed() {
    let mut test_wsgi = TestWsgi::new();

    let zips_value = context::tests::TestFileSystem::make_file();
    zips_value
        .borrow_mut()
        .write_all(b"1111\t10\n1121\t20\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[("workdir/stats/2020-05-10.zipcount", &zips_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/zipprogress");

    let results = TestWsgi::find_all(&root, "body/table/tr");
    // header; also 1111/1121 is both in ref and osm
    assert_eq!(results.len(), 3);
}

/// Tests handle_invalid_refstreets(): if the output is well-formed.
#[test]
fn test_handle_invalid_refstreets_well_formed() {
    let mut test_wsgi = TestWsgi::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    test_wsgi.get_ctx().set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/invalid-relations");

    let results = TestWsgi::find_all(&root, "body/h1/a");
    assert_eq!(results.is_empty(), false);
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
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let hide_path = test_wsgi.ctx.get_abspath("workdir/streets-gazdagret.csv");
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system_arc);

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/invalid-relations");

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
    let osm_streets = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &test_wsgi.ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-myrelation.csv", &osm_streets),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system: Arc<dyn context::FileSystem> = Arc::new(file_system);
    test_wsgi.ctx.set_file_system(&file_system);

    let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/invalid-relations");

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

    let ret = get_housenr_additional_count(&ctx, relation.get_files()).unwrap();

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
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.percent", &percent_value),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string("4.2", &ctx.get_abspath("workdir/gazdagret.percent"))
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.percent"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-streets.percent", &percent_value),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "80.0",
            &ctx.get_abspath("workdir/gazdagret-streets.percent"),
        )
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret-streets.percent"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
    let count_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-additional-streets.count", &count_value),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "42",
            &ctx.get_abspath("workdir/gazdagret-additional-streets.count"),
        )
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret-additional-streets.count"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
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
