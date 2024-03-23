/*
 * Copyright 2023 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the area_files module.

use super::*;
use crate::areas;

/// Tests RelationFiles::write_osm_json_streets(), when the json has duplicated streets.
#[test]
fn test_write_osm_json_streets_duplicate() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let result =
        std::fs::read_to_string("src/fixtures/network/overpass-streets-duplicate.json").unwrap();

    relation
        .get_files()
        .write_osm_json_streets(&ctx, &result)
        .unwrap();
}

/// Tests RelationFiles::write_osm_json_housenumbers(), when the json has duplicated housenumbers.
#[test]
fn test_write_osm_json_housenumbers_duplicate() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let result =
        std::fs::read_to_string("src/fixtures/network/overpass-housenumbers-duplicate.json")
            .unwrap();

    relation
        .get_files()
        .write_osm_json_housenumbers(&ctx, &result)
        .unwrap();
}

/// Tests write_whole_country(), when it gets non-JSON input.
#[test]
fn test_write_whole_country_non_json_input() {
    let ctx = context::tests::make_test_context().unwrap();

    let ret = write_whole_country(&ctx, "");

    assert!(ret.is_ok());
}
