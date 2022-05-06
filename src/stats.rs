/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The stats module creates statistics about missing / non-missing house numbers.

use crate::context;
use crate::util;
use anyhow::Context;
use chrono::Datelike;
use std::collections::HashMap;
use std::io::BufRead;
use std::ops::DerefMut;

/// Generates stats for a global progressbar.
fn handle_progress(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let mut ret = serde_json::json!({});
    let num_ref: f64 = ctx
        .get_file_system()
        .read_to_string(&format!("{}/ref.count", src_root))
        .context("failed to read ref.count")?
        .trim()
        .parse()
        .context("failed to parse ref.count")?;
    let today = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let mut num_osm = 0_f64;
    let count_path = format!("{}/{}.count", src_root, today);
    if ctx.get_file_system().path_exists(&count_path) {
        num_osm = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()
            .context("failed to parse today.count")?;
    }
    // Round to 2 digits.
    let percentage = ((num_osm * 100.0 / num_ref) * 100.0).round() / 100.0;
    let ret_obj = ret.as_object_mut().unwrap();
    ret_obj.insert("date".into(), serde_json::json!(today));
    ret_obj.insert("percentage".into(), serde_json::json!(percentage));
    ret_obj.insert("reference".into(), serde_json::json!(num_ref));
    ret_obj.insert("osm".into(), serde_json::json!(num_osm));
    j.as_object_mut().unwrap().insert("progress".into(), ret);

    Ok(())
}

/// Generates status for the progress of the capital.
fn handle_capital_progress(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let mut ret = serde_json::json!({});
    let mut ref_count = 0;
    {
        let ref_path = ctx.get_ini().get_reference_citycounts_path()?;
        let stream = ctx.get_file_system().open_read(&ref_path)?;
        let mut guard = stream.borrow_mut();
        let mut read = guard.deref_mut();
        let mut csv_read = util::CsvRead::new(&mut read);
        let mut first = true;
        for result in csv_read.records() {
            let row = result?;
            if first {
                first = false;
                continue;
            }

            if row[0].starts_with("budapest_") {
                ref_count += row[1].parse::<i32>()?;
            }
        }
    }

    let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    let today = now.format("%Y-%m-%d").to_string();
    let mut osm_count = 0;
    let osm_path = format!("{}/{}.citycount", src_root, today);
    if ctx.get_file_system().path_exists(&osm_path) {
        let stream = ctx.get_file_system().open_read(&osm_path)?;
        let mut guard = stream.borrow_mut();
        let mut read = guard.deref_mut();
        let mut csv_read = util::CsvRead::new(&mut read);
        let mut first = true;
        for result in csv_read.records() {
            let row = result?;
            if first {
                first = false;
                continue;
            }

            if row[0].starts_with("budapest_") {
                osm_count += row[1].parse::<i32>()?;
            }
        }
    }

    // Round to 2 digits.
    let percentage = ((osm_count as f64 * 100.0 / ref_count as f64) * 100.0).round() / 100.0;
    let ret_obj = ret.as_object_mut().unwrap();
    ret_obj.insert("date".into(), serde_json::json!(today));
    ret_obj.insert("percentage".into(), serde_json::json!(percentage));
    ret_obj.insert("reference".into(), serde_json::json!(ref_count));
    ret_obj.insert("osm".into(), serde_json::json!(osm_count));
    j.as_object_mut()
        .unwrap()
        .insert("capital-progress".into(), ret);

    Ok(())
}

/// Generates stats for top users.
fn handle_topusers(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let today = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let mut ret: Vec<(String, String)> = Vec::new();
    let topusers_path = format!("{}/{}.topusers", src_root, today);
    if ctx.get_file_system().path_exists(&topusers_path) {
        let stream = std::io::BufReader::new(std::fs::File::open(topusers_path)?);
        for line in stream.lines() {
            let line = line?.trim().to_string();
            let mut tokens = line.split(' ');
            let count = tokens.next().unwrap();
            let user = tokens.next();
            if user.is_none() {
                // Busted, skip it.
                continue;
            }
            let user = user.unwrap();
            ret.push((user.into(), count.into()));
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("topusers".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Generates a list of cities, sorted by how many new hours numbers they got recently.
pub fn get_topcities(ctx: &context::Context, src_root: &str) -> anyhow::Result<Vec<(String, i64)>> {
    let new_day = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let day_delta =
        chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0) - chrono::Duration::days(30);
    let old_day = day_delta.format("%Y-%m-%d").to_string();
    let mut old_counts: HashMap<String, i64> = HashMap::new();
    let mut counts: Vec<(String, i64)> = Vec::new();

    let old_count_path = format!("{}/{}.citycount", src_root, old_day);
    if !ctx.get_file_system().path_exists(&old_count_path) {
        return Ok(vec![]);
    }
    let stream = ctx.get_file_system().open_read(&old_count_path)?;
    let mut guard = stream.borrow_mut();
    let mut read = std::io::BufReader::new(guard.deref_mut());
    let mut csv_read = util::CsvRead::new(&mut read);
    for result in csv_read.records() {
        let row = result?;
        let mut tokens = row.iter();
        let city = match tokens.next() {
            Some(value) => value,
            None => {
                continue;
            }
        };
        if city.is_empty() {
            continue;
        }
        let count = match tokens.next() {
            Some(value) => value,
            None => {
                continue;
            }
        };
        if !count.is_empty() {
            let count: i64 = count.parse()?;
            old_counts.insert(city.into(), count);
        }
    }

    let new_count_path = format!("{}/{}.citycount", src_root, new_day);
    if !ctx.get_file_system().path_exists(&new_count_path) {
        return Ok(vec![]);
    }
    let stream = ctx.get_file_system().open_read(&new_count_path)?;
    let mut guard = stream.borrow_mut();
    let mut read = std::io::BufReader::new(guard.deref_mut());
    let mut csv_read = util::CsvRead::new(&mut read);
    for result in csv_read.records() {
        let row = result?;
        let mut tokens = row.iter();
        let city = match tokens.next() {
            Some(value) => value,
            None => {
                continue;
            }
        };
        let count = match tokens.next() {
            Some(value) => value,
            None => {
                continue;
            }
        };
        if !count.is_empty() && old_counts.contains_key(city) {
            let count: i64 = count.parse()?;
            counts.push((city.into(), count - old_counts[city]));
        }
    }
    counts.sort_by_key(|x| x.1);
    counts.reverse();
    Ok(counts)
}

/// Generates stats for top cities.
/// This lists the top 20 cities which got lots of new house numbers in the past 30 days.
fn handle_topcities(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let mut ret = get_topcities(ctx, src_root)?;
    ret = ret[0..std::cmp::min(20, ret.len())].to_vec();
    j.as_object_mut()
        .unwrap()
        .insert("topcities".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of total users / day.
fn handle_user_total(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
    day_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, u64)> = Vec::new();
    let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - chrono::Duration::days(day_offset);
        let day = day_delta.format("%Y-%m-%d");
        let count_path = format!("{}/{}.usercount", src_root, day);
        if !ctx.get_file_system().path_exists(&count_path) {
            break;
        }
        let count: u64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        ret.push((day.to_string(), count));
    }
    j.as_object_mut()
        .unwrap()
        .insert("usertotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of new housenumbers / day.
fn handle_daily_new(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
    day_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, i64)> = Vec::new();
    let mut prev_count = 0;
    let mut prev_day: String = "".into();
    let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - chrono::Duration::days(day_offset);
        let day = day_delta.format("%Y-%m-%d");
        let count_path = format!("{}/{}.count", src_root, day);
        if !ctx.get_file_system().path_exists(&count_path) {
            break;
        }
        let count: i64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        if prev_count > 0 {
            ret.push((prev_day, count - prev_count));
        }
        prev_count = count;
        prev_day = day.to_string();
    }
    j.as_object_mut()
        .unwrap()
        .insert("daily".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Returns a date that was today N months ago.
fn get_previous_month(today: i64, months: i64) -> anyhow::Result<i64> {
    let today = chrono::NaiveDateTime::from_timestamp(today, 0);
    let mut month_ago = today;
    for _month in 0..months {
        let first_of_current = month_ago.with_day(1).unwrap();
        month_ago = first_of_current - chrono::Duration::days(1);
    }
    Ok(month_ago.timestamp())
}

/// Shows # of new housenumbers / month.
fn handle_monthly_new(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
    month_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, i64)> = Vec::new();
    let mut prev_count = 0;
    let mut prev_month: String = "".into();
    for month_offset in (0..=month_range).rev() {
        let month_delta = chrono::NaiveDateTime::from_timestamp(
            get_previous_month(ctx.get_time().now(), month_offset)?,
            0,
        );
        // Get the first day of each month.
        let month = month_delta.with_day(1).unwrap().format("%Y-%m");
        let count_path = format!("{}/{}-01.count", src_root, month);
        if !ctx.get_file_system().path_exists(&count_path) {
            break;
        }
        let count: i64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        if prev_count > 0 {
            ret.push((prev_month, count - prev_count));
        }
        prev_count = count;
        prev_month = month.to_string();
    }

    // Also show the current, incomplete month.
    let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    let mut month = now.format("%Y-%m-%d").to_string();
    let count_path = format!("{}/{}.count", src_root, month);
    if ctx.get_file_system().path_exists(&count_path) {
        let count: i64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        month = now.format("%Y-%m").to_string();
        ret.push((month, count - prev_count));
    }

    j.as_object_mut()
        .unwrap()
        .insert("monthly".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of total housenumbers / day.
fn handle_daily_total(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
    day_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, i64)> = Vec::new();
    let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - chrono::Duration::days(day_offset);
        let day = day_delta.format("%Y-%m-%d");
        let count_path = format!("{}/{}.count", src_root, day);
        if !ctx.get_file_system().path_exists(&count_path) {
            break;
        }
        let count: i64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        ret.push((day.to_string(), count));
    }
    j.as_object_mut()
        .unwrap()
        .insert("dailytotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of total housenumbers / month.
fn handle_monthly_total(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
    month_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, i64)> = Vec::new();
    let today = ctx.get_time().now();
    for month_offset in (0..=month_range).rev() {
        let month_delta =
            chrono::NaiveDateTime::from_timestamp(get_previous_month(today, month_offset)?, 0);
        let prev_month_delta =
            chrono::NaiveDateTime::from_timestamp(get_previous_month(today, month_offset + 1)?, 0);
        // Get the first day of each past month.
        let mut month = month_delta.with_day(1).unwrap().format("%Y-%m");
        let prev_month = prev_month_delta.with_day(1).unwrap().format("%Y-%m");
        let mut count_path = format!("{}/{}-01.count", src_root, month);
        if !ctx.get_file_system().path_exists(&count_path) {
            break;
        }
        let count: i64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        ret.push((prev_month.to_string(), count));

        if month_offset == 0 {
            // Current month: show today's count as well.
            month = month_delta.format("%Y-%m-%d");
            count_path = format!("{}/{}.count", src_root, month);
            let count: i64 = ctx
                .get_file_system()
                .read_to_string(&count_path)?
                .trim()
                .parse()?;
            month = month_delta.format("%Y-%m");
            ret.push((month.to_string(), count));
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("monthlytotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Generates the stats json and writes it to `json_path`.
pub fn generate_json(
    ctx: &context::Context,
    state_dir: &str,
    json_path: &str,
) -> anyhow::Result<()> {
    let mut j = serde_json::json!({});
    handle_progress(ctx, state_dir, &mut j)?;
    handle_capital_progress(ctx, state_dir, &mut j)?;
    handle_topusers(ctx, state_dir, &mut j)?;
    handle_topcities(ctx, state_dir, &mut j)?;
    handle_user_total(ctx, state_dir, &mut j, /*day_range=*/ 13)?;
    handle_daily_new(ctx, state_dir, &mut j, /*day_range=*/ 14)?;
    handle_daily_total(ctx, state_dir, &mut j, /*day_range=*/ 13)?;
    handle_monthly_new(ctx, state_dir, &mut j, /*month_range=*/ 12)?;
    handle_monthly_total(ctx, state_dir, &mut j, /*month_range=*/ 11)?;
    let stream = ctx.get_file_system().open_write(json_path)?;
    let mut guard = stream.borrow_mut();
    let write = guard.deref_mut();
    serde_json::to_writer(write, &j)?;

    Ok(())
}

#[cfg(test)]
mod tests;
