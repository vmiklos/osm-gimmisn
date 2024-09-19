/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the validator module.

use super::*;

/// Tests main(): valid relations.
#[test]
fn test_relations() {
    let content = r#"gazdagret:
    osmrelation: 2713748
    refcounty: "01"
    refsettlement: "011"
"#;
    let path = "data/relations.yaml";
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv: &[String] = &["".into(), ctx.get_abspath(path)];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let file = context::tests::TestFileSystem::make_file();
    file.borrow_mut().write_all(content.as_bytes()).unwrap();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[(path, &file)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(argv, &mut buf, &ctx);

    assert_eq!(ret, 0);
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
    let relations_yaml_path = "data/relation-gazdagret.yaml";
    let mut ctx = context::tests::make_test_context().unwrap();
    let argv: &[String] = &["".into(), ctx.get_abspath(relations_yaml_path)];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let relations_yaml = context::tests::TestFileSystem::make_file();
    relations_yaml
        .borrow_mut()
        .write_all(
            br#"filters:
  'my street':
    valid: ['1']
"#,
        )
        .unwrap();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[(relations_yaml_path, &relations_yaml)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(argv, &mut buf, &ctx);

    assert_eq!(ret, 0);
    assert_eq!(buf.into_inner(), b"");
}

/// Asserts that a given input (path, content) fails with a given error message.
fn assert_failure_msg(content: &str, expected: &str) {
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

/// Asserts that a given input (content) succeeds.
fn assert_success(content: &str) {
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

    assert_eq!(ret, 0);
}

/// Tests validate_filter_invalid_valid(): 42/1 is a valid filter item.
#[test]
fn test_validate_filter_invalid_valid() {
    let content = r#"filters:
  'Budaörsi út':
    valid: ['42/1']
"#;
    assert_success(content);
}

/// Tests validate_filter_invalid_valid(): 1c is a valid filter item.
#[test]
fn test_validate_filter_invalid_valid2() {
    let content = r#"filters:
  'Budaörsi út':
    valid: ['1c']
"#;
    assert_success(content);
}

/// Tests validate_filter_invalid_valid(): 40-60 is a valid filter item.
#[test]
fn test_validate_filter_invalid_valid3() {
    let content = r#"filters:
  'mystreet':
    invalid: ['40-60']
"#;
    assert_success(content);
}

/// Tests validate_filter_invalid_valid(): 50a-b is a valid filter item.
#[test]
fn test_validate_filter_invalid_valid4() {
    let content = r#"filters:
  'mystreet':
    invalid: ['50a-b']
"#;
    assert_success(content);
}

/// Tests the relation path: bad source type.
#[test]
fn test_relation_source_bad_type() {
    let content = "source: 42\n";
    let expected = "expected value type for 'source' is str\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad tab indent.
#[test]
fn test_relation_tab() {
    let content = "source:\tsurvey\n";
    let expected = "expected indent with 2 spaces, not with tabs\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad strfilters value type.
#[test]
fn test_relation_strfilters_bad_type() {
    let content = r#"street-filters:
  - 42
"#;
    let expected = "expected value type for 'street-filters[0]' is str\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad refstreets value, osm=ref.
#[test]
fn test_relation_refstreets_bad_value() {
    let content = r#"refstreets:
  'OSM Name 1': 'OSM Name 1'
"#;
    let expected = r#"expected value != key for 'refstreets.OSM Name 1'
failed to validate {0}
"#;
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad filters -> ... -> invalid subkey.
#[test]
fn test_relation_filters_invalid_bad2() {
    let content = r#"filters:
  'Budaörsi út':
    invalid: ['1c 1']
"#;
    let expected = "expected format for 'filters.Budaörsi út.invalid[0]' is '42', '42a' or '42/1'\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
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
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad filters -> ... -> ranges -> start type.
#[test]
fn test_relation_filters_ranges_bad_start() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: 42, end: '137'}
"#;
    let expected = "expected start % 2 == end % 2 for 'filters.Budaörsi út.ranges[0]'\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests the relation path: missing filters -> ... -> ranges -> start key.
#[test]
fn test_relation_filters_ranges_missing_start() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {end: '137'}
"#;
    let expected = r#"failed to validate {0}

Caused by:
    filters.Budaörsi út.ranges[0]: missing field `start` at line 4 column 9
"#;
    assert_failure_msg(content, expected);
}

/// Tests the relation path: missing filters -> ... -> ranges -> end key.
#[test]
fn test_relation_filters_ranges_missing_end() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '137'}
"#;
    let expected = r#"failed to validate {0}

Caused by:
    filters.Budaörsi út.ranges[0]: missing field `end` at line 4 column 9
"#;
    assert_failure_msg(content, expected);
}

/// Tests the housenumber-letters key: bad type.
#[test]
fn test_relation_housenumber_letters_bad() {
    let content = "housenumber-letters: 42\n";
    let expected = r#"failed to validate {0}

Caused by:
    housenumber-letters: invalid type: integer `42`, expected a boolean at line 1 column 22
"#;
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad alias subkey.
#[test]
fn test_relation_alias_bad() {
    let content = "alias: [1]\n";
    let expected = "expected value type for 'alias[0]' is str\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad type for the alias subkey.
#[test]
fn test_relation_filters_alias_bad_type() {
    let content = r#"alias: "hello"
"#;
    let expected = "failed to validate {0}\n\nCaused by:\n    alias: invalid type: string \"hello\", expected a sequence at line 1 column 8\n";
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad refstreets map, not 1:1.
#[test]
fn test_relation_refstreets_bad_map_type() {
    let content = r#"refstreets:
  'OSM Name 1': "Ref Name 1"
  # maps to the same ref name
  'OSM Name 2': "Ref Name 1"
"#;
    let expected =
        "osm and ref streets are not a 1:1 mapping in 'refstreets'\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests the relation path: bad filters -> ... -> valid subkey.
#[test]
fn test_relation_filters_valid_bad2() {
    let content = r#"filters:
  'Budaörsi út':
    valid: ['1c 1']
"#;
    let expected = "expected format for 'filters.Budaörsi út.valid[0]' is '42', '42a' or '42/1'\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests that we do not accept whitespace in the value of the 'start' key.
#[test]
fn test_start_whitespace() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '137 ', end: '165'}
"#;
    let expected = "expected value type for 'filters.Budaörsi út.ranges[0].start' is a digit str\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests that we do not accept whitespace in the value of the 'end' key.
#[test]
fn test_end_whitespace() {
    let content = r#"filters:
  'Budaörsi út':
    ranges:
      - {start: '137', end: '165 '}
"#;
    let expected = "expected value type for 'filters.Budaörsi út.ranges[0].end' is a digit str\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}

/// Tests that we do not accept keys with null values.
#[test]
fn test_null_value() {
    let content = r#"filters:
  'Budaörsi út':
"#;
    let expected =
        "expected at least one sub-key for 'filters.Budaörsi út'\nfailed to validate {0}\n";
    assert_failure_msg(content, expected);
}
