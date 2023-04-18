/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the parse_access_log module.

use super::*;

use std::io::Read;
use std::io::Seek;
use std::rc::Rc;

use context::FileSystem as _;

/// Tests check_top_edited_relations().
#[test]
fn test_check_top_edited_relations() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let old_citycount = b"CITY\tCNT\n
foo\t0\n\
city1\t0\n\
city2\t0\n\
city3\t0\n\
city4\t0\n\
bar\t0\n\
baz\t0\n";
    let old_citycount_value = context::tests::TestFileSystem::make_file();
    old_citycount_value
        .borrow_mut()
        .write_all(old_citycount)
        .unwrap();
    let new_citycount = b"CITY\tCNT\n
foo\t1000\n\
city1\t1000\n\
city2\t1000\n\
city3\t1000\n\
city4\t1000\n\
bar\t2\n\
baz\t2\n";
    let new_citycount_value = context::tests::TestFileSystem::make_file();
    new_citycount_value
        .borrow_mut()
        .write_all(new_citycount)
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-04-10.citycount", &old_citycount_value),
            ("workdir/stats/2020-05-10.citycount", &new_citycount_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let mut frequent_relations: HashSet<String> = ["foo".to_string(), "bar".to_string()]
        .iter()
        .cloned()
        .collect();
    check_top_edited_relations(&ctx, &mut frequent_relations).unwrap();

    assert_eq!(frequent_relations.contains("foo"), true);
    assert_eq!(frequent_relations.contains("city1"), true);
    assert_eq!(frequent_relations.contains("city2"), true);
    assert_eq!(frequent_relations.contains("city3"), true);
    assert_eq!(frequent_relations.contains("city4"), true);
    assert_eq!(frequent_relations.contains("bar"), false);
    assert_eq!(frequent_relations.contains("baz"), false);
}

/// Tests is_complete_relation().
#[test]
fn test_is_complete_relation() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = areas::Relations::new(&ctx).unwrap();
    assert_eq!(
        is_complete_relation(&ctx, &mut relations, "gazdagret").unwrap(),
        false
    );
}

/// Tests is_complete_relation_complete(), the complete case.
#[test]
fn test_is_complete_relation_complete() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let housenumbers_percent = context::tests::TestFileSystem::make_file();
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/gazdagret.percent", &housenumbers_percent)],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string("100.00", &ctx.get_abspath("workdir/gazdagret.percent"))
        .unwrap();
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    let ret = is_complete_relation(&ctx, &mut relations, "gazdagret").unwrap();

    assert_eq!(ret, true);
}

/// Tests main().
#[test]
fn test_main() {
    let argv = [
        "".to_string(),
        "src/fixtures/file-system/access_log".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let relations_path = ctx.get_abspath("data/relations.yaml");
    // 2020-05-09, so this will be recent
    let expected_args = format!("git blame --line-porcelain {relations_path}");
    let expected_out = "\n\
author-time 1588975200\n\
\tujbuda:\n"
        .to_string();
    let outputs: HashMap<_, _> = vec![(expected_args, expected_out)].into_iter().collect();
    let subprocess = context::tests::TestSubprocess::new(&outputs);
    let subprocess_rc: Rc<dyn context::Subprocess> = Rc::new(subprocess);
    ctx.set_subprocess(&subprocess_rc);

    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "inactiverelation": {
                "inactive": true,
            },
            "gazdagret": {
            },
            "nosuchrelation": {
                "inactive": true,
            },
            "ujbuda": {
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let frequent_relations = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/frequent-relations.csv", &frequent_relations),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(&argv, &mut buf, &ctx);

    assert_eq!(ret, 0);
    buf.rewind().unwrap();
    let mut actual: Vec<u8> = Vec::new();
    buf.read_to_end(&mut actual).unwrap();
    let actual = String::from_utf8(actual).unwrap();
    assert_eq!(
        actual.contains("data/relation-inactiverelation.yaml: set inactive: false\n"),
        true
    );
    assert_eq!(
        actual.contains("data/relation-gazdagret.yaml: set inactive: true\n"),
        true
    );
    assert_eq!(
        actual.contains("data/relation-nosuchrelation.yaml: set inactive: "),
        false
    );

    // This is not in the output because it's considered as a recent relation.
    assert_eq!(
        actual.contains("data/relation-ujbuda.yaml: set inactive: "),
        false
    );

    // This is not in the output as it's not a valid relation name.
    assert_eq!(actual.contains("budafokxxx"), false);

    // This is not in the output as it's a search bot, so such visits don't count.
    // Also, if this would be not ignored, it would push 'inactiverelation' out of the active
    // relation list.
    assert_eq!(actual.contains("gyomaendrod"), false);
}

/// Tests main(), the failing case: missing required parameter.
#[test]
fn test_main_error() {
    let argv = vec!["".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 1);
}
