/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the validator module.

use super::*;

/// Tests main(): valid relations.
#[test]
fn test_relations() {
    let paths = [
        "tests/data/relations.yaml",
        "tests/data/relation-gazdagret-filter-invalid-good.yaml",
        "tests/data/relation-gazdagret-filter-invalid-good2.yaml",
        "tests/data/relation-gazdagret-filter-valid-good.yaml",
        "tests/data/relation-gazdagret-filter-valid-good2.yaml",
    ];
    for path in paths {
        let argv = ["".to_string(), path.to_string()];
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let ctx = context::tests::make_test_context().unwrap();
        let ret = main(&argv, &mut buf, &ctx);
        assert_eq!(ret, 0);
    }
}

/// Tests the missing-osmrelation relations path.
#[test]
fn test_relations_missing_osmrelation() {
    // Set up arguments.
    let relations_yaml_path = "data/relations.yaml";
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv: &[String] = &["".into(), ctx.get_abspath(relations_yaml_path)];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let relations_yaml = context::tests::TestFileSystem::make_file();
    relations_yaml
        .borrow_mut()
        .write_all(
            br#"gazdagret:
# osmrelation is intentionally missing.
    refcounty: "01"
    refsettlement: "011"
"#,
        )
        .unwrap();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[(relations_yaml_path, &relations_yaml)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(argv, &mut buf, &ctx);

    assert_eq!(ret, 1);
    let expected = format!(
        "missing key 'gazdagret.osmrelation'\nfailed to validate {}\n",
        ctx.get_abspath(relations_yaml_path)
    );
    assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), expected);
}

/// Tests the missing-refcounty relations path.
#[test]
fn test_relations_missing_refcounty() {
    // Set up arguments.
    let relations_yaml_path = "data/relations.yaml";
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv: &[String] = &["".into(), ctx.get_abspath(relations_yaml_path)];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let relations_yaml = context::tests::TestFileSystem::make_file();
    relations_yaml
        .borrow_mut()
        .write_all(
            br#"gazdagret:
    osmrelation: 42
    # refcounty is intentionally missing.
    refsettlement: "011"
"#,
        )
        .unwrap();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[(relations_yaml_path, &relations_yaml)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(argv, &mut buf, &ctx);

    assert_eq!(ret, 1);
    let expected = format!(
        "missing key 'gazdagret.refcounty'\nfailed to validate {}\n",
        ctx.get_abspath(relations_yaml_path)
    );
    assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), expected);
}

/// Tests the missing-refsettlement relations path.
#[test]
fn test_relations_missing_refsettlement() {
    // Set up arguments.
    let relations_yaml_path = "data/relations.yaml";
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv: &[String] = &["".into(), ctx.get_abspath(relations_yaml_path)];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let relations_yaml = context::tests::TestFileSystem::make_file();
    relations_yaml
        .borrow_mut()
        .write_all(
            br#"gazdagret:
    osmrelation: 42
    refcounty: "01"
    # refsettlement is intentionally missing
"#,
        )
        .unwrap();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[(relations_yaml_path, &relations_yaml)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(argv, &mut buf, &ctx);

    assert_eq!(ret, 1);
    let expected = format!(
        "missing key 'gazdagret.refsettlement'\nfailed to validate {}\n",
        ctx.get_abspath(relations_yaml_path)
    );
    assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), expected);
}

/// Tests the happy relation path.
#[test]
fn test_relation() {
    // Set up arguments.
    let argv: &[String] = &["".into(), "tests/data/relation-gazdagret.yaml".into()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let ctx = context::tests::make_test_context().unwrap();
    let ret = main(argv, &mut buf, &ctx);
    assert_eq!(ret, 0);
    assert_eq!(buf.into_inner(), b"");
}

/// Asserts that a given input fails with a given error message.
fn assert_failure_msg(path: &str, expected: &str) {
    let argv: &[String] = &["".to_string(), path.to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let ctx = context::tests::make_test_context().unwrap();
    let ret = main(argv, &mut buf, &ctx);
    assert_eq!(ret, 1);
    assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), expected);
}

/// Asserts that a given input (path, content) fails with a given error message.
fn assert_failure_msg2(content: &str, expected: &str) {
    let path = "data/relation-myrelation.yaml";
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv: &[String] = &["".into(), ctx.get_abspath(path)];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());

    let file = context::tests::TestFileSystem::make_file();
    file.borrow_mut().write_all(content.as_bytes()).unwrap();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[(path, &file)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(argv, &mut buf, &ctx);

    assert_eq!(ret, 1);
    let expected = expected.replace("{0}", &ctx.get_abspath(path));
    assert_eq!(String::from_utf8(buf.into_inner()).unwrap(), expected);
}

/// Tests the relation path: bad source type.
#[test]
fn test_relation_source_bad_type() {
    let content = "source: 42\n";
    let expected = "expected value type for 'source' is str\nfailed to validate {0}\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters type.
#[test]
fn test_relation_filters_bad_type() {
    let content = r#"filters:
  'Budaörsi út':
    ranges: 42
"#;
    let expected = r#"failed to validate {0}

Caused by:
    filters.Budaörsi út.ranges: invalid type: integer `42`, expected a sequence at line 3 column 13
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad toplevel key name.
#[test]
fn test_relation_bad_key_name() {
    let content = "invalid: 42\n";
    let expected = r#"failed to validate {0}

Caused by:
    unknown field `invalid`, expected one of `additional-housenumbers`, `alias`, `filters`, `housenumber-letters`, `inactive`, `missing-streets`, `osm-street-filters`, `osmrelation`, `refcounty`, `refsettlement`, `refstreets`, `street-filters`, `source` at line 1 column 1
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad strfilters value type.
#[test]
fn test_relation_strfilters_bad_type() {
    let content = r#"street-filters:
  - 42
"#;
    let expected = "expected value type for 'street-filters[0]' is str\nfailed to validate {0}\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad refstreets value type.
#[test]
fn test_relation_refstreets_bad_value_type() {
    let content = r#"refstreets:
  'OSM Name 1': 42
"#;
    let expected = r#"expected value type for 'refstreets.OSM Name 1' is str
failed to validate {0}
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: quote in refstreets key or value.
#[test]
fn test_relation_refstreets_quote() {
    let content = r#"refstreets:
  OSM Name 1': 42'
"#;
    let expected = r#"expected no quotes in 'refstreets.OSM Name 1''
expected no quotes in value of 'refstreets.OSM Name 1''
failed to validate {0}
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filterssubkey name.
#[test]
fn test_relation_filters_bad_subkey() {
    let content = r#"filters:
  'Budaörsi út':
    unexpected: 42
"#;
    let expected = r#"failed to validate {0}

Caused by:
    filters.Budaörsi út: unknown field `unexpected`, expected one of `interpolation`, `invalid`, `ranges`, `valid`, `refsettlement`, `show-refstreet` at line 3 column 5
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> ... -> invalid subkey.
#[test]
fn test_relation_filters_invalid_bad2() {
    let content = r#"filters:
  'Budaörsi út':
    invalid: ['1c 1']
"#;
    let expected = "expected format for 'filters.Budaörsi út.invalid[0]' is '42', '42a' or '42/1'\nfailed to validate {0}\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad type for the filters -> ... -> invalid subkey.
#[test]
fn test_relation_filters_invalid_bad_type() {
    let content = r#"filters:
  'Budaörsi út':
    invalid: "hello"
"#;
    let expected = r#"failed to validate {0}

Caused by:
    filters.Budaörsi út.invalid: invalid type: string "hello", expected a sequence at line 3 column 14
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> ... -> ranges subkey.
#[test]
fn test_relation_filters_ranges_bad() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '137', end: '165', unexpected: 42}
"#;
    let expected = r#"failed to validate {0}

Caused by:
    filters.Budaörsi út.ranges[0]: unknown field `unexpected`, expected one of `end`, `refsettlement`, `start` at line 4 column 36
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> ... -> ranges -> end type.
#[test]
fn test_relation_filters_ranges_bad_end() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '137', end: 42}
"#;
    let expected = r#"expected end >= start for 'filters.Budaörsi út.ranges[0]'
expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'
failed to validate {0}
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> ... -> ranges -> if start/end is swapped type.
#[test]
fn test_relation_filters_ranges_start_end_swap() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '142', end: '42'}
"#;
    let expected =
        "expected end >= start for 'filters.Budaörsi út.ranges[0]'\nfailed to validate {0}\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> ... -> ranges -> if start/end is either both
/// even/odd or not.
#[test]
fn test_relation_filters_ranges_start_end_even_odd() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '42', end: '143'}
"#;
    let expected = "expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\nfailed to validate {0}\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> ... -> ranges -> start type.
#[test]
fn test_relation_filters_ranges_bad_start() {
    let expected = "expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\nfailed to validate tests/data/relation-gazdagret-filter-range-bad-start.yaml\n";
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-range-bad-start.yaml",
        expected,
    );
}

/// Tests the relation path: missing filters -> ... -> ranges -> start key.
#[test]
fn test_relation_filters_ranges_missing_start() {
    let expected = r#"failed to validate tests/data/relation-gazdagret-filter-range-missing-start.yaml

Caused by:
    filters.Budaörsi út.ranges[0]: missing field `start` at line 4 column 9
"#;
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-range-missing-start.yaml",
        expected,
    );
}

/// Tests the relation path: missing filters -> ... -> ranges -> end key.
#[test]
fn test_relation_filters_ranges_missing_end() {
    let expected = r#"failed to validate tests/data/relation-gazdagret-filter-range-missing-end.yaml

Caused by:
    filters.Budaörsi út.ranges[0]: missing field `end` at line 4 column 9
"#;
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-range-missing-end.yaml",
        expected,
    );
}

/// Tests the housenumber-letters key: bad type.
#[test]
fn test_relation_housenumber_letters_bad() {
    let content = "housenumber-letters: 42\n";
    let expected = r#"failed to validate {0}

Caused by:
    housenumber-letters: invalid type: integer `42`, expected a boolean at line 1 column 22
"#;
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad alias subkey.
#[test]
fn test_relation_alias_bad() {
    let content = "alias: [1]\n";
    let expected = "expected value type for 'alias[0]' is str\nfailed to validate {0}\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad type for the alias subkey.
#[test]
fn test_relation_filters_alias_bad_type() {
    let content = r#"alias: "hello"
"#;
    let expected = "failed to validate {0}\n\nCaused by:\n    alias: invalid type: string \"hello\", expected a sequence at line 1 column 8\n";
    assert_failure_msg2(content, expected);
}

/// Tests the relation path: bad filters -> show-refstreet value type.
#[test]
fn test_relation_filters_show_refstreet_bad() {
    let expected = r#"failed to validate tests/data/relation-gazdagret-filter-show-refstreet-bad.yaml

Caused by:
    filters.Hamzsabégi út.show-refstreet: invalid type: integer `42`, expected a boolean at line 3 column 21
"#;
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-show-refstreet-bad.yaml",
        expected,
    );
}

/// Tests the relation path: bad refstreets map, not 1:1.
#[test]
fn test_relation_refstreets_bad_map_type() {
    let expected = "osm and ref streets are not a 1:1 mapping in 'refstreets'\nfailed to validate tests/data/relation-gazdagret-refstreets-bad-map.yaml\n";
    assert_failure_msg(
        "tests/data/relation-gazdagret-refstreets-bad-map.yaml",
        expected,
    );
}

/// Tests the relation path: bad filters -> ... -> valid subkey.
#[test]
fn test_relation_filters_valid_bad2() {
    let expected = "expected format for 'filters.Budaörsi út.valid[0]' is '42', '42a' or '42/1'\nfailed to validate tests/data/relation-gazdagret-filter-valid-bad2.yaml\n";
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-valid-bad2.yaml",
        expected,
    );
}

/// Tests the relation path: bad type for the filters -> ... -> valid subkey.
#[test]
fn test_relation_filters_valid_bad_type() {
    let expected = r#"failed to validate tests/data/relation-gazdagret-filter-valid-bad-type.yaml

Caused by:
    filters.Budaörsi út.valid: invalid type: string "hello", expected a sequence at line 3 column 12
"#;
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-valid-bad-type.yaml",
        expected,
    );
}

/// Tests that we do not accept whitespace in the value of the 'start' key.
#[test]
fn test_start_whitespace() {
    let expected = "expected value type for 'filters.Budaörsi út.ranges[0].start' is a digit str\nfailed to validate tests/data/relation-gazdagret-filter-range-bad-start2.yaml\n";
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-range-bad-start2.yaml",
        expected,
    );
}

/// Tests that we do not accept whitespace in the value of the 'end' key.
#[test]
fn test_end_whitespace() {
    let expected = "expected value type for 'filters.Budaörsi út.ranges[0].end' is a digit str\nfailed to validate tests/data/relation-gazdagret-filter-range-bad-end2.yaml\n";
    assert_failure_msg(
        "tests/data/relation-gazdagret-filter-range-bad-end2.yaml",
        expected,
    );
}
