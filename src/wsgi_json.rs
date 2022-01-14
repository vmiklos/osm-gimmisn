/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Contains functionality specific to the json part of the web interface.

use crate::areas;
use crate::context;
use crate::overpass_query;
use crate::webframe;
use anyhow::Context;
use std::collections::HashMap;

/// Expected request_uri: e.g. /osm/streets/ormezo/update-result.json.
fn streets_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let relation = relations
        .get_relation(relation_name)
        .context("get_relation() failed")?;
    let query = relation.get_osm_streets_query()?;
    let mut ret: HashMap<String, String> = HashMap::new();
    match overpass_query::overpass_query(ctx, query) {
        Ok(buf) => {
            relation.get_files().write_osm_streets(ctx, &buf)?;
            ret.insert("error".into(), "".into())
        }
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

/// Expected request_uri: e.g. /osm/street-housenumbers/ormezo/update-result.json.
fn street_housenumbers_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let relation = relations.get_relation(relation_name)?;
    let query = relation.get_osm_housenumbers_query()?;
    let mut ret: HashMap<String, String> = HashMap::new();
    match overpass_query::overpass_query(ctx, query) {
        Ok(buf) => {
            relation.get_files().write_osm_housenumbers(ctx, &buf)?;
            ret.insert("error".into(), "".into())
        }
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/update-result.json.
fn missing_housenumbers_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let references = ctx.get_ini().get_reference_housenumber_paths()?;
    let relation = relations.get_relation(relation_name)?;
    let mut ret: HashMap<String, String> = HashMap::new();
    relation.write_ref_housenumbers(&references)?;
    ret.insert("error".into(), "".into());
    Ok(serde_json::to_string(&ret)?)
}

/// Expected request_uri: e.g. /osm/missing-streets/ormezo/update-result.json.
fn missing_streets_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let reference = ctx.get_ini().get_reference_street_path()?;
    let relation = relations.get_relation(relation_name)?;
    let mut ret: HashMap<String, String> = HashMap::new();
    relation.write_ref_streets(&reference)?;
    ret.insert("error".into(), "".into());
    Ok(serde_json::to_string(&ret)?)
}

/// Dispatches json requests based on their URIs.
pub fn our_application_json(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<rouille::Response> {
    let mut headers: webframe::Headers = Vec::new();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let output: String;
    if request_uri.starts_with(&format!("{}/streets/", prefix)) {
        output = streets_update_result_json(ctx, relations, request_uri)?;
    } else if request_uri.starts_with(&format!("{}/street-housenumbers/", prefix)) {
        output = street_housenumbers_update_result_json(ctx, relations, request_uri)?;
    } else if request_uri.starts_with(&format!("{}/missing-housenumbers/", prefix)) {
        output = missing_housenumbers_update_result_json(ctx, relations, request_uri)?;
    } else {
        // Assume that request_uri starts with prefix + "/missing-streets/".
        output = missing_streets_update_result_json(ctx, relations, request_uri)?;
    }
    let output_bytes = output.as_bytes().to_vec();
    headers.push((
        "Content-type".into(),
        "application/json; charset=utf-8".into(),
    ));
    Ok(webframe::make_response(200_u16, headers, output_bytes))
}

#[cfg(test)]
mod tests {
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
        let housenumbers_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            test_wsgi.get_ctx(),
            &[(
                "workdir/street-housenumbers-gazdagret.csv",
                &housenumbers_value,
            )],
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

        let root = test_wsgi.get_json_for_path("/street-housenumbers/gazdagret/update-result.json");

        assert_eq!(root.as_object().unwrap().contains_key("error"), true);
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

        let root =
            test_wsgi.get_json_for_path("/missing-housenumbers/gazdagret/update-result.json");

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
}
