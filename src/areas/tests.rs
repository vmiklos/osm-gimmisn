/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the areas module.

use super::*;
use rusqlite::types::FromSql as _;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

/// Tests normalize().
#[test]
fn test_normalize() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers =
        normalize(&relation, "139", "mystreet", &normalizers, &mut None, None).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["139"])
}

/// Tests normalize: when the number is not in range.
#[test]
fn test_normalize_not_in_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Budaörsi út": {
                    "ranges": [
                        {
                            "start": "1",
                            "end": "499",
                        }
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "999",
        "Budaörsi út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    assert_eq!(house_numbers.is_empty(), true);
}

/// Tests normalize: the case when the house number is not a number.
#[test]
fn test_normalize_not_a_number() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers =
        normalize(&relation, "x", "Budaörsi út", &normalizers, &mut None, None).unwrap();
    assert_eq!(house_numbers.is_empty(), true);
}

/// Tests normalize: the case when there is no filter for this street.
#[test]
fn test_normalize_nofilter() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers =
        normalize(&relation, "1", "Budaörs út", &normalizers, &mut None, None).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["1"])
}

/// Tests normalize: the case when ';' is a separator.
#[test]
fn test_normalize_separator_semicolon() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "1;2",
        "Budaörs út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["1", "2"])
}

/// Tests normalize: the 2-6 case means implicit 4.
#[test]
fn test_normalize_separator_interval() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers =
        normalize(&relation, "2-6", "mystreet", &normalizers, &mut None, None).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2", "4", "6"])
}

/// Tests normalize: the 5-8 case: means just 5 and 8 as the parity doesn't match.
#[test]
fn test_normalize_separator_interval_parity() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "5-8",
        "Budaörs út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["5", "8"])
}

/// Tests normalize: the 2-5 case: means implicit 3 and 4 (interpolation=all).
#[test]
fn test_normalize_separator_interval_interp_all() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Hamzsabégi út": {
                    "interpolation": "all",
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "2-5",
        "Hamzsabégi út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2", "3", "4", "5"])
}

/// Tests normalize: the case where x-y is partially filtered out.
#[test]
fn test_normalize_separator_interval_filter() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Budaörsi út": {
                    "ranges": [
                        {
                            "start": "137",
                            "end": "165",
                        }
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    // filter is 137-165
    let house_numbers = normalize(
        &relation,
        "163-167",
        "Budaörsi út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    // Make sure there is no 167.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["163", "165"])
}

/// Tests normalize: the case where x-y is nonsense: y is too large.
#[test]
fn test_normalize_separator_interval_block() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "2-2000",
        "mystreet",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    // Make sure that we simply ignore 2000: it's larger than the default <998 filter and the
    // 2-2000 range would be too large.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2"])
}

/// Tests normalize: the case where x-y is nonsense: y-x is too large.
#[test]
fn test_normalize_separator_interval_block2() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers =
        normalize(&relation, "2-56", "mystreet", &normalizers, &mut None, None).unwrap();
    // No expansions for 4, 6, etc.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2", "56"])
}

/// Tests normalize: the case where x-y is nonsense: x is 0.
#[test]
fn test_normalize_separator_interval_block3() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "0-42",
        "Budaörs út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    // No expansion like 0, 2, 4, etc.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["42"])
}

/// Tests normalize: the case where x-y is only partially useful: x is OK, but y is a suffix.
#[test]
fn test_normalize_separator_interval_block4() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "42-1",
        "Budaörs út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    // No "1", just "42".
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["42"])
}

/// Tests normalize: the * suffix is preserved.
#[test]
fn test_normalize_keep_suffix() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers =
        normalize(&relation, "1*", "Budaörs út", &normalizers, &mut None, None).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["1*"]);
    let house_numbers =
        normalize(&relation, "2", "Budaörs út", &normalizers, &mut None, None).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2"]);
}

/// Tests normalize: the case when ',' is a separator.
#[test]
fn test_normalize_separator_comma() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let house_numbers = normalize(
        &relation,
        "2,6",
        "Budaörs út",
        &normalizers,
        &mut None,
        None,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    // Same as ";", no 4.
    assert_eq!(actual, vec!["2", "6"]);
}

/// Tests Relation.get_osm_streets().
#[test]
fn test_relation_get_osm_streets() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('test', '1', 'B2', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('test', '2', 'B1', '', '', '', '', '');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('test', '0', 'HB2', 'HC2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('test', '1', 'HB1', 'HC1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('test', '2', '', 'HC0', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('test', '3', '', '', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/test', '0');"
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("test").unwrap();
    let actual: Vec<String> = relation
        .get_osm_streets(/*sorted_result=*/ true)
        .unwrap()
        .iter()
        .map(|i| i.get_osm_name().clone())
        .collect();
    let expected: Vec<String> = vec!["B1".into(), "B2".into(), "HB1".into(), "HB2".into()];
    assert_eq!(actual, expected);
}

/// Tests Relation.get_osm_streets(): the case when the street name is coming from a house
/// number (node).
#[test]
fn test_relation_get_osm_streets_street_is_node() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('myrelation', '3136661536', 'Bártfai utca', '52/b', '1115', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/myrelation', '0')",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let actual = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].get_osm_type(), "node");
}

/// Tests Relation.get_osm_streets(): the case when we have streets, but no house numbers.
#[test]
fn test_relation_get_osm_streets_no_house_number() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('myrelation', '1', 'mystreet', '', '', '', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let osm_streets = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
    let actual: Vec<_> = osm_streets.iter().map(|i| i.get_osm_name()).collect();
    let expected = vec!["mystreet"];
    assert_eq!(actual, expected);
}

/// Tests Relation.get_osm_streets(): when there is only an addr:conscriptionnumber.
#[test]
fn test_relation_get_osm_streets_conscriptionnumber() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('myrelation', '295291710', 'mystreet', '', '8272', '', '', '045/2', '', '', '', '', 'myname', 'way');
             insert into mtimes (page, last_modified) values ('housenumbers/myrelation', '0')",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let osm_streets = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
    let streets: Vec<_> = osm_streets.iter().map(|i| i.get_osm_name()).collect();
    // This is coming from a house number which has addr:street and addr:conscriptionnumber, but
    // no addr:housenumber.
    let expected: &String = &String::from("mystreet");
    assert_eq!(streets.contains(&expected), true);
}

/// Tests Relation.get_osm_streets_query().
#[test]
fn test_relation_get_osm_streets_query() {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_osm_streets_query().unwrap();
    assert_eq!(ret, "aaa 42 bbb 3600000042 ccc\n");
}

/// Tests Relation.get_osm_streets_json_query().
#[test]
fn test_relation_get_osm_streets_json_query() {
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
        .write_all(b"[out:csv(::id)] [timeout:425];\naaa @RELATION@ bbb @AREA@ ccc\n")
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_osm_streets_json_query().unwrap();
    assert_eq!(ret, "[out:json];\naaa 42 bbb 3600000042 ccc");
}

/// Tests Relation.get_osm_housenumbers_query().
#[test]
fn test_relation_get_osm_housenumbers_query() {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let ret = relation.get_osm_housenumbers_query().unwrap();
    assert_eq!(ret, "housenr aaa 42 bbb 3600000042 ccc\n");
}

/// Tests Relation.get_osm_housenumbers_json_query().
#[test]
fn test_relation_get_osm_housenumbers_json_query() {
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
        .write_all(b"[out:csv(::id)] [timeout:425];\nhousenr aaa @RELATION@ bbb @AREA@ ccc\n")
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let ret = relation.get_osm_housenumbers_json_query().unwrap();
    assert_eq!(ret, "[out:json];\nhousenr aaa 42 bbb 3600000042 ccc");
}

/// Tests RelationFiles.write_osm_streets().
#[test]
fn test_relation_files_write_osm_streets() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let result_from_overpass = String::from_utf8(
        std::fs::read("src/fixtures/network/overpass-streets-gazdagret.json").unwrap(),
    )
    .unwrap();
    relation
        .get_files()
        .write_osm_json_streets(&ctx, &result_from_overpass)
        .unwrap();
    assert_eq!(
        relation
            .get_files()
            .get_osm_json_streets(&ctx)
            .unwrap()
            .len(),
        4
    );
}

/// Tests RelationFiles.write_osm_housenumbers().
#[test]
fn test_relation_files_write_osm_housenumbers() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let result_from_overpass = String::from_utf8(
        std::fs::read("src/fixtures/network/overpass-housenumbers-gazdagret.json").unwrap(),
    )
    .unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    relation
        .get_files()
        .write_osm_json_housenumbers(&ctx, &result_from_overpass)
        .unwrap();
    assert_eq!(
        relation
            .get_files()
            .get_osm_json_housenumbers(&ctx)
            .unwrap()
            .len(),
        8
    );
}

/// Tests Relation::get_street_ranges().
#[test]
fn test_relation_get_street_ranges() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet1": {
                    "ranges": [
                        {
                            "start": "1",
                            "end": "3",
                        },
                    ],
                },
            },
            "refstreets": {
                "myosm": "myref",
            },
            "street-filters": [
                "mystreet2",
            ],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let filters = relation.get_street_ranges().unwrap();
    let mut expected_filters: HashMap<String, ranges::Ranges> = HashMap::new();
    expected_filters.insert(
        "mystreet1".into(),
        ranges::Ranges::new(vec![ranges::Range::new(1, 3, "")]),
    );
    assert_eq!(filters, expected_filters);

    let mut expected_streets: HashMap<String, String> = HashMap::new();
    expected_streets.insert("myosm".into(), "myref".into());
    assert_eq!(relation.get_config().get_refstreets(), expected_streets);

    let street_blacklist = relation.get_config().get_street_filters();
    assert_eq!(street_blacklist, ["mystreet2".to_string()]);
}

/// Tests Relation::get_street_ranges() error handling.
#[test]
fn test_relation_get_street_ranges_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    "ranges": [
                        {
                            "start": "foo",
                            "end": "3",
                        },
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();

    let ret = relation.get_street_ranges();

    assert_eq!(ret.is_err(), true);
}

/// Tests Relation::get_street_ranges() error handling, end case.
#[test]
fn test_relation_get_street_ranges_error_end() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    "ranges": [
                        {
                            "start": "1",
                            "end": "foo",
                        },
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();

    let ret = relation.get_street_ranges();

    assert_eq!(ret.is_err(), true);
}

/// Tests Relation::get_street_ranges(): when the filter file is empty.
#[test]
fn test_relation_get_street_ranges_empty() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("empty").unwrap();
    let filters = relation.get_street_ranges().unwrap();
    assert_eq!(filters.is_empty(), true);
}

/// Tests Relation::get_ref_street_from_osm_street().
#[test]
fn test_relation_get_ref_street_from_osm_street() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let mut street: String = "mystreet".into();
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    street = relation
        .get_config()
        .get_ref_street_from_osm_street(&street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011"]
    );
    assert_eq!(street, "mystreet");
}

/// Tests Relation::get_ref_street_from_osm_street(): street-specific refsettlement override.
#[test]
fn test_relation_get_ref_street_from_osm_street_refsettlement_override() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    // this would be 011 by default, but here it's overwritten at a street
                    // level
                    "refsettlement": "012",
                },
                "mystreet2": {
                    // make sure the above 012 is picked up, not this one
                    "refsettlement": "013",
                }
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "mystreet";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["012"]
    );
    assert_eq!(street, "mystreet");
}

/// Tests Relation.get_ref_street_from_osm_street(): OSM -> ref name mapping.
#[test]
fn test_relation_get_ref_street_from_osm_street_refstreets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-myrelation.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "OSM Name 1";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011"]
    );
    assert_eq!(street, "Ref Name 1");
}

/// Tests Relation.get_ref_street_from_osm_street(): a relation with an empty filter file.
#[test]
fn test_relation_get_ref_street_from_osm_street_emptyrelation() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "OSM Name 1";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011"]
    );
    assert_eq!(street, "OSM Name 1");
}

/// Tests Relation.get_ref_street_from_osm_street(): the refsettlement range-level override.
#[test]
fn test_relation_get_ref_street_from_osm_street_range_level_override() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    "ranges": [
                    {
                        "start": "1",
                        "end": "1",
                        "refsettlement": "013",
                    },
                    ]
                },
                "mystreet2": {
                    "ranges": [
                    {
                        "start": "1",
                        "end": "1",
                    },
                    ]
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "mystreet";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011", "013"]
    );
    // mystreet2 has ranges, but no refsettlement in the single range.
    assert_eq!(
        relation.get_config().get_street_refsettlement("mystreet2"),
        ["011"]
    );
    assert_eq!(street, "mystreet");
}

/// Tests make_turbo_query_for_streets().
#[test]
fn test_make_turbo_query_for_streets() {
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
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let from = ["A2".to_string()];
    let ret = make_turbo_query_for_streets(&relation, &from);
    let expected = r#"[out:json][timeout:425];
rel(2713748)->.searchRelation;
area(3602713748)->.searchArea;
(rel(2713748);
way["name"="A2"](r.searchRelation);
way["name"="A2"](area.searchArea);
);
out body;
>;
out skel qt;
{{style:
relation{width:3}
way{color:blue; width:4;}
}}"#;
    assert_eq!(ret, expected);
}

/// Tests Relation::get_ref_streets().
#[test]
fn test_relation_get_ref_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let ref_streets = ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&ctx, &ref_streets).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-gazdagret.yaml": {
            "refcounty": "01",
            "refsettlement": "011",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let streets = relation.get_ref_streets().unwrap();
    assert_eq!(
        streets,
        [
            "Hamzsabégi út",
            "Only In Ref Nonsense utca",
            "Only In Ref utca",
            "Ref Name 1",
            "Törökugrató utca",
            "Tűzkő utca"
        ]
    );
}

/// Tests Relation::get_osm_housenumbers().
#[test]
fn test_relation_get_osm_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('myrelation', '1', 'mystreet', '1', '', '', '', '', '', '', '', '', '', 'node');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let street_name = "mystreet";
    let mut relation = relations.get_relation(relation_name).unwrap();
    let house_numbers = relation.get_osm_housenumbers(street_name).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["1"]);
}

/// Tests Relation::get_osm_housenumbers(): the case when addr:place is used instead of addr:street.
#[test]
fn test_relation_get_osm_housenumbers_addr_place() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('myrelation', '3136661536', '', '52/b', '1115', 'myplace', '', '', '', '', '', '', '', 'node');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let mut relation = relations.get_relation(relation_name).unwrap();
    let street_name = "myplace";

    let house_numbers = relation.get_osm_housenumbers(street_name).unwrap();

    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["52"]);
}

/// Tests Relation::get_missing_housenumbers().
#[test]
fn test_relation_get_missing_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
                    "invalid": [ "11", "12" ],
                }
            },
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
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
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '3', 'Tűzkő utca', '9', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '4', 'Tűzkő utca', '10', '', '', '', '', '', '', '', '', '', 'node');
            insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '5', 'OSM Name 1', '1', '', '', '', '', '', '', '', '', '', 'node');
            insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '6', 'OSM Name 1', '2', '', '', '', '', '', '', '', '', '', 'node');
            insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '7', 'Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
            insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '8', 'Second Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    let missing_housenumbers = relation.get_missing_housenumbers().unwrap();
    let ongoing_streets_strs: Vec<_> = missing_housenumbers
        .ongoing_streets
        .iter()
        .map(|numbered_street| {
            let numbers: Vec<_> = numbered_street
                .house_numbers
                .iter()
                .map(|i| i.get_number())
                .collect();
            (numbered_street.street.get_osm_name().clone(), numbers)
        })
        .collect();
    // Notice how 11 and 12 is filtered out by the 'invalid' mechanism for 'Törökugrató utca'.
    assert_eq!(
        ongoing_streets_strs,
        [
            ("Törökugrató utca".to_string(), vec!["7", "10"]),
            ("Tűzkő utca".to_string(), vec!["1", "2"]),
            ("Hamzsabégi út".to_string(), vec!["1"])
        ]
    );
    let expected = [
        ("OSM Name 1".to_string(), vec!["1", "2"]),
        ("Törökugrató utca".to_string(), vec!["1", "2"]),
        ("Tűzkő utca".to_string(), vec!["9", "10"]),
    ];
    let done_streets_strs: Vec<_> = missing_housenumbers
        .done_streets
        .iter()
        .map(|numbered_street| {
            let numbers: Vec<_> = numbered_street
                .house_numbers
                .iter()
                .map(|i| i.get_number())
                .collect();
            (numbered_street.street.get_osm_name().clone(), numbers)
        })
        .collect();
    assert_eq!(done_streets_strs, expected);
}

/// Tests Relation::get_lints().
#[test]
fn test_relation_get_lints() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": [ "1", "11", "12" ],
                }
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    let _missing_housenumbers = relation.get_missing_housenumbers().unwrap();

    let lints = relation.get_lints();

    assert_eq!(lints.len(), 1);
    let lint = lints[0].clone();
    assert_eq!(lint.relation_name, "gazdagret");
    assert_eq!(lint.street_name, "Törökugrató utca");
    assert_eq!(lint.source, RelationLintSource::Invalid);
    assert_eq!(format!("{:?}", lint.source), "Invalid");
    assert_eq!(lint.housenumber, "1");
    assert_eq!(lint.reason, RelationLintReason::CreatedInOsm);
    assert_eq!(
        RelationLintReason::DeletedFromRef.to_string(),
        "deleted-from-ref"
    );
    assert_eq!(
        format!("{:?}", RelationLintReason::CreatedInOsm),
        "CreatedInOsm"
    );
}

/// Tests Relation::get_lints(), the housenumber-letters=true case.
#[test]
fn test_relation_get_lints_hn_letters() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-myrelation.yaml": {
            "refcounty": "0",
            "refsettlement": "0",
            "filters": {
                "Tolvajos tanya": {
                    "invalid": [ "52b" ],
                }
            },
            "housenumber-letters": true,
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-myrelation.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Tolvajos tanya', '52/b', '');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('myrelation', '1', '', '52/b', '', 'Tolvajos tanya', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/myrelation', '0');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    let _missing_housenumbers = relation.get_missing_housenumbers().unwrap();

    let lints = relation.get_lints();

    // Previously this failed, lints was empty.
    assert_eq!(lints.len(), 1);
    let lint = lints[0].clone();
    assert_eq!(lint.relation_name, "myrelation");
    assert_eq!(lint.street_name, "Tolvajos tanya");
    assert_eq!(lint.source, RelationLintSource::Invalid);
    assert_eq!(format!("{:?}", lint.source), "Invalid");
    assert_eq!(lint.housenumber, "52/B");
    assert_eq!(lint.reason, RelationLintReason::CreatedInOsm);
}

/// Tests Relation::get_lints(), the out-of-range case.
#[test]
fn test_relation_get_lints_out_of_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh3073": {
                "refcounty": "0",
                "refsettlement": "0",
            },
        },
        "relation-gh3073.yaml": {
            "filters": {
                "Hadak útja": {
                    "invalid": [ "3" ],
                    "ranges": [
                        {"start": "5", "end": "7"},
                    ],
                },
                "Hadak útja2": {
                    "invalid": [ "3" ],
                    "ranges": [
                        {"start": "5", "end": "7"},
                    ],
                },
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gh3073.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Hadak útja', '3', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh3073', '7988705', 'Hadak útja', 'residential', '', 'asphalt', '', 'way');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh3073', '7988706', 'Hadak útja2', 'residential', '', 'asphalt', '', 'way');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh3073";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    let _missing_housenumbers = relation.get_missing_housenumbers().unwrap();

    let lints = relation.get_lints();

    assert!(!lints.is_empty());
    let lint = lints[0].clone();
    assert_eq!(lint.relation_name, "gh3073");
    assert_eq!(lint.street_name, "Hadak útja");
    assert_eq!(lint.source, RelationLintSource::Invalid);
    assert_eq!(lint.housenumber, "3");
    assert_eq!(lint.reason, RelationLintReason::OutOfRange);
    assert_eq!(RelationLintReason::OutOfRange.to_string(), "out-of-range");
}

/// Tests Relation::write_lints().
#[test]
fn test_relation_write_lints() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refcounty": "0",
                "refsettlement": "0",
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": [ "1", "11", "12" ],
                }
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    let _missing_housenumbers = relation.get_missing_housenumbers().unwrap();

    relation.write_lints().unwrap();

    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn.prepare("select count(*) from relation_lints").unwrap();
    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        let count: i64 = row.get(0).unwrap();
        assert_eq!(count, 1);
    }
}

/// Sets the housenumber_letters property from code.
fn set_config_housenumber_letters(config: &mut RelationConfig, housenumber_letters: bool) {
    config.dict.housenumber_letters = Some(housenumber_letters);
}

/// Sets the 'filters' key from code.
fn set_config_filters(config: &mut RelationConfig, filters: &HashMap<String, RelationFiltersDict>) {
    config.dict.filters = Some(filters.clone());
}

/// Tests Relation::get_missing_housenumbers(): 7/A is detected when 7/B is already mapped.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh267": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/street-housenumbers-reference-gh267.lst", &ref_file),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '1/1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '1/2', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '3', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '5', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '7', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '7/A', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '7/B', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '7a', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '7b', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '7 c', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '9', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '9 AB', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '11', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '11 ABC', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '13', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '13-15', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '13-15 B', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh267', '24746223', 'Kalotaszeg utca', 'residential', '', 'asphalt', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh267', '695548547', 'Kalotaszeg utca', 'residential', '', '', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh267";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.house_numbers);
    let mut housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
    // Make sure that 1/1 shows up in the output: it's not the same as '1' or '11'.
    let expected = [
        "1", "1/1", "1/2", "3", "5", "7", "7/A", "7/B", "7/C", "9", "11", "13", "13-15",
    ];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_housenumbers(): how 'invalid' interacts with normalization.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_invalid() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh296": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/street-housenumbers-reference-gh296.lst", &ref_file),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Rétköz utca', '9', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Rétköz utca', '9/A', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Rétköz utca', '9 A 1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Rétköz utca', '47/49D', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh296', '24746223', 'Rétköz utca', 'residential', '', 'asphalt', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh296";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    // Set custom 'invalid' map.
    let filters: HashMap<String, RelationFiltersDict> = serde_json::from_value(serde_json::json!({
        "Rétköz utca": {
            "invalid": ["9", "47"]
        }
    }))
    .unwrap();
    set_config_filters(&mut config, &filters);
    relation.set_config(&config);
    let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.house_numbers);
    let housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    // Notice how '9 A 1' is missing here: it's not a simple house number, so it gets normalized
    // to just '9' and the above filter silences it.
    let expected = ["9/A"];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_housenumbers(): how 'invalid' interacts with housenumber-letters: true or false.
#[test]
fn test_relation_get_missing_housenumbers_invalid_simplify() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kővirág sor', '37/B', '');",
         )
         .unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-myrelation.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();

    // Default case: housenumber-letters=false.
    {
        let filters: HashMap<String, RelationFiltersDict> =
            serde_json::from_value(serde_json::json!({
                "Kővirág sor": {
                    "invalid": ["37b"]
                }
            }))
            .unwrap();
        let mut config = relation.get_config().clone();
        set_config_filters(&mut config, &filters);
        relation.set_config(&config);
        let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
        // Note how 37b from invalid is simplified to 37; and how 37/B from ref is simplified to
        // 37 as well, so we find the match.
        assert_eq!(ongoing_streets.is_empty(), true);
    }

    // Opt-in case: housenumber-letters=true.
    {
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let filters: HashMap<String, RelationFiltersDict> =
            serde_json::from_value(serde_json::json!({
                "Kővirág sor": {
                    "invalid": ["37b"]
                }
            }))
            .unwrap();
        set_config_filters(&mut config, &filters);
        relation.set_config(&config);
        let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
        // In this case 37b from invalid matches 37/B from ref.
        assert_eq!(ongoing_streets.is_empty(), true);
    }

    // Make sure out-of-range invalid elements are just ignored and no exception is raised.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let filters: HashMap<String, RelationFiltersDict> = serde_json::from_value(serde_json::json!({
        "Kővirág sor": {
            "invalid": ["5"],
            "ranges": [{"start": "1", "end": "3"}],
        }
    }))
    .unwrap();
    set_config_filters(&mut config, &filters);
    relation.set_config(&config);
    relation.get_missing_housenumbers().unwrap();
}

/// Tests Relation::get_missing_housenumbers(): '42 A' vs '42/A' is recognized as a match.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_normalize() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh286": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/street-housenumbers-reference-gh286.lst", &ref_file),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Királyleányka utca', '10 A', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Királyleányka utca', '10 B', '');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gh286', '460828116', 'Királyleányka utca', '10/B', '1112', '', '', '', '', '', '', '', '', 'way');
             insert into mtimes (page, last_modified) values ('housenumbers/gh286', '0');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh286";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.house_numbers);
    let housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    // Note how 10/B is not in this list.
    let expected = ["10/A"];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_housenumbers(): '42/A*' and '42/a' matches.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_source_suffix() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh299": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/street-housenumbers-reference-gh299.lst", &ref_file),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Bártfai utca', '52/B', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh299";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
    // Note how '52/B*' is not in this list.
    assert_eq!(ongoing_streets.len(), 0);
}

/// Tests Relation::get_missing_housenumbers(): 'a' is not stripped from '1;3a'.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_normalize_semicolon() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh303": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/street-housenumbers-reference-gh303.lst", &ref_file),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Albert utca', '43/A', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Albert utca', '43/B', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Albert utca', '43/C', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Albert utca', '43/D', '');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gh303', '6852648009', 'Albert utca', '43/B;43/C', '1119', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/gh303', '0');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh303";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let ongoing_streets = relation.get_missing_housenumbers().unwrap().ongoing_streets;
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.house_numbers);
    let mut housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
    // Note how 43/B and 43/C is not here.
    let expected = ["43/A", "43/D"];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_streets().
#[test]
fn test_relation_get_missing_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let ref_streets = ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&ctx, &ref_streets).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-gazdagret.yaml": {
            "refcounty": "01",
            "refsettlement": "011",
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
            "street-filters": [
                "Only In Ref Nonsense utca",
            ],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let (only_in_reference, in_both) = relation.get_missing_streets().unwrap();

    // Note that 'Only In Ref Nonsense utca' is missing from this list.
    assert_eq!(only_in_reference, ["Only In Ref utca"]);

    assert_eq!(
        in_both,
        [
            "Hamzsabégi út",
            "Ref Name 1",
            "Törökugrató utca",
            "Tűzkő utca"
        ]
    );
}

/// Tests Relation::get_additional_streets().
#[test]
fn test_relation_get_additional_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let ref_streets = ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&ctx, &ref_streets).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "osm-street-filters": [
                "Second Only In OSM utca",
            ],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
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
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let only_in_osm = relation
        .get_additional_streets(/*sorted_result=*/ true)
        .unwrap();

    assert_eq!(only_in_osm, [util::Street::from_string("Only In OSM utca")]);

    // This is filtered out, even if it's OSM-only.
    let osm_street_blacklist = relation.get_config().get_osm_street_filters();
    assert_eq!(osm_street_blacklist, ["Second Only In OSM utca"]);
}

/// Tests Relation::get_additional_streets(): when the osm-street-filters key is missing.
#[test]
fn test_relation_get_additional_streets_no_osm_street_filters() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kővirág sor', '37/B', '');",
         )
         .unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "refcounty": "0",
                "refsettlement": "0",
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-myrelation.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    assert_eq!(
        relation.get_config().get_osm_street_filters().is_empty(),
        true
    );
}

/// Relation::get_additional_housenumbers().
#[test]
fn test_relation_get_additional_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
                "Second Only In OSM utca": {
                    "valid": ['1'],
                },
            },
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();
    let only_in_osm = relation.get_additional_housenumbers().unwrap();
    let only_in_osm_strs: Vec<_> = only_in_osm
        .iter()
        .map(|numbered_street| {
            let numbers: Vec<_> = numbered_street
                .house_numbers
                .iter()
                .map(|i| i.get_number())
                .collect();
            (numbered_street.street.get_osm_name(), numbers)
        })
        .collect();
    // Note how Second Only In OSM utca 1 is filtered out explicitly.
    assert_eq!(
        only_in_osm_strs,
        [(&"Only In OSM utca".to_string(), vec!["1"])]
    );
}

/// Unwraps an escaped matrix of rust.PyDocs into a string matrix.
fn table_doc_to_string(table: &[Vec<yattag::Doc>]) -> Vec<Vec<String>> {
    let mut table_content = Vec::new();
    for row in table {
        let mut row_content = Vec::new();
        for cell in row {
            row_content.push(cell.get_value());
        }
        table_content.push(row_content);
    }
    table_content
}

/// Tests Relation::write_missing_housenumbers().
#[test]
fn test_relation_write_missing_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
            "street-filters": ["Only In Ref Nonsense utca"],
            "osm-street-filters": ["Second Only In OSM utca"],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file,
            ),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
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
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '3', 'Tűzkő utca', '9', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '4', 'Tűzkő utca', '10', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '5', 'OSM Name 1', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '6', 'OSM Name 1', '2', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '7', 'Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '8', 'Second Only In OSM utca', '1', '', '', '', '', '', '', '', '', '', 'node');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (todo_street_count, todo_count, done_count, percent, table) = ret;
    assert_eq!(todo_street_count, 3);
    assert_eq!(todo_count, 5);
    assert_eq!(done_count, 6);
    assert_eq!(format!("{percent:.2}"), "54.55");
    let string_table = table_doc_to_string(&table);
    assert_eq!(
        string_table,
        [
            ["Street name", "Missing count", "House numbers"],
            ["Törökugrató utca", "2", "7<br />10"],
            ["Tűzkő utca", "2", "1<br />2"],
            ["Hamzsabégi út", "1", "1"]
        ]
    );
    assert_eq!(relation.get_osm_housenumber_coverage().unwrap(), "54.55");
}

/// Tests Relation::write_missing_housenumbers(): the case when percent can't be determined.
#[test]
fn test_relation_write_missing_housenumbers_empty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let json_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/cache-empty.json", &json_value)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/cache-empty.json"),
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "empty";
    let mut relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (_todo_street_count, _todo_count, _done_count, percent, _table) = ret;
    assert_eq!(percent, 100.0);
    assert_eq!(relation.config.get_filters().is_none(), true);
}

/// Tests Relation::write_missing_housenumbers(): the case when the street is interpolation=all and coloring is wanted.
#[test]
fn test_relation_write_missing_housenumbers_interpolation_all() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let json_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/cache-budafok.json", &json_value)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/cache-budafok.json");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('budafok', '458338075', 'Vöröskúti határsor', 'residential', '', 'asphalt', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "budafok";
    let mut relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (_todo_street_count, _todo_count, _done_count, _percent, table) = ret;
    let string_table = table_doc_to_string(&table);
    assert_eq!(
        string_table,
        [
            ["Street name", "Missing count", "House numbers"],
            [
                "Vöröskúti határsor",
                "4",
                "2, 12, 34, <span style=\"color: blue;\">36</span>"
            ]
        ]
    );
    assert_eq!(relation.has_osm_housenumber_coverage().unwrap(), true);
}

/// Tests Relation::write_missing_housenumbers(): sorting is performed after range reduction.
#[test]
fn test_relation_write_missing_housenumbers_sorting() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'A utca', '2-10', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'B utca', '1', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'B utca', '3', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('myrelation', '46985966', 'A utca', 'residential', '', 'asphalt', '', 'way');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('myrelation', '46985967', 'B utca', 'residential', '', 'asphalt', '', 'way');",
        )
        .unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-myrelation.yaml": {
            "refcounty": "0",
            "refsettlement": "0",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-myrelation.lst",
                &ref_file,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let mut relation = relations.get_relation(relation_name).unwrap();
    relation.write_ref_housenumbers().unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (_todo_street_count, _todo_count, _done_count, _percent, table) = ret;
    let string_table = table_doc_to_string(&table);
    // Note how 'A utca' is logically 5 house numbers, but it's a single range, so it's
    // ordered after 'B utca'.
    assert_eq!(
        string_table,
        [
            ["Street name", "Missing count", "House numbers"],
            ["B utca", "2", "1, 3"],
            ["A utca", "1", "2-10"]
        ]
    );
    assert_eq!(relation.has_osm_housenumber_coverage().unwrap(), true);
    let conn = ctx.get_database_connection().unwrap();
    let json: String = conn
        .query_row(
            "select json from missing_housenumbers_cache where relation = ?1",
            ["myrelation"],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!json.is_empty());
}

/// Tests Relation::write_missing_streets().
#[test]
fn test_write_missing_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let ref_streets = ctx.get_ini().get_reference_street_path().unwrap();
        util::build_street_reference_index(&ctx, &ref_streets).unwrap();
    }
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
            "street-filters": ["Only In Ref Nonsense utca"],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let expected = "80.00".to_string();

    let ret = relation.write_missing_streets().unwrap();

    let (todo_count, done_count, percent, streets) = ret;

    assert_eq!(todo_count, 1);
    assert_eq!(done_count, 4);
    assert_eq!(format!("{percent:.2}"), "80.00");
    assert_eq!(streets, ["Only In Ref utca"]);
    assert_eq!(relation.get_osm_street_coverage().unwrap(), expected);
}

/// Tests Relation::write_missing_streets(): the case when percent can't be determined.
#[test]
fn test_write_missing_streets_empty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-empty.yaml": {
            "refcounty": "42",
            "refsettlement": "43",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "empty";
    let relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_streets().unwrap();

    assert_eq!(relation.has_osm_street_coverage().unwrap(), true);
    let (_todo_count, _done_count, percent, _streets) = ret;
    assert_eq!(format!("{percent:.2}"), "100.00");
}

/// Tests Relation::write_ref_housenumbers().
#[test]
fn test_relation_writer_ref_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers2 = context::tests::TestFileSystem::make_file();
    ref_housenumbers2
        .borrow_mut()
        .write_all(
            r#"COUNTY_CODE	SETTLEMENT_CODE	STREET	HOUSENUMBER	COMMENT
01	011	Márton Áron tér	1	comment
01	011	Márton Áron tér	2	
"#
            .as_bytes(),
        )
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_value,
            ),
            (
                "workdir/refs/hazszamok_kieg_20190808.tsv",
                &ref_housenumbers2,
            ),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let references = ctx.get_ini().get_reference_housenumber_paths().unwrap();
    util::build_reference_index(&ctx, &references).unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '5', 'Márton Áron tér', '', '', '', '', '');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let expected = r#"Hamzsabégi út	1	
Márton Áron tér	1*	comment
Márton Áron tér	2*	
Ref Name 1	1	
Ref Name 1	2	
Törökugrató utca	1	
Törökugrató utca	10	
Törökugrató utca	11	
Törökugrató utca	12	
Törökugrató utca	2	
Törökugrató utca	7	
Tűzkő utca	1	
Tűzkő utca	10	
Tűzkő utca	2	
Tűzkő utca	9	
"#;
    let relation = relations.get_relation(relation_name).unwrap();

    relation.write_ref_housenumbers().unwrap();

    let mut guard = ref_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(String::from_utf8(actual).unwrap(), expected);
}

/// Tests Relation::write_ref_housenumbers(): the case when the refcounty code is missing in the reference.
#[test]
fn test_relation_writer_ref_housenumbers_nosuchrefcounty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "nosuchrefcounty": {
                "refsettlement": "43",
                "refcounty": "98",
                "refsettlement": "99",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-nosuchrefcounty.lst",
                &ref_value,
            ),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "nosuchrefcounty";
    let relation = relations.get_relation(relation_name).unwrap();

    relation.write_ref_housenumbers().unwrap();
}

/// Tests Relation::write_ref_housenumbers(): the case when the refsettlement code is missing in the reference.
#[test]
fn test_relation_writer_ref_housenumbers_nosuchrefsettlement() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "nosuchrefsettlement": {
                "refcounty": "01",
                "refsettlement": "99",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-nosuchrefsettlement.lst",
                &ref_value,
            ),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "nosuchrefsettlement";
    let relation = relations.get_relation(relation_name).unwrap();

    relation.write_ref_housenumbers().unwrap();
}

/// Tests the Relations struct.
#[test]
fn test_relations() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation1": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "myrelation2": {
                "osmrelation": 43,
                "refcounty": "43", // not 01
                "refsettlement": "011",
            },
            "myrelation3": {
                "osmrelation": 44,
                "refcounty": "01",
                "refsettlement": "99", // not 011
            },
        },
        "relation-myrelation2.yaml": {
            "inactive": true,
        },
        "relation-myrelation3.yaml": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let expected_relation_names = ["myrelation1", "myrelation2", "myrelation3"];
    assert_eq!(relations.get_names(), expected_relation_names);
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation2".to_string()),
        false
    );
    let mut osmids: Vec<_> = relations
        .get_relations()
        .unwrap()
        .iter()
        .map(|relation| relation.get_config().get_osmrelation())
        .collect();
    osmids.sort();
    assert_eq!(osmids, [42, 43, 44]);
    let ujbuda = relations.get_relation("myrelation3").unwrap();
    assert_eq!(ujbuda.get_config().should_check_missing_streets(), "only");

    relations.activate_all(true);
    let active_names = relations.get_active_names().unwrap();
    assert_eq!(active_names.contains(&"myrelation2".to_string()), true);

    // Allow seeing data of a relation even if it's not in relations.yaml.
    relations.get_relation("gh195").unwrap();

    // Test limit_to_refcounty().
    // 01
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        true
    );
    // 43
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation2".to_string()),
        true
    );
    relations
        .limit_to_refcounty(&Some(&"01".to_string()))
        .unwrap();
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        true
    );
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation2".to_string()),
        false
    );

    // Test limit_to_refsettlement().
    // 011
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        true
    );
    // 99
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation3".to_string()),
        true
    );
    relations
        .limit_to_refsettlement(&Some(&"99".to_string()))
        .unwrap();
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        false
    );
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation3".to_string()),
        true
    );
}

/// Tests Relations::limit_to_refarea().
#[test]
fn test_relations_limit_to_refarea() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation1": {
            },
            "myrelation2": {
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
    let mut relations = Relations::new(&ctx).unwrap();

    relations
        .limit_to_refarea(&Some(&"myrelation1".to_string()))
        .unwrap();

    let expected_relation_names = ["myrelation1"];
    assert_eq!(relations.get_names(), expected_relation_names);
}

/// Tests RelationConfig::should_check_missing_streets().
#[test]
fn test_relation_config_should_check_missing_streets() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_config().should_check_missing_streets();
    assert_eq!(ret, "only");
}

/// Tests RelationConfig::should_check_missing_streets(): the default.
#[test]
fn test_relation_config_should_check_missing_streets_default() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_config().should_check_missing_streets();
    assert_eq!(ret, "yes");
}

/// Tests refcounty_get_name().
#[test]
fn test_refcounty_get_name() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
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
    let relations = Relations::new(&ctx).unwrap();
    assert_eq!(relations.refcounty_get_name("01"), "Budapest");
    assert_eq!(relations.refcounty_get_name("99"), "");
}

/// Tests refcounty_get_refsettlement_ids().
#[test]
fn test_refcounty_get_refsettlement_ids() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "refcounty-names.yaml": {
            "01": "mycity",
        },
        "refsettlement-names.yaml": {
            "01": {
                "011": "myrelation1",
                "012": "myrelation1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = Relations::new(&ctx).unwrap();
    assert_eq!(
        relations.refcounty_get_refsettlement_ids("01"),
        ["011".to_string(), "012".to_string()]
    );
    assert_eq!(
        relations.refcounty_get_refsettlement_ids("99").is_empty(),
        true
    );
}

/// Tests Relations::new(), when refsettlement-names.yaml is invalid.
#[test]
fn test_relations_new_invalid_refsettlement_names() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "refcounty-names.yaml": {
            "01": "mycity",
        },
        "refsettlement-names.yaml": "hello",
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = Relations::new(&ctx);

    assert_eq!(ret.is_err(), true);
}

/// Tests refsettlement_get_name().
#[test]
fn test_refsettlement_get_name() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "refcounty-names.yaml": {
            "01": "mycity",
        },
        "refsettlement-names.yaml": {
            "01": {
                "011": "mysettlement",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = Relations::new(&ctx).unwrap();
    assert_eq!(
        relations.refsettlement_get_name("01", "011"),
        "mysettlement"
    );
    assert_eq!(relations.refsettlement_get_name("99", ""), "");
    assert_eq!(relations.refsettlement_get_name("01", "99"), "");
}

/// Tests Relalations::get_aliases().
#[test]
fn test_relations_get_aliases() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
            },
        },
        "relation-budafok.yaml": {
            "alias": ["budapest_22"],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    // Expect an alias -> canonicalname map.
    let mut expected = HashMap::new();
    expected.insert("budapest_22".to_string(), "budafok".to_string());
    assert_eq!(relations.get_aliases().unwrap(), expected);
}

/// Tests RelationConfig::get_street_is_even_odd().
#[test]
fn test_relation_config_get_street_is_even_odd() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Hamzsabégi út": {
                    "interpolation": "all",
                },
                "Teszt utca": {
                    "interpolation": "notall",
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    assert_eq!(
        relation.config.get_street_is_even_odd("Hamzsabégi út"),
        false
    );

    assert_eq!(relation.config.get_street_is_even_odd("Teszt utca"), true);
}

/// Tests RelationConfig::should_show_ref_street().
#[test]
fn test_relation_config_should_show_ref_street() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet1": {
                    "show-refstreet": false,
                },
                "mystreet2": {
                    "show-refstreet": true,
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    assert_eq!(relation.config.should_show_ref_street("mystreet1"), false);
    assert_eq!(relation.config.should_show_ref_street("mystreet2"), true);
}

/// Tests RelationConfig::is_active().
#[test]
fn test_relation_config_is_active() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    assert_eq!(relation.get_config().is_active(), true);
}

/// Tests Relation::numbered_streets_to_table(): when a street is not even-odd.
#[test]
fn test_relation_numbered_streets_to_table() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    "interpolation": "all",
                }
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let street = util::Street::new("mystreet", "mystreet", false, 0);
    let house_numbers = vec![
        util::HouseNumber::new("1", "1", ""),
        util::HouseNumber::new("2", "2", ""),
    ];
    let streets = vec![util::NumberedStreet {
        street,
        house_numbers,
    }];

    let (table, _todo_count) = relation.numbered_streets_to_table(&streets);

    assert_eq!(table.len(), 2);
    // Ignore header.
    let row = &table[1];
    assert_eq!(row.len(), 3);
    assert_eq!(row[0].get_value(), "mystreet");
    assert_eq!(row[1].get_value(), "2");
    // No line break here.
    assert_eq!(row[2].get_value(), "1, 2");
}

/// Tests RelationConfig::set_active().
#[test]
fn test_relation_config_set_active() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let mut config = relation.get_config().clone();
    assert_eq!(config.is_active(), true);
    config.set_active(false);
    assert_eq!(config.is_active(), false);
}

/// Tests get_invalid_filter_keys().
#[test]
fn test_get_invalid_filter_keys() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "mystreet": {
                    "interpolation": "all",
                }
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
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('gazdagret', '1', 'Törökugrató utca', '1', '', '', '', '', '', '', '', '', '', 'node');
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let ret = relation.get_invalid_filter_keys().unwrap();

    let expected: Vec<String> = vec!["mystreet".to_string()];
    assert_eq!(ret, expected);
}

/// Tests Relation::new() error handling.
#[test]
fn test_relation_new_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "invalidkey": 42
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();

    let ret = relations.get_relation("gazdagret");

    assert_eq!(ret.is_err(), true);
}

/// Tests RelationConfig::get_alias(), the case when the parent provides the data.
#[test]
fn test_relation_config_get_alias_parent() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "alias": ["myoldrelation"],
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();

    let ret = relation.get_config().get_alias();

    assert_eq!(ret, vec!["myoldrelation".to_string()]);
}

/// Tests Relation::normalize_invalids().
#[test]
fn test_relation_normalize_invalids() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Tűzkő utca": {
                    "ranges": [
                        {
                            "start": "1",
                            "end": "3",
                        }
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();

    let ret = relation
        .normalize_invalids("Tűzkő utca", &["5".to_string()])
        .unwrap();

    // This is empty because 5 is outside 1-3.
    assert_eq!(ret.is_empty(), true);
}

/// Tests Relations::is_new().
#[test]
fn test_relations_is_new() {
    // Case 1: active-new is false -> myrelation is not found.
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "inactive": true,
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();

    let actual = relations.get_active_names().unwrap();

    assert!(actual.is_empty());

    // Case 2: active-new is true -> myrelation is not found.
    relations.activate_new();

    let actual = relations.get_active_names().unwrap();

    assert_eq!(actual, vec!["myrelation".to_string()]);

    // Case 3: active-new is true and myrelation is not new -> myrelation is not found.
    let osm_housenumbers_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-myrelation.csv",
                &osm_housenumbers_value,
            ),
            (
                "workdir/street-housenumbers-reference-myrelation.lst",
                &ref_housenumbers_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into mtimes (page, last_modified) values ('streets/myrelation', '0');
             insert into mtimes (page, last_modified) values ('housenumbers/myrelation', '0');
             insert into osm_housenumber_coverages (relation_name, coverage, last_modified) values ('myrelation', '', '');
             insert into osm_street_coverages (relation_name, coverage, last_modified) values ('myrelation', '', '');",
        ).unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    relations.activate_new();

    let actual = relations.get_active_names().unwrap();

    assert!(actual.is_empty());
}

/// Tests Relations::is_inactive().
#[test]
fn test_relations_is_inactive() {
    // Case 1: active-invalid is false -> myrelation is not found.
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "Misspelled OSM Name 1": "OSM Name 1",
            },
            "inactive": true,
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '1', 'Tűzkő utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '2', 'Törökugrató utca', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '3', 'OSM Name 1', '', '', '', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gazdagret', '4', 'Hamzsabégi út', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/gazdagret', '0');",
        )
        .unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();

    let actual = relations.get_active_names().unwrap();

    assert!(actual.is_empty());

    // Case 2: active-invalid is true -> myrelation is found.
    relations.activate_invalid();

    let actual = relations.get_active_names().unwrap();

    assert_eq!(actual, vec!["gazdagret".to_string()]);
}

/// Tests Relations::is_inactive(), the no-osm-streets case.
#[test]
fn test_relations_is_inactive_no_osm_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
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
            "inactive": true,
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
    let mut relations = Relations::new(&ctx).unwrap();
    relations.activate_invalid();

    let actual = relations.get_active_names().unwrap();

    assert!(actual.is_empty());
}

/// Tests RelationLintSource::try_from().
#[test]
fn test_relation_lint_source_try_from() {
    let result = RelationLintSource::try_from("test");
    assert_eq!(result.is_err(), true);
}

/// Tests RelationLintSource::column_result().
#[test]
fn test_relation_lint_source_column_result() {
    let value_ref = rusqlite::types::ValueRef::from("test");

    let result = RelationLintSource::column_result(value_ref);

    assert_eq!(result.is_err(), true);
}

/// Tests RelationLintReason::try_from().
#[test]
fn test_relation_lint_reason_try_from() {
    let result = RelationLintReason::try_from("test");
    assert_eq!(result.is_err(), true);
}

/// Tests RelationLintReason::column_result().
#[test]
fn test_relation_lint_reason_column_result() {
    let value_ref = rusqlite::types::ValueRef::from("test");

    let result = RelationLintReason::column_result(value_ref);

    assert_eq!(result.is_err(), true);
}

/// Tests RelationLint's Ord impl for source.
#[test]
fn test_relation_list_ord_source() {
    let lint1 = {
        let relation_name = "".to_string();
        let street_name = "".to_string();
        let source = RelationLintSource::Range;
        let housenumber = "1".to_string();
        let reason = RelationLintReason::CreatedInOsm;
        let id: u64 = 0;
        let object_type = "".to_string();
        RelationLint {
            relation_name,
            street_name,
            source,
            housenumber,
            reason,
            id,
            object_type,
        }
    };
    let lint2 = {
        let relation_name = "".to_string();
        let street_name = "".to_string();
        let source = RelationLintSource::Invalid;
        let housenumber = "1".to_string();
        let reason = RelationLintReason::CreatedInOsm;
        let id: u64 = 0;
        let object_type = "".to_string();
        RelationLint {
            relation_name,
            street_name,
            source,
            housenumber,
            reason,
            id,
            object_type,
        }
    };
    assert_eq!(lint1.cmp(&lint2), std::cmp::Ordering::Less);
    assert_eq!(
        lint1.source.partial_cmp(&lint2.source).unwrap(),
        std::cmp::Ordering::Less
    );
}

/// Tests RelationLint's Ord impl for reason.
#[test]
fn test_relation_list_ord_reason() {
    let lint1 = {
        let relation_name = "".to_string();
        let street_name = "".to_string();
        let source = RelationLintSource::Range;
        let housenumber = "1".to_string();
        let reason = RelationLintReason::CreatedInOsm;
        let id: u64 = 0;
        let object_type = "".to_string();
        RelationLint {
            relation_name,
            street_name,
            source,
            housenumber,
            reason,
            id,
            object_type,
        }
    };
    let lint2 = {
        let relation_name = "".to_string();
        let street_name = "".to_string();
        let source = RelationLintSource::Range;
        let housenumber = "1".to_string();
        let reason = RelationLintReason::DeletedFromRef;
        let id: u64 = 0;
        let object_type = "".to_string();
        RelationLint {
            relation_name,
            street_name,
            source,
            housenumber,
            reason,
            id,
            object_type,
        }
    };
    assert_eq!(lint1.cmp(&lint2), std::cmp::Ordering::Less);
    assert_eq!(
        lint1.reason.partial_cmp(&lint2.reason).unwrap(),
        std::cmp::Ordering::Less
    );
}

/// Tests normalizer_contains(), the case when `osm_housenumber` is None.
#[test]
fn test_normalizer_contains_osm_housenumber_none() {
    let number: i64 = 5;
    let normalizer = ranges::Ranges::new(vec![ranges::Range::new(1, 3, "")]);
    let relation_name = "";
    let street_name = "mystreet";
    let mut lints: Vec<RelationLint> = Vec::new();
    assert!(!normalizer_contains(
        number,
        &normalizer,
        relation_name,
        street_name,
        &mut Some(&mut lints),
        None
    ));
}

#[test]
fn test_relation_get_osm_housenumber_split() {
    // Given a relation with housenumber-letters=true & with a housenumber 12a,12b:
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-myrelation.yaml": {
            "housenumber-letters": true,
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('myrelation', '1', 'mystreet', '', '', '', '', '');
             insert into mtimes (page, last_modified) values ('streets/myrelation', '0');
             insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values ('myrelation', '1', 'mystreet', '12a,12b', '', '', '', '', '', '', '', '', '', 'node');
             insert into mtimes (page, last_modified) values ('housenumbers/myrelation', '0');"
        ).unwrap();
    }
    let mut relations = Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("myrelation").unwrap();

    // When getting the osm housenumbers:
    let housenumbers = relation.get_osm_housenumbers("mystreet").unwrap();

    // Then make sure we split by ',': without the fix, this was a single '12' housenumber.
    assert_eq!(housenumbers.len(), 2);
    assert_eq!(housenumbers[0].get_number(), "12/A");
    assert_eq!(housenumbers[1].get_number(), "12/B");
}
