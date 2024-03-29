/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the stats module.

use super::*;
use std::rc::Rc;

use crate::context::FileSystem as _;

fn make_test_time_old() -> context::tests::TestTime {
    context::tests::TestTime::new(1970, 1, 1)
}

/// Tests handle_progress().
#[test]
fn test_handle_progress() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-10", "254651"],
        )
        .unwrap();
    }
    let ref_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/ref.count", &ref_count)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string("300", &ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap();
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_progress(&ctx, &src_root, &mut j).unwrap();

    let progress = &j.as_object().unwrap()["progress"];
    assert_eq!(progress["date"], "2020-05-10");
    // 254651 / 300 * 100
    assert_eq!(progress["percentage"], 84883.67);
}

/// Tests handle_capital_progress().
#[test]
fn test_handle_capital_progress() {
    let mut ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_citycounts (date, city, count) values (?1, ?2, ?3)"#,
            ["2020-05-10", "Budapest_02", "200"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_citycounts (date, city, count) values (?1, ?2, ?3)"#,
            ["2020-05-10", "Budapest_11", "11"],
        )
        .unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let ref_city_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/refs/varosok_count_20190717.tsv", &ref_city_count)],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "CITY\tCNT\nbudapest_11\t100\nbudapest_12\t200\nmycity\t42\n",
            &ctx.get_abspath("workdir/refs/varosok_count_20190717.tsv"),
        )
        .unwrap();
    let file_system: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system);
    let mut j = serde_json::json!({});

    handle_capital_progress(&ctx, &mut j).unwrap();

    let progress = &j.as_object().unwrap()["capital-progress"];
    assert_eq!(progress["date"], "2020-05-10");
    // 211 / 300 * 100
    // Note that the capital sum is 300, the total sum is 342.
    assert_eq!(progress["percentage"], 70.33);
}

/// Tests handle_progress(): the case when the .count file doesn't exist for a date.
#[test]
fn test_handle_progress_old_time() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = make_test_time_old();
    let time_rc: Rc<dyn context::Time> = Rc::new(time);
    ctx.set_time(&time_rc);
    let ref_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/ref.count", &ref_count)],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string("42", &ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap();
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);

    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_progress(&ctx, &src_root, &mut j).unwrap();

    let progress = &j.as_object().unwrap()["progress"];
    assert_eq!(progress["date"], "1970-01-01");
}

/// Tests handle_topusers().
#[test]
fn test_handle_topusers() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        for i in 0..20 {
            conn.execute(
                r#"insert into stats_topusers (date, user, count) values (?1, ?2, ?3)"#,
                [
                    "2020-05-10",
                    &format!("user{}", i + 1),
                    &format!("{}", 20 - i),
                ],
            )
            .unwrap();
        }
    }
    let mut j = serde_json::json!({});
    handle_topusers(&ctx, &mut j).unwrap();
    let topusers = &j.as_object().unwrap()["topusers"].as_array().unwrap();
    assert_eq!(topusers.len(), 20);
    assert_eq!(topusers[0], serde_json::json!(["user1", 20]));
}

/// Tests handle_topusers(): the case when the stats_topusers table has no rows for a date.
#[test]
fn test_handle_topusers_old_time() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = make_test_time_old();
    let time_rc: Rc<dyn context::Time> = Rc::new(time);
    ctx.set_time(&time_rc);
    let mut j = serde_json::json!({});
    handle_topusers(&ctx, &mut j).unwrap();
    let topusers = &j.as_object().unwrap()["topusers"].as_array().unwrap();
    assert_eq!(topusers.len(), 0);
}

/// Tests handle_topcities().
#[test]
fn test_handle_topcities() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into stats_citycounts (date, city, count) values ('2020-04-10', 'budapest_01', '10');
             insert into stats_citycounts (date, city, count) values ('2020-04-10', 'budapest_02', '10');
             insert into stats_citycounts (date, city, count) values ('2020-05-10', 'budapest_01', '100');
             insert into stats_citycounts (date, city, count) values ('2020-05-10', 'budapest_02', '200');
             insert into stats_citycounts (date, city, count) values ('2020-05-10', 'budapest_03', '300');"
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_topcities(&ctx, &mut j).unwrap();
    let topcities = &j.as_object().unwrap()["topcities"].as_array().unwrap();
    assert_eq!(topcities.len(), 2);
    assert_eq!(topcities[0], serde_json::json!(["budapest_02", 190]));
    assert_eq!(topcities[1], serde_json::json!(["budapest_01", 90]));
}

/// Tests handle_daily_new().
#[test]
fn test_hanle_daily_new() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-04-26", "251250"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-04-27", "251614"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    // From now on, today is 2020-05-10, so this will read 2020-04-26, 2020-04-27, etc
    // (till a file is missing.)
    handle_daily_new(&ctx, &mut j, /*day_range=*/ 14).unwrap();
    let daily = &j.as_object().unwrap()["daily"].as_array().unwrap();
    assert_eq!(daily.len(), 1);
    assert_eq!(daily[0], serde_json::json!(["2020-04-26", 364]));
}

/// Tests handle_invalid_addr_cities().
#[test]
fn test_handle_invalid_addr_cities() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_invalid_addr_cities_counts (date, count) values (?1, ?2)"#,
            ["2020-04-26", "1"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_invalid_addr_cities_counts (date, count) values (?1, ?2)"#,
            ["2020-04-27", "2"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});

    handle_invalid_addr_cities(&ctx, &mut j, /*day_range=*/ 1).unwrap();

    let invalids = &j.as_object().unwrap()["invalidAddrCities"]
        .as_array()
        .unwrap();
    assert_eq!(invalids.len(), 1);
    assert_eq!(invalids[0], serde_json::json!(["2020-04-27", 2]));
}

/// Tests handle_daily_new(): the case when the day range is empty.
#[test]
fn test_handle_daily_new_empty_day_range() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut j = serde_json::json!({});
    handle_daily_new(&ctx, &mut j, /*day_range=*/ -1).unwrap();
    let daily = &j.as_object().unwrap()["daily"].as_array().unwrap();
    assert_eq!(daily.len(), 0);
}

/// Tests handle_monthly_new().
#[test]
fn test_handle_monthly_new() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2019-05-01", "199518"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2019-06-01", "203317"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-01", "253027"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-10", "254651"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_monthly_new(&ctx, &mut j, /*month_range=*/ 12).unwrap();
    let monthly = &j.as_object().unwrap()["monthly"].as_array().unwrap();
    assert_eq!(monthly.len(), 2);
    // 2019-05 start -> end
    assert_eq!(monthly[0], serde_json::json!(["2019-05", 3799]));
    // diff from last month end -> today
    assert_eq!(monthly[1], serde_json::json!(["2020-05", 51334]));
}

/// Tests handle_monthly_new(): the case when the month range is empty.
#[test]
fn test_handle_monthly_new_empty_month_range() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-01", "253027"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-10", "254651"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_monthly_new(&ctx, &mut j, /*month_range=*/ -1).unwrap();
    let monthly = &j.as_object().unwrap()["monthly"].as_array().unwrap();
    assert_eq!(monthly.is_empty(), false);
}

/// Tests handle_monthly_new(): the case when we have no data for the last, incomplete month.
#[test]
fn test_handle_monthly_new_incomplete_last_month() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2019-05-01", "199518"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2019-06-01", "203317"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-01", "253027"],
        )
        .unwrap();
        // 2020-05-10 would be the data for the current state of the last, incomplete month.
    }
    let mut j = serde_json::json!({});

    handle_monthly_new(&ctx, &mut j, /*month_range=*/ 12).unwrap();
    let monthly = &j.as_object().unwrap()["monthly"].as_array().unwrap();
    // 1st element: 2019-05 start -> end
    // No 2nd element, would be diff from last month end -> today
    assert_eq!(monthly.len(), 1);
    assert_eq!(monthly[0], serde_json::json!(["2019-05", 3799]));
}

/// Tests handle_daily_total().
#[test]
fn test_handle_daily_total() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-04-27", "251614"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_daily_total(&ctx, &mut j, /*day_range=*/ 13).unwrap();
    let dailytotal = &j.as_object().unwrap()["dailytotal"].as_array().unwrap();
    assert_eq!(dailytotal.len(), 1);
    assert_eq!(dailytotal[0], serde_json::json!(["2020-04-27", 251614]));
}

/// Tests handle_daily_total(): the case when the day range is empty.
#[test]
fn test_handle_daily_total_empty_day_range() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut j = serde_json::json!({});
    handle_daily_total(&ctx, &mut j, /*day_range=*/ -1).unwrap();
    let dailytotal = &j.as_object().unwrap()["dailytotal"].as_array().unwrap();
    assert_eq!(dailytotal.is_empty(), true);
}

/// Tests handle_user_total().
#[test]
fn test_handle_user_total() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_usercounts (date, count) values (?1, ?2)"#,
            ["2020-04-27", "43"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_user_total(&ctx, &mut j, /*day_range=*/ 13).unwrap();
    let usertotal = &j.as_object().unwrap()["usertotal"].as_array().unwrap();
    assert_eq!(usertotal.len(), 1);
    assert_eq!(usertotal[0], serde_json::json!(["2020-04-27", 43]));
}

/// Tests handle_user_total(): the case when the day range is empty.
#[test]
fn test_handle_user_total_empty_day_range() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut j = serde_json::json!({});
    handle_user_total(&ctx, &mut j, /*day_range=*/ -1).unwrap();
    let usertotal = &j.as_object().unwrap()["usertotal"].as_array().unwrap();
    assert_eq!(usertotal.is_empty(), true);
}

/// Tests handle_monthly_total().
#[test]
fn test_handle_monthly_total() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2019-06-01", "203317"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_monthly_total(&ctx, &mut j, /*month_range=*/ 11).unwrap();
    let monthlytotal = &j.as_object().unwrap()["monthlytotal"].as_array().unwrap();
    assert_eq!(monthlytotal.len(), 1);
    assert_eq!(monthlytotal[0], serde_json::json!(["2019-05", 203317]))
}

/// Tests handle_monthly_total(): the case when the day range is empty.
#[test]
fn test_handle_monthly_total_empty_day_range() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut j = serde_json::json!({});
    handle_monthly_total(&ctx, &mut j, /*month_range=*/ -1).unwrap();
    let monthlytotal = &j.as_object().unwrap()["monthlytotal"].as_array().unwrap();
    assert_eq!(monthlytotal.is_empty(), true);
}

/// Tests handle_monthly_total(): the case when the day range is of just one element.
#[test]
fn test_handle_monthly_total_one_element_day_range() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-01", "253027"],
        )
        .unwrap();
        conn.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)"#,
            ["2020-05-10", "254651"],
        )
        .unwrap();
    }
    let mut j = serde_json::json!({});
    handle_monthly_total(&ctx, &mut j, /*month_range=*/ 0).unwrap();
    let monthlytotal = &j.as_object().unwrap()["monthlytotal"].as_array().unwrap();
    assert_eq!(monthlytotal.len(), 2);
    assert_eq!(monthlytotal[0], serde_json::json!(["2020-04", 253027]));
    assert_eq!(monthlytotal[1], serde_json::json!(["2020-05", 254651]));
}

/// Tests get_previous_month().
#[test]
fn test_get_previous_month() {
    let ctx = context::tests::make_test_context().unwrap();
    let today = ctx.get_time().now();

    let actual = get_previous_month(&today, 2).unwrap();

    let expected = time::macros::datetime!(2020-03-31 0:00:00).assume_utc();
    assert_eq!(actual, expected);
}

/// Tests update_invalid_addr_cities().
#[test]
fn test_update_invalid_addr_cities() {
    let ctx = context::tests::make_test_context().unwrap();
    {
        let conn = ctx.get_database_connection().unwrap();
        conn.execute_batch(
            "insert into whole_country (postcode, city, street, housenumber, user, osm_id, osm_type, timestamp, place, unit, name, fixme) values ('7677', 'mycity', 'mystreet', '1', 'myuser', '42', 'way', '2020-05-10T22:02:25Z', '', '', '', '');
             insert into whole_country (postcode, city, street, housenumber, user, osm_id, osm_type, timestamp, place, unit, name, fixme) values ('7677', 'Budapest', 'mystreet', '1', 'myuser1', '43', 'node', '2020-05-10T22:02:25Z', '', '', '', '');"
        )
        .unwrap();
    }

    update_invalid_addr_cities(&ctx).unwrap();

    let conn = ctx.get_database_connection().unwrap();
    let mut stmt = conn
        .prepare("select count(*) from stats_invalid_addr_cities")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        let count: i64 = row.get(0).unwrap();
        // mycity not listed in tests/workdir/refs/varosok_count_20190717.tsv as valid.
        assert_eq!(count, 1);
    }
}
