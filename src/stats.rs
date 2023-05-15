/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The stats module creates statistics about missing / non-missing house numbers.

use crate::context;
use crate::util;
use anyhow::Context;
use std::collections::HashMap;
use std::io::BufRead;
use std::ops::DerefMut;

#[cfg(not(test))]
use log::info;
#[cfg(not(test))]
use log::warn;

#[cfg(test)]
use std::println as info;
#[cfg(test)]
use std::println as warn;

/// Generates stats for a global progressbar.
fn handle_progress(
    ctx: &context::Context,
    src_root: &str,
    j: &mut serde_json::Value,
) -> anyhow::Result<()> {
    let mut ret = serde_json::json!({});
    let num_ref: f64 = ctx
        .get_file_system()
        .read_to_string(&format!("{src_root}/ref.count"))
        .context("failed to read ref.count")?
        .trim()
        .parse()
        .context("failed to parse ref.count")?;
    let today = {
        let now = ctx.get_time().now();
        let format = time::format_description::parse("[year]-[month]-[day]")?;
        now.format(&format)?
    };
    let mut num_osm = 0_f64;
    let count_path = format!("{src_root}/{today}.count");
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
        let mut csv_reader = util::make_csv_reader(&mut read);
        for result in csv_reader.deserialize() {
            let row: util::CityCount = result?;

            if row.city.starts_with("budapest_") {
                ref_count += row.count;
            }
        }
    }

    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;
    let mut osm_count = 0;
    let osm_path = format!("{src_root}/{today}.citycount");
    if ctx.get_file_system().path_exists(&osm_path) {
        let stream = ctx.get_file_system().open_read(&osm_path)?;
        let mut guard = stream.borrow_mut();
        let mut read = guard.deref_mut();
        let mut csv_reader = util::make_csv_reader(&mut read);
        for result in csv_reader.deserialize() {
            let row: util::CityCount = result?;
            if row.city.starts_with("budapest_") {
                osm_count += row.count;
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
        let now = ctx.get_time().now();
        let format = time::format_description::parse("[year]-[month]-[day]")?;
        now.format(&format)?
    };
    let mut ret: Vec<(String, String)> = Vec::new();
    let topusers_path = format!("{src_root}/{today}.topusers");
    if ctx.get_file_system().path_exists(&topusers_path) {
        let stream = ctx.get_file_system().open_read(&topusers_path)?;
        let mut guard = stream.borrow_mut();
        let read = std::io::BufReader::new(guard.deref_mut());
        let mut first = true;
        for line in read.lines() {
            if first {
                first = false;
                continue;
            }
            let line = line?.trim().to_string();
            let mut tokens = line.split('\t');
            let count = tokens.next().unwrap();
            let user = tokens.next().unwrap();
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
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    let new_day = now.format(&ymd)?;
    let day_delta = now - time::Duration::days(30);
    let old_day = day_delta.format(&ymd)?;
    let mut old_counts: HashMap<String, u64> = HashMap::new();
    let mut counts: Vec<(String, i64)> = Vec::new();

    let old_count_path = format!("{src_root}/{old_day}.citycount");
    if !ctx.get_file_system().path_exists(&old_count_path) {
        info!("get_topcities: empty result: no such path: {old_count_path}");
        return Ok(vec![]);
    }
    let stream = ctx.get_file_system().open_read(&old_count_path)?;
    let mut guard = stream.borrow_mut();
    let mut read = std::io::BufReader::new(guard.deref_mut());
    let mut csv_reader = util::make_csv_reader(&mut read);
    for result in csv_reader.deserialize() {
        let row: util::CityCount =
            result.context(format!("failed to read row in {old_count_path}"))?;
        old_counts.insert(row.city, row.count);
    }

    let new_count_path = format!("{src_root}/{new_day}.citycount");
    if !ctx.get_file_system().path_exists(&new_count_path) {
        return Ok(vec![]);
    }
    let stream = ctx.get_file_system().open_read(&new_count_path)?;
    let mut guard = stream.borrow_mut();
    let mut read = std::io::BufReader::new(guard.deref_mut());
    let mut csv_reader = util::make_csv_reader(&mut read);
    for result in csv_reader.deserialize() {
        let row: util::CityCount = result?;
        if old_counts.contains_key(&row.city) {
            counts.push((
                row.city.to_string(),
                row.count as i64 - old_counts[&row.city] as i64,
            ));
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
    let mut ret = get_topcities(ctx, src_root).context("get_topcities failed")?;
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
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - time::Duration::days(day_offset);
        let day = day_delta.format(&ymd)?;
        let count_path = format!("{src_root}/{day}.usercount");
        if !ctx.get_file_system().path_exists(&count_path) {
            warn!("handle_user_total: no such path: {count_path}");
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
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - time::Duration::days(day_offset);
        let day = day_delta.format(&ymd)?;
        let count_path = format!("{src_root}/{day}.count");
        if !ctx.get_file_system().path_exists(&count_path) {
            warn!("handle_daily_new: no such path: {count_path}");
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
fn get_previous_month(
    today: &time::OffsetDateTime,
    months: i64,
) -> anyhow::Result<time::OffsetDateTime> {
    let mut month_ago = *today;
    for _month in 0..months {
        let first_of_current = month_ago.replace_day(1).unwrap();
        month_ago = first_of_current - time::Duration::days(1);
    }
    Ok(month_ago)
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
    let ym = time::format_description::parse("[year]-[month]")?;
    for month_offset in (0..=month_range).rev() {
        let month_delta = get_previous_month(&ctx.get_time().now(), month_offset)?;
        // Get the first day of each month.
        let month = month_delta.replace_day(1).unwrap().format(&ym)?;
        let count_path = format!("{src_root}/{month}-01.count");
        if !ctx.get_file_system().path_exists(&count_path) {
            warn!("handle_monthly_new: no such path: {count_path}");
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
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    let mut month = now.format(&ymd)?;
    let count_path = format!("{src_root}/{month}.count");
    if ctx.get_file_system().path_exists(&count_path) {
        let count: i64 = ctx
            .get_file_system()
            .read_to_string(&count_path)?
            .trim()
            .parse()?;
        month = now.format(&ym)?;
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
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - time::Duration::days(day_offset);
        let day = day_delta.format(&ymd)?;
        let count_path = format!("{src_root}/{day}.count");
        if !ctx.get_file_system().path_exists(&count_path) {
            warn!("handle_daily_total: no such path: {count_path}");
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
    let ym = time::format_description::parse("[year]-[month]")?;
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    for month_offset in (0..=month_range).rev() {
        let month_delta = get_previous_month(&today, month_offset)?;
        let prev_month_delta = get_previous_month(&today, month_offset + 1)?;
        // Get the first day of each past month.
        let mut month = month_delta.replace_day(1)?.format(&ym)?;
        let prev_month = prev_month_delta.replace_day(1).unwrap().format(&ym)?;
        let mut count_path = format!("{src_root}/{month}-01.count");
        if !ctx.get_file_system().path_exists(&count_path) {
            warn!("handle_monthly_total: no such path: {count_path}");
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
            month = month_delta.format(&ymd)?;
            count_path = format!("{src_root}/{month}.count");
            let count: i64 = ctx
                .get_file_system()
                .read_to_string(&count_path)?
                .trim()
                .parse()?;
            month = month_delta.format(&ym)?;
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
    handle_progress(ctx, state_dir, &mut j).context("handle_progress failed")?;
    handle_capital_progress(ctx, state_dir, &mut j).context("handle_capital_progress failed")?;
    handle_topusers(ctx, state_dir, &mut j).context("handle_topusers failed")?;
    handle_topcities(ctx, state_dir, &mut j).context("handle_topcities failed")?;
    handle_user_total(ctx, state_dir, &mut j, /*day_range=*/ 13).context("handle_user_total")?;
    handle_daily_new(ctx, state_dir, &mut j, /*day_range=*/ 14)
        .context("handle_daily_new failed")?;
    handle_daily_total(ctx, state_dir, &mut j, /*day_range=*/ 13)
        .context("handle_daily_total failed")?;
    handle_monthly_new(ctx, state_dir, &mut j, /*month_range=*/ 12)
        .context("handle_monthly_new failed")?;
    handle_monthly_total(ctx, state_dir, &mut j, /*month_range=*/ 11)
        .context("handle_monthly_total failed")?;
    let stream = ctx.get_file_system().open_write(json_path)?;
    let mut guard = stream.borrow_mut();
    let write = guard.deref_mut();
    serde_json::to_writer(write, &j)?;

    Ok(())
}

fn set_sql_mtime(ctx: &context::Context, page: &str) -> anyhow::Result<()> {
    let conn = ctx.get_database_connection()?;
    conn.execute(
        r#"insert into mtimes (page, last_modified) values (?1, ?2)
             on conflict(page) do update set last_modified = excluded.last_modified"#,
        [
            page,
            &ctx.get_time().now().unix_timestamp_nanos().to_string(),
        ],
    )?;
    Ok(())
}

pub fn get_sql_mtime(ctx: &context::Context, page: &str) -> anyhow::Result<time::OffsetDateTime> {
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select last_modified from mtimes where page = ?1")?;
    let mut rows = stmt.query([page])?;
    let row = rows.next()?.context("no next row")?;
    let last_modified: String = row.get(0)?;
    let nanos: i128 = last_modified.parse()?;
    let modified = time::OffsetDateTime::from_unix_timestamp_nanos(nanos)?;
    let now = ctx.get_time().now();
    Ok(modified.to_offset(now.offset()))
}

pub fn update_invalid_addr_cities(ctx: &context::Context, state_dir: &str) -> anyhow::Result<()> {
    info!("stats: updating invalid_addr_cities");
    let valid_settlements =
        util::get_valid_settlements(ctx).context("get_valid_settlements() failed")?;
    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;
    let csv_path = format!("{state_dir}/{today}.csv");
    if !ctx.get_file_system().path_exists(&csv_path) {
        warn!("update_invalid_addr_cities: no such path: {csv_path}");
        return Ok(());
    }

    let stream = ctx.get_file_system().open_read(&csv_path)?;
    let mut guard = stream.borrow_mut();
    let mut read = std::io::BufReader::new(guard.deref_mut());
    let mut csv_reader = util::make_csv_reader(&mut read);
    {
        let mut conn = ctx.get_database_connection()?;
        conn.execute("delete from stats_invalid_addr_cities", [])?;
        let tx = conn.transaction()?;
        for result in csv_reader.deserialize() {
            let row: util::OsmLightHouseNumber = result?;
            let city = row.city.to_lowercase();
            if !valid_settlements.contains(&city) && city != "budapest" {
                tx.execute("insert into stats_invalid_addr_cities (osm_id, osm_type, postcode, city, street, housenumber, user) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                       [row.osm_id, row.osm_type, row.postcode, row.city, row.street, row.housenumber, row.user])?;
            }
        }
        tx.commit()?;
    }

    set_sql_mtime(ctx, "stats/invalid-addr-cities")?;

    Ok(())
}

#[cfg(test)]
mod tests;
