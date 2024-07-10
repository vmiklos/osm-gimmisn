/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the missing_housenumbers module.

use super::*;
use std::io::Read;
use std::io::Seek;
use std::rc::Rc;

/// Tests main().
#[test]
fn test_main() {
    let argv = vec!["".to_string(), "gh195".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh195": {
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
            ("workdir/street-housenumbers-reference-gh195.lst", &ref_file),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '25', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '27-37', '');
             insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values ('0', '0', 'Kalotaszeg utca', '31', 'CIVIL');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh195', '24746223', 'Kalotaszeg utca', 'residential', '', 'asphalt', '', '');
             insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values ('gh195', '695548547', 'Kalotaszeg utca', 'residential', '', '', '', '');").unwrap();
    }
    {
        let mut relations = areas::Relations::new(&ctx).unwrap();
        let relation_name = "gh195";
        let relation = relations.get_relation(relation_name).unwrap();
        relation.write_ref_housenumbers().unwrap();
    }

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 0);
    buf.rewind().unwrap();
    let mut actual: Vec<u8> = Vec::new();
    buf.read_to_end(&mut actual).unwrap();
    assert_eq!(
        String::from_utf8(actual).unwrap(),
        "Kalotaszeg utca\t3\n[\"25\", \"27-37\", \"31*\"]\n"
    );
}

/// Tests main(), the failing case.
#[test]
fn test_main_error() {
    let argv = vec!["".to_string(), "gh195".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_rc: Rc<dyn context::Unit> = Rc::new(unit);
    ctx.set_unit(&unit_rc);

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 1);
}
