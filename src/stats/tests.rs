/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the stats module.

use super::*;
use std::io::Write;
use std::sync::Arc;

use crate::context::FileSystem as _;

fn make_test_time_old() -> context::tests::TestTime {
    context::tests::TestTime::new(1970, 1, 1)
}

/// Tests handle_progress().
#[test]
fn test_handle_progress() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

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
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let mut file_system = context::tests::TestFileSystem::new();
    let city_count = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("refdir/varosok_count_20190717.tsv", &city_count)],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "CITY\tCOUNT\nbudapest_11\t100\nbudapest_12\t200\nmycity\t42\n",
            &ctx.get_abspath("refdir/varosok_count_20190717.tsv"),
        )
        .unwrap();
    let file_system: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});

    handle_capital_progress(&ctx, &src_root, &mut j).unwrap();

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
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
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
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_progress(&ctx, &src_root, &mut j).unwrap();

    let progress = &j.as_object().unwrap()["progress"];
    assert_eq!(progress["date"], "1970-01-01");
}

/// Tests handle_capital_progress(): the case when the .count file doesn't exist for a date.
#[test]
fn test_handle_capital_progress_old_time() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = make_test_time_old();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_capital_progress(&ctx, &src_root, &mut j).unwrap();
    let progress = &j.as_object().unwrap()["capital-progress"];
    assert_eq!(progress["date"], "1970-01-01");
}

/// Tests handle_topusers().
#[test]
fn test_handle_topusers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_topusers(&ctx, &src_root, &mut j).unwrap();
    let topusers = &j.as_object().unwrap()["topusers"].as_array().unwrap();
    assert_eq!(topusers.len(), 20);
    assert_eq!(topusers[0], serde_json::json!(["user1", "68885"]));
}

/// Tests handle_topusers(): the case when the .topusers file doesn't exist for a date.
#[test]
fn test_handle_topusers_old_time() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = make_test_time_old();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_topusers(&ctx, &src_root, &mut j).unwrap();
    let topusers = &j.as_object().unwrap()["topusers"].as_array().unwrap();
    assert_eq!(topusers.len(), 0);
}

/// Tests handle_topusers(): the case when the .topusers file is broken.
#[test]
fn test_handle_topusers_broken_input() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let today_topusers = b"myuser\n";
    let today_topusers_value = context::tests::TestFileSystem::make_file();
    today_topusers_value
        .borrow_mut()
        .write_all(today_topusers)
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/2020-05-10.topusers", &today_topusers_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_topusers(&ctx, &src_root, &mut j).unwrap();
    let topusers = &j.as_object().unwrap()["topusers"].as_array().unwrap();
    assert_eq!(topusers.len(), 0);
}

/// Tests handle_topcities().
#[test]
fn test_handle_topcities() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let today_citycount = b"budapest_01\t100\n\
budapest_02\t200\n";
    let today_citycount_value = context::tests::TestFileSystem::make_file();
    today_citycount_value
        .borrow_mut()
        .write_all(today_citycount)
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/2020-05-10.citycount", &today_citycount_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut j = serde_json::json!({});
    handle_topcities(&ctx, &src_root, &mut j).unwrap();
    let topcities = &j.as_object().unwrap()["topcities"].as_array().unwrap();
    assert_eq!(topcities.len(), 2);
    assert_eq!(topcities[0], serde_json::json!(["budapest_02", 190]));
    assert_eq!(topcities[1], serde_json::json!(["budapest_01", 90]));
}

/// Tests handle_daily_new().
#[test]
fn test_hanle_daily_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    // From now on, today is 2020-05-10, so this will read 2020-04-26, 2020-04-27, etc
    // (till a file is missing.)
    handle_daily_new(&ctx, &src_root, &mut j, /*day_range=*/ 14).unwrap();
    let daily = &j.as_object().unwrap()["daily"].as_array().unwrap();
    assert_eq!(daily.len(), 1);
    assert_eq!(daily[0], serde_json::json!(["2020-04-26", 364]));
}

/// Tests handle_daily_new(): the case when the day range is empty.
#[test]
fn test_handle_daily_new_empty_day_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_daily_new(&ctx, &src_root, &mut j, /*day_range=*/ -1).unwrap();
    let daily = &j.as_object().unwrap()["daily"].as_array().unwrap();
    assert_eq!(daily.len(), 0);
}

/// Tests handle_monthly_new().
#[test]
fn test_handle_monthly_new() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_monthly_new(&ctx, &src_root, &mut j, /*month_range=*/ 12).unwrap();
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
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_monthly_new(&ctx, &src_root, &mut j, /*month_range=*/ -1).unwrap();
    let monthly = &j.as_object().unwrap()["monthly"].as_array().unwrap();
    assert_eq!(monthly.is_empty(), false);
}

/// Tests handle_monthly_new(): the case when we have no data for the last, incomplete month.
#[test]
fn test_handle_monthly_new_incomplete_last_month() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    // This would be the data for the current state of the last, incomplete month.
    let hide_path = ctx.get_abspath("workdir/stats/2020-05-10.count");
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_hide_paths(&[hide_path]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    handle_monthly_new(&ctx, &src_root, &mut j, /*month_range=*/ 12).unwrap();
    let monthly = &j.as_object().unwrap()["monthly"].as_array().unwrap();
    // 1st element: 2019-05 start -> end
    // No 2nd element, would be diff from last month end -> today
    assert_eq!(monthly.len(), 1);
    assert_eq!(monthly[0], serde_json::json!(["2019-05", 3799]));
}

/// Tests handle_daily_total().
#[test]
fn test_handle_daily_total() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_daily_total(&ctx, &src_root, &mut j, /*day_range=*/ 13).unwrap();
    let dailytotal = &j.as_object().unwrap()["dailytotal"].as_array().unwrap();
    assert_eq!(dailytotal.len(), 1);
    assert_eq!(dailytotal[0], serde_json::json!(["2020-04-27", 251614]));
}

/// Tests handle_daily_total(): the case when the day range is empty.
#[test]
fn test_handle_daily_total_empty_day_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_daily_total(&ctx, &src_root, &mut j, /*day_range=*/ -1).unwrap();
    let dailytotal = &j.as_object().unwrap()["dailytotal"].as_array().unwrap();
    assert_eq!(dailytotal.is_empty(), true);
}

/// Tests handle_user_total().
#[test]
fn test_handle_user_total() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_user_total(&ctx, &src_root, &mut j, /*day_range=*/ 13).unwrap();
    let usertotal = &j.as_object().unwrap()["usertotal"].as_array().unwrap();
    assert_eq!(usertotal.len(), 1);
    assert_eq!(usertotal[0], serde_json::json!(["2020-04-27", 43]));
}

/// Tests handle_user_total(): the case when the day range is empty.
#[test]
fn test_handle_user_total_empty_day_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_user_total(&ctx, &src_root, &mut j, /*day_range=*/ -1).unwrap();
    let usertotal = &j.as_object().unwrap()["usertotal"].as_array().unwrap();
    assert_eq!(usertotal.is_empty(), true);
}

/// Tests handle_monthly_total().
#[test]
fn test_handle_monthly_total() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_monthly_total(&ctx, &src_root, &mut j, /*month_range=*/ 11).unwrap();
    let monthlytotal = &j.as_object().unwrap()["monthlytotal"].as_array().unwrap();
    assert_eq!(monthlytotal.len(), 1);
    assert_eq!(monthlytotal[0], serde_json::json!(["2019-05", 203317]))
}

/// Tests handle_monthly_total(): the case when the day range is empty.
#[test]
fn test_handle_monthly_total_empty_day_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_monthly_total(&ctx, &src_root, &mut j, /*month_range=*/ -1).unwrap();
    let monthlytotal = &j.as_object().unwrap()["monthlytotal"].as_array().unwrap();
    assert_eq!(monthlytotal.is_empty(), true);
}

/// Tests handle_monthly_total(): the case when the day range is of just one element.
#[test]
fn test_handle_monthly_total_one_element_day_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let src_root = ctx.get_abspath("workdir/stats");
    let mut j = serde_json::json!({});
    handle_monthly_total(&ctx, &src_root, &mut j, /*month_range=*/ 0).unwrap();
    let monthlytotal = &j.as_object().unwrap()["monthlytotal"].as_array().unwrap();
    assert_eq!(monthlytotal.len(), 2);
    assert_eq!(monthlytotal[0], serde_json::json!(["2020-04", 253027]));
    assert_eq!(monthlytotal[1], serde_json::json!(["2020-05", 254651]));
}

/// Tests get_previous_month().
#[test]
fn test_get_previous_month() {
    let time: &dyn context::Time = &context::tests::make_test_time();
    let today = time.now();

    let actual = chrono::NaiveDateTime::from_timestamp(get_previous_month(today, 2).unwrap(), 0);

    let expected = chrono::NaiveDate::from_ymd(2020, 3, 31).and_hms(0, 0, 0);
    assert_eq!(actual, expected);
}

/// Tests get_topcities(): the case when the old path is missing.
#[test]
fn test_get_topcities_test_old_missing() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let mut file_system = context::tests::TestFileSystem::new();
    let src_root = ctx.get_abspath("workdir/stats");
    file_system.set_hide_paths(&vec![format!("{}/2020-04-10.citycount", src_root)]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let ret = get_topcities(&ctx, &src_root).unwrap();
    assert_eq!(ret.is_empty(), true);
}

/// Tests get_topcities(): the case when the new path is missing.
#[test]
fn test_get_topcities_test_new_missing() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let mut file_system = context::tests::TestFileSystem::new();
    let src_root = ctx.get_abspath("workdir/stats");
    file_system.set_hide_paths(&vec![format!("{}/2020-05-10.citycount", src_root)]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let ret = get_topcities(&ctx, &src_root).unwrap();
    assert_eq!(ret.is_empty(), true);
}
