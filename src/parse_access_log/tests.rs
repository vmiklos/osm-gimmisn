/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the parse_access_log module.

use super::*;

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::Arc;

/// Tests check_top_edited_relations().
#[test]
fn test_check_top_edited_relations() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let old_citycount = b"foo\t0\n\
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
    let new_citycount = b"foo\t1000\n\
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
        is_complete_relation(&mut relations, "gazdagret").unwrap(),
        false
    );
}

/// Tests main().
#[test]
fn test_main() {
    let argv = ["".to_string(), "tests/mock/access_log".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let relations_path = ctx.get_abspath("data/relations.yaml");
    // 2020-05-09, so this will be recent
    let expected_args = format!("git blame --line-porcelain {}", relations_path);
    let expected_out = "\n\
author-time 1588975200\n\
\tujbuda:\n"
        .to_string();
    let outputs: HashMap<_, _> = vec![(expected_args, expected_out)].into_iter().collect();
    let subprocess = context::tests::TestSubprocess::new(&outputs);
    let subprocess_arc: Arc<dyn context::Subprocess> = Arc::new(subprocess);
    ctx.set_subprocess(&subprocess_arc);

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
    main(&argv, &mut buf, &ctx).unwrap();

    buf.seek(SeekFrom::Start(0)).unwrap();
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
