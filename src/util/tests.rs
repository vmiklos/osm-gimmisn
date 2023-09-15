/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the util module.

use super::*;
use std::io::Seek;
use std::io::Write;
use std::rc::Rc;

/// Convers a string list into a street list.
fn street_list(streets: &[&str]) -> Vec<Street> {
    streets.iter().map(|i| Street::from_string(i)).collect()
}

/// Tests get_only_in_first().
#[test]
fn test_only_in_first() {
    let ret = get_only_in_first(&street_list(&["1", "2", "3"]), &street_list(&["3", "4"]));
    let names: Vec<_> = ret.iter().map(|i| i.get_osm_name()).collect();
    assert_eq!(names, vec!["1", "2"]);
}

/// Tests get_in_both().
#[test]
fn test_get_in_both() {
    let ret = get_in_both(
        &street_list(&["1", "2", "3"]),
        &street_list(&["2", "3", "4"]),
    );
    let names: Vec<_> = ret.iter().map(|i| i.get_osm_name()).collect();
    assert_eq!(names, vec!["2", "3"]);
}

/// Converts a string list into a house number range list.
fn hnr_list(ranges: Vec<&str>) -> Vec<HouseNumberRange> {
    ranges
        .iter()
        .map(|i| HouseNumberRange::new(i, ""))
        .collect()
}

/// Tests format_even_odd().
#[test]
fn test_format_even_odd() {
    let expected = vec!["1".to_string(), "2".to_string()];
    assert_eq!(format_even_odd(&hnr_list(vec!["1", "2"])), expected);
}

/// Tests format_even_odd(): when we have odd numbers only.
#[test]
fn test_format_even_odd_only_odd() {
    let expected = vec!["1, 3".to_string()];
    assert_eq!(format_even_odd(&hnr_list(vec!["1", "3"])), expected);
}

/// Tests format_even_odd(): when we have even numbers only.
#[test]
fn test_format_even_odd_only_even() {
    let expected = vec!["2, 4".to_string()];
    assert_eq!(format_even_odd(&hnr_list(vec!["2", "4"])), expected);
}

/// Tests format_even_odd(): HTML coloring.
#[test]
fn test_format_even_odd_html() {
    let doc = format_even_odd_html(&hnr_list(vec!["2*", "4"]));
    let expected = r#"<span style="color: blue;">2</span>, 4"#;
    assert_eq!(doc.get_value(), expected)
}

/// Tests format_even_odd(): HTML commenting.
#[test]
fn test_format_even_odd_html_comment() {
    let house_numbers = vec![
        HouseNumberRange::new("2*", "foo"),
        HouseNumberRange::new("4", ""),
    ];
    let doc = format_even_odd_html(&house_numbers);
    let expected =
        r#"<span style="color: blue;"><abbr title="foo" tabindex="0">2</abbr></span>, 4"#;
    assert_eq!(doc.get_value(), expected);
}

/// Tests format_even_odd(): HTML output with multiple odd numbers.
#[test]
fn test_format_even_odd_html_multi_odd() {
    let doc = format_even_odd_html(&hnr_list(vec!["1", "3"]));
    assert_eq!(doc.get_value(), "1, 3".to_string());
}

/// Tests build_reference_index().
#[test]
fn test_build_reference_index() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut conn = ctx.get_database_connection().unwrap();
    conn.execute("delete from ref_housenumbers", []).unwrap();
    let refpath = ctx.get_abspath("workdir/refs/hazszamok_20190511.tsv");
    build_reference_index(&ctx, &mut conn, &[refpath.clone()]).unwrap();
    {
        let mut stmt = conn
            .prepare("select count(*) from ref_housenumbers")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let count: i64 = row.get(0).unwrap();
            // Empty table, so changes from 0 to 14.
            assert_eq!(count, 14);
        }
    }

    build_reference_index(&ctx, &mut conn, &[refpath]).unwrap();
    {
        let mut stmt = conn
            .prepare("select count(*) from ref_housenumbers")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let count: i64 = row.get(0).unwrap();
            // Early return, so doesn't change from 14 to 28.
            assert_eq!(count, 14);
        }
    }
}

/// Tests build_street_reference_index().
#[test]
fn test_build_street_reference_index() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut conn = ctx.get_database_connection().unwrap();
    conn.execute("delete from ref_streets", []).unwrap();
    let refpath = ctx.get_abspath("workdir/refs/utcak_20190514.tsv");
    build_street_reference_index(&ctx, &mut conn, &refpath).unwrap();
    {
        let mut stmt = conn.prepare("select count(*) from ref_streets").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let count: i64 = row.get(0).unwrap();
            // Empty table, so changes from 0 to 6.
            assert_eq!(count, 6);
        }
    }

    build_street_reference_index(&ctx, &mut conn, &refpath).unwrap();
    {
        let mut stmt = conn.prepare("select count(*) from ref_streets").unwrap();
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            let count: i64 = row.get(0).unwrap();
            // Early return, so doesn't change from 6 to 12.
            assert_eq!(count, 6);
        }
    }
}

/// Tests split_house_number(): just numbers.
#[test]
fn test_split_house_number_only_number() {
    assert_eq!(split_house_number("42"), (42, "".to_string()));
}

/// Tests split_house_number(): numbers and suffixes.
#[test]
fn test_split_house_number_number_alpha() {
    assert_eq!(split_house_number("42ab"), (42, "ab".to_string()));
}

/// Tests split_house_number(): just suffixes.
#[test]
fn test_split_house_number_only_alpha() {
    assert_eq!(split_house_number("a"), (0, "a".to_string()));
    assert_eq!(split_house_number(""), (0, "".to_string()));
}

/// Tests parse_filters(): the incomplete case.
#[test]
fn test_parse_filters_incomplete() {
    let from = &[
        "osm".to_string(),
        "filter-for".to_string(),
        "incomplete".to_string(),
    ];
    assert_eq!(parse_filters(from).contains_key("incomplete"), true)
}

/// Tests parse_filters(): the refcounty case.
#[test]
fn test_parse_filters_refcounty() {
    let from = &[
        "osm".to_string(),
        "filter-for".to_string(),
        "refcounty".to_string(),
        "42".to_string(),
    ];
    let mut expected: HashMap<String, String> = HashMap::new();
    expected.insert("refcounty".into(), "42".into());
    assert_eq!(parse_filters(from), expected);
}

/// Tests parse_filters(): the refsettlement case.
#[test]
fn test_parse_filters_refsettlement() {
    let from = &[
        "osm".to_string(),
        "filter-for".to_string(),
        "refcounty".to_string(),
        "42".to_string(),
        "refsettlement".to_string(),
        "43".to_string(),
    ];
    let filters = parse_filters(from);
    assert_eq!(filters["refcounty"], "42");
    assert_eq!(filters["refsettlement"], "43");
}

/// Tests handle_overpass_error(): the case when no sleep is needed.
#[test]
fn test_handle_overpass_error_no_sleep() {
    let error = "HTTP Error 404: no such file";
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let doc = handle_overpass_error(&ctx, error);
    let expected = r#"<div id="overpass-error">Overpass error: HTTP Error 404: no such file</div>"#;
    assert_eq!(doc.get_value(), expected);
}

/// Tests handle_overpass_error(): the case when sleep is needed.
#[test]
fn test_handle_overpass_error_need_sleep() {
    let error = "HTTP Error 404: no such file";
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-wait.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let doc = handle_overpass_error(&ctx, error);
    let expected = r#"<div id="overpass-error">Overpass error: HTTP Error 404: no such file<br />Note: wait for 12 seconds</div>"#;
    assert_eq!(doc.get_value(), expected);
}

/// Tests setup_localization().
#[test]
fn test_setup_localization() {
    let ctx = context::tests::make_test_context().unwrap();
    let request = rouille::Request::fake_http(
        "GET",
        "/",
        vec![(
            "Accept-Language".to_string(),
            "hu,en;q=0.9,en-US;q=0.8".to_string(),
        )],
        Vec::new(),
    );
    i18n::set_language(&ctx, "en");
    setup_localization(&ctx, request.headers());
    assert_eq!(i18n::get_language(), "hu");
    i18n::set_language(&ctx, "en");
}

/// Tests setup_localization(): the error path.
#[test]
fn test_setup_localization_parse_error() {
    let ctx = context::tests::make_test_context().unwrap();
    let request = rouille::Request::fake_http(
        "GET",
        "/",
        vec![("Accept-Language".to_string(), ",".to_string())],
        Vec::new(),
    );
    i18n::set_language(&ctx, "en");
    setup_localization(&ctx, request.headers());
    assert_eq!(i18n::get_language(), "en");
}

/// Tests gen_link().
#[test]
fn test_gen_link() {
    let doc = gen_link("http://www.example.com", "label");
    let expected = r#"<a href="http://www.example.com">label...</a>"#;
    assert_eq!(doc.get_value(), expected);
}

/// Tests process_template().
#[test]
fn test_process_template() {
    let template = "aaa @RELATION@ bbb @AREA@ ccc";
    let expected = "aaa 42 bbb 3600000042 ccc";
    let actual = process_template(template, 42);
    assert_eq!(actual, expected);
}

/// Tests html_table_from_list().
#[test]
fn test_html_table_from_list() {
    let fro = vec![
        vec![yattag::Doc::from_text("A1"), yattag::Doc::from_text("B1")],
        vec![yattag::Doc::from_text("A2"), yattag::Doc::from_text("B2")],
    ];
    let expected = "<table class=\"sortable\">\
<tr><th><a href=\"#\">A1</a></th>\
<th><a href=\"#\">B1</a></th></tr>\
<tr><td>A2</td><td>B2</td></tr></table>";
    let ret = html_table_from_list(&fro).get_value();
    assert_eq!(ret, expected);
}

/// Tests tsv_to_list().
#[test]
fn test_tsv_to_list() {
    let mut cursor = std::io::Cursor::new(b"h1\th2\n\nv1\tv2\n");
    let ret = tsv_to_list(&mut cursor).unwrap();
    assert_eq!(ret.len(), 2);
    let row1: Vec<_> = ret[0].iter().map(|cell| cell.get_value()).collect();
    assert_eq!(row1, vec!["h1", "h2"]);
    let row2: Vec<_> = ret[1].iter().map(|cell| cell.get_value()).collect();
    assert_eq!(row2, vec!["v1", "v2"]);
}

/// Tests tsv_to_list(): when a @type column is available.
#[test]
fn test_tsv_to_list_type() {
    let mut cursor = std::io::Cursor::new(b"@id\t@type\n42\tnode\n");
    let ret = tsv_to_list(&mut cursor).unwrap();
    assert_eq!(ret.len(), 2);
    let row1: Vec<_> = ret[0].iter().map(|cell| cell.get_value()).collect();
    assert_eq!(row1, vec!["@id", "@type"]);
    let row2: Vec<_> = ret[1].iter().map(|cell| cell.get_value()).collect();
    let cell_a2 = r#"<a href="https://www.openstreetmap.org/node/42" target="_blank">42</a>"#;
    assert_eq!(row2, vec![cell_a2, "node"]);
}

/// Tests tsv_to_list(): escaping.
#[test]
fn test_tsv_to_list_escape() {
    let mut cursor = std::io::Cursor::new(b"\"h,1\"\th2\n");
    let ret = tsv_to_list(&mut cursor).unwrap();
    assert_eq!(ret.len(), 1);
    let row1: Vec<_> = ret[0].iter().map(|cell| cell.get_value()).collect();
    // Note how this is just h,1 and not "h,1".
    assert_eq!(row1, vec!["h,1", "h2"]);
}

/// Tests tsv_to_list(): sorting.
#[test]
fn test_tsv_to_list_sort() {
    let mut cursor = std::io::Cursor::new(
        b"addr:street\taddr:housenumber\n\
A street\t1\n\
A street\t10\n\
A street\t9",
    );
    let ret = tsv_to_list(&mut cursor).unwrap();
    // 0th is header
    let row3: Vec<_> = ret[3].iter().map(|cell| cell.get_value()).collect();
    // Note how 10 is ordered after 9.
    assert_eq!(row3[1], "10");
}

/// Tests the HouseNumber class.
#[test]
fn test_house_number() {
    let house_number = HouseNumber::new("1", "1-2", "");
    assert_eq!(house_number.get_number(), "1");
    assert_eq!(house_number.get_source(), "1-2");
    assert_eq!(
        HouseNumber::new("1", "1-2", "") != HouseNumber::new("2", "1-2", ""),
        true
    );
    let mut house_numbers = vec![
        HouseNumber::new("1", "1-2", ""),
        HouseNumber::new("2", "1-2", ""),
        HouseNumber::new("2", "1-2", ""),
    ];
    house_numbers.sort_unstable();
    house_numbers.dedup();
    assert_eq!(house_numbers.len(), 2);
}

/// Tests HouseNumber::is_invalid().
#[test]
fn test_house_number_is_invalid() {
    let mut used_invalids: Vec<String> = Vec::new();
    assert_eq!(
        HouseNumber::is_invalid("15 a", &["15a".to_string()], &mut used_invalids),
        true
    );
    assert_eq!(
        HouseNumber::is_invalid("15/a", &["15a".to_string()], &mut used_invalids),
        true
    );
    assert_eq!(
        HouseNumber::is_invalid("15A", &["15a".to_string()], &mut used_invalids),
        true
    );
    assert_eq!(
        HouseNumber::is_invalid("67/5*", &["67/5".to_string()], &mut used_invalids),
        true
    );

    // Make sure we don't panic on input which does not start with a number.
    assert_eq!(
        HouseNumber::is_invalid("A", &["15a".to_string()], &mut used_invalids),
        false
    );
}

/// Tests HouseNumber::has_letter_suffix().
#[test]
fn test_house_number_letter_suffix() {
    assert_eq!(HouseNumber::has_letter_suffix("42a", ""), true);
    assert_eq!(HouseNumber::has_letter_suffix("42 a", ""), true);
    assert_eq!(HouseNumber::has_letter_suffix("42/a", ""), true);
    assert_eq!(HouseNumber::has_letter_suffix("42/a*", "*"), true);
    assert_eq!(HouseNumber::has_letter_suffix("42A", ""), true);
    assert_eq!(HouseNumber::has_letter_suffix("42 AB", ""), false);
}

/// Tests HouseNumber::normalize_letter_suffix().
#[test]
fn test_house_number_normalize_letter_suffix() {
    assert_eq!(
        HouseNumber::normalize_letter_suffix("42a", "").unwrap(),
        "42/A"
    );
    assert_eq!(
        HouseNumber::normalize_letter_suffix("42 a", "").unwrap(),
        "42/A"
    );
    assert_eq!(
        HouseNumber::normalize_letter_suffix("42/a", "").unwrap(),
        "42/A"
    );
    assert_eq!(
        HouseNumber::normalize_letter_suffix("42/A", "").unwrap(),
        "42/A"
    );
    assert_eq!(
        HouseNumber::normalize_letter_suffix("42/A*", "*").unwrap(),
        "42/A*"
    );
    assert_eq!(
        HouseNumber::normalize_letter_suffix("42 A", "").unwrap(),
        "42/A"
    );
    assert_eq!(HouseNumber::normalize_letter_suffix("x", "").is_err(), true);
}

/// Tests HouseNumberRange::get_lowercase_number().
#[test]
fn test_house_number_range_get_lowercase_number() {
    let range = HouseNumberRange::new("42/A", "");
    assert_eq!(range.get_lowercase_number(), "42a");
    let range = HouseNumberRange::new("43b", "");
    assert_eq!(range.get_lowercase_number(), "43b");
    let range = HouseNumberRange::new("44/C*", "");
    assert_eq!(range.get_lowercase_number(), "44c*");
}

/// Tests get_housenumber_ranges().
#[test]
fn test_get_housenumber_ranges() {
    let house_numbers = [
        HouseNumber::new("25", "25", ""),
        HouseNumber::new("27", "27-37", ""),
        HouseNumber::new("29", "27-37", ""),
        HouseNumber::new("31", "27-37", ""),
        HouseNumber::new("33", "27-37", ""),
        HouseNumber::new("35", "27-37", ""),
        HouseNumber::new("37", "27-37", ""),
        HouseNumber::new("31*", "31*", ""),
    ];
    let ranges = get_housenumber_ranges(&house_numbers);
    let range_names: Vec<_> = ranges.iter().map(|i| i.get_number()).collect();
    assert_eq!(range_names, ["25", "27-37", "31*"]);
}

/// Tests git_link().
#[test]
fn test_git_link() {
    let actual = git_link("v1-151-g64ecc85", "http://www.example.com/").get_value();
    let expected = "<a href=\"http://www.example.com/64ecc85\">v1-151-g64ecc85</a>";
    assert_eq!(actual, expected);
}

/// Tests sort_numerically(): numbers.
#[test]
fn test_sort_numerically_numbers() {
    let ascending = sort_numerically(&[
        HouseNumber::new("1", "", ""),
        HouseNumber::new("20", "", ""),
        HouseNumber::new("3", "", ""),
    ]);
    let actual: Vec<_> = ascending.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["1", "3", "20"]);
}

/// Tests sort_numerically(): numbers with suffixes.
#[test]
fn test_sort_numerically_alpha_suffix() {
    let ascending = sort_numerically(&[
        HouseNumber::new("1a", "", ""),
        HouseNumber::new("20a", "", ""),
        HouseNumber::new("3a", "", ""),
    ]);
    let actual: Vec<_> = ascending.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["1a", "3a", "20a"]);
}

/// Tests sort_numerically(): just suffixes.
#[test]
fn test_sort_numerically_alpha() {
    let ascending = sort_numerically(&[
        HouseNumber::new("a", "", ""),
        HouseNumber::new("c", "", ""),
        HouseNumber::new("b", "", ""),
    ]);
    let actual: Vec<_> = ascending.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["a", "b", "c"]);
}

/// Tests Street.
#[test]
fn test_street() {
    let street = Street::new(
        "foo", "bar", /*show_ref_street=*/ true, /*osm_id=*/ 0,
    );
    assert_eq!(street.to_html().get_value(), "foo<br />(bar)");
}

/// Tests get_city_key().
#[test]
fn test_get_city_key() {
    let mut valid_settlements: HashSet<String> = HashSet::new();
    valid_settlements.insert("lábatlan".into());
    assert_eq!(
        get_city_key("1234", "Budapest", &valid_settlements).unwrap(),
        "budapest_23"
    );
    assert_eq!(
        get_city_key("1889", "Budapest", &valid_settlements).unwrap(),
        "budapest"
    );
    assert_eq!(
        get_city_key("9999", "", &valid_settlements).unwrap(),
        "_Empty"
    );
    assert_eq!(
        get_city_key("9999", "Lábatlan", &valid_settlements).unwrap(),
        "lábatlan"
    );
    assert_eq!(
        get_city_key("9999", "junk", &valid_settlements).unwrap(),
        "_Invalid"
    );
    // Even if the pos does not start with 1.
    assert_eq!(
        get_city_key("9999", "Budapest", &valid_settlements).unwrap(),
        "budapest"
    );
    // postcode vs housenumber swap.
    assert_eq!(
        get_city_key("1/A", "junk", &valid_settlements).unwrap(),
        "_Invalid"
    );
}

/// Tests get_street_from_housenumber(): the case when addr:place is used.
#[test]
fn test_get_street_from_housenumber_addr_place() {
    let mut read = std::fs::File::open("tests/workdir/street-housenumbers-gh964.csv").unwrap();
    let mut csv_reader = make_csv_reader(&mut read);
    let actual = get_street_from_housenumber(&mut csv_reader).unwrap();
    // This is picked up from addr:place because addr:street was empty.
    assert_eq!(actual, [Street::from_string("Tolvajos tanya")]);
}

/// Tests get_street_from_housenumber(): the case when the addr:housenumber column is missing.
#[test]
fn test_get_street_from_housenumber_missing_column() {
    let mut cursor = std::io::Cursor::new(Vec::new());
    cursor.write_all(b"@id\n42\n").unwrap();
    cursor.rewind().unwrap();
    let mut csv_reader = make_csv_reader(&mut cursor);
    assert_eq!(get_street_from_housenumber(&mut csv_reader).is_err(), true);
}

/// Tests invalid_filter_keys_to_html().
#[test]
fn test_invalid_filter_keys_to_html() {
    let ret = invalid_filter_keys_to_html(&["foo".into()]);
    assert_eq!(ret.get_value().contains("<li>"), true);
}

/// Tests invalid_filter_keys_to_html(): when the arg is empty.
#[test]
fn test_invalid_filter_keys_to_html_empty() {
    let ret = invalid_filter_keys_to_html(&[]);
    assert_eq!(ret.get_value(), "");
}

/// Tests get_column().
#[test]
fn test_get_column() {
    // id, street name, housenumber
    let row = [
        yattag::Doc::from_text("42"),
        yattag::Doc::from_text("A street"),
        yattag::Doc::from_text("1"),
    ];
    assert_eq!(get_column(&row, 1), "A street");
    assert_eq!(natnum(&get_column(&row, 2)), 1);
    // Too large column index -> first column.
    assert_eq!(get_column(&row, 3), "42");
}

/// Tests get_column(): the 'housenumber is junk' case.
#[test]
fn test_get_column_junk() {
    // id, street name, housenumber
    let row = [
        yattag::Doc::from_text("42"),
        yattag::Doc::from_text("A street"),
        yattag::Doc::from_text("fixme"),
    ];
    assert_eq!(natnum(&get_column(&row, 2)), 0);
}

/// Tests get_mtime(): what happens when the file is not there.
#[test]
fn test_get_mtime_no_such_file() {
    let ctx = context::tests::make_test_context().unwrap();
    assert_eq!(get_mtime(&ctx, ""), time::OffsetDateTime::UNIX_EPOCH);
}

/// Tests get_lexical_sort_key().
#[test]
fn test_get_lexical_sort_key() {
    // This is less naive than the classic "a, "á", "b", "c" list.
    let mut strings = vec!["Kőpor", "Kórház"];
    strings.sort_by_key(|i| get_sort_key(i));
    assert_eq!(strings, ["Kórház", "Kőpor"]);
}

/// Tests split_house_number_by_separator().
#[test]
fn test_split_house_number_by_separator() {
    let normalizer = ranges::Ranges::new(vec![ranges::Range::new(2, 4, "")]);

    let relation_name = "";
    let street_name = "";
    let mut lints = None;
    let ret = split_house_number_by_separator(
        "2-6",
        "-",
        &normalizer,
        relation_name,
        street_name,
        &mut lints,
        None,
    );

    assert_eq!(ret.0, vec![2]);
    assert_eq!(ret.1, vec![2, 6]);
}

/// Tests get_valid_settlements().
#[test]
fn test_get_valid_settlements() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let citycounts_path = "workdir/refs/varosok_count_20190717.tsv";
    let citycounts = context::tests::TestFileSystem::make_file();
    citycounts
        .borrow_mut()
        .write_all(b"CITY\tCNT\nmycity1\t1\nmycity2\t2\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[(citycounts_path, &citycounts)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = get_valid_settlements(&ctx).unwrap();

    let mut expected: HashSet<String> = HashSet::new();
    expected.insert("mycity1".to_string());
    expected.insert("mycity2".to_string());
    assert_eq!(ret, expected);
}

/// Tests get_valid_settlements(): ignore broken lines.
#[test]
fn test_get_valid_settlements_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let citycounts_path = "workdir/refs/varosok_count_20190717.tsv";
    let citycounts = context::tests::TestFileSystem::make_file();
    citycounts
        .borrow_mut()
        .write_all(b"CITY\tCNT\nmycity1\t1\nmycity2\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[(citycounts_path, &citycounts)]);
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = get_valid_settlements(&ctx).unwrap();

    let mut expected: HashSet<String> = HashSet::new();
    expected.insert("mycity1".to_string());
    assert_eq!(ret, expected);
}

/// Tests that HouseNumberRange implements the Debug trait.
#[test]
fn test_house_number_range_debug() {
    let range = HouseNumberRange::new("1", "");

    let ret = format!("{range:?}");

    assert_eq!(ret.starts_with("HouseNumberRange"), true);
}

/// Tests that Street implements the Debug trait.
#[test]
fn test_street_debug() {
    let street = Street::from_string("mystreet");

    let ret = format!("{street:?}");

    assert_eq!(ret.starts_with("Street"), true);
}

/// Tests that HouseNumber implements the Debug trait.
#[test]
fn test_house_number_debug() {
    let street = HouseNumber::new("1", "1-3", "");

    let ret = format!("{street:?}");

    assert_eq!(ret.starts_with("HouseNumber"), true);
}

/// Tests that NumberedStreet implements the Debug trait.
#[test]
fn test_numbered_street_debug() {
    let street = Street::from_string("mystreet");
    let house_numbers = Vec::new();
    let numbered_street = NumberedStreet {
        street,
        house_numbers,
    };

    let ret = format!("{numbered_street:?}");

    assert_eq!(ret.starts_with("NumberedStreet"), true);
}
