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
use crate::sql;
use crate::util;
use anyhow::Context;
use std::collections::HashMap;
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
fn handle_progress(ctx: &context::Context, j: &mut serde_json::Value) -> anyhow::Result<()> {
    let mut ret = serde_json::json!({});
    let conn = ctx.get_database_connection()?;
    let num_ref: f64 = {
        let mut stmt = conn.prepare("select count from counts where category = 'ref'")?;
        let mut counts = stmt.query([])?;
        let count = counts.next()?.context("no count")?;
        let count: String = count.get(0).unwrap();
        count.parse().context("failed to parse ref count")?
    };
    let today = {
        let now = ctx.get_time().now();
        let format = time::format_description::parse("[year]-[month]-[day]")?;
        now.format(&format)?
    };
    let mut num_osm = 0_f64;
    let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
    let mut counts = stmt.query([&today])?;
    if let Some(count) = counts.next()? {
        let count: String = count.get(0).unwrap();
        num_osm = count.parse().context("failed to parse today's count")?;
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
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select city, count from stats_citycounts where date = ?1 order by cast(count as integer) desc")?;
    let mut rows = stmt.query([&today])?;
    while let Some(row) = rows.next()? {
        let city: String = row.get(0).unwrap();
        let count: String = row.get(1).unwrap();
        if city.starts_with("Budapest_") {
            let count: u64 = count.parse()?;
            osm_count += count;
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
fn handle_topusers(ctx: &context::Context, j: &mut serde_json::Value) -> anyhow::Result<()> {
    let today = {
        let now = ctx.get_time().now();
        let format = time::format_description::parse("[year]-[month]-[day]")?;
        now.format(&format)?
    };
    let mut ret: Vec<(String, u64)> = Vec::new();
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select user, count from stats_topusers where date = ?1 order by cast(count as integer) desc")?;
    let mut rows = stmt.query([&today])?;
    while let Some(row) = rows.next()? {
        let user: String = row.get(0).unwrap();
        let count: String = row.get(1).unwrap();
        ret.push((user, count.parse()?));
    }
    j.as_object_mut()
        .unwrap()
        .insert("topusers".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Generates a list of cities, sorted by how many new hours numbers they got recently.
pub fn get_topcities(ctx: &context::Context) -> anyhow::Result<Vec<(String, i64)>> {
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    let new_day = now.format(&ymd)?;
    let day_delta = now - time::Duration::days(30);
    let old_day = day_delta.format(&ymd)?;
    let mut old_counts: HashMap<String, i64> = HashMap::new();
    let mut counts: Vec<(String, i64)> = Vec::new();

    let conn = ctx.get_database_connection()?;
    {
        let mut stmt = conn.prepare("select city, count from stats_citycounts where date = ?1 order by cast(count as integer) desc")?;
        let mut rows = stmt.query([&old_day])?;
        while let Some(row) = rows.next()? {
            let city: String = row.get(0).unwrap();
            let count: String = row.get(1).unwrap();
            let count: i64 = count.parse()?;
            old_counts.insert(city, count);
        }
    }

    {
        let mut stmt = conn.prepare("select city, count from stats_citycounts where date = ?1 order by cast(count as integer) desc")?;
        let mut rows = stmt.query([&new_day])?;
        while let Some(row) = rows.next()? {
            let city: String = row.get(0).unwrap();
            let count: String = row.get(1).unwrap();
            let count: i64 = count.parse()?;
            if old_counts.contains_key(&city) {
                counts.push((city.to_string(), count - old_counts[&city]));
            }
        }
    }
    counts.sort_by_key(|x| x.1);
    counts.reverse();
    Ok(counts)
}

/// Generates stats for top cities.
/// This lists the top 20 cities which got lots of new house numbers in the past 30 days.
fn handle_topcities(ctx: &context::Context, j: &mut serde_json::Value) -> anyhow::Result<()> {
    let mut ret = get_topcities(ctx).context("get_topcities failed")?;
    ret = ret[0..std::cmp::min(20, ret.len())].to_vec();
    j.as_object_mut()
        .unwrap()
        .insert("topcities".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of total users / day.
fn handle_user_total(
    ctx: &context::Context,
    j: &mut serde_json::Value,
    day_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, u64)> = Vec::new();
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - time::Duration::days(day_offset);
        let day = day_delta.format(&ymd)?;
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select count from stats_usercounts where date = ?1")?;
        let mut usercounts = stmt.query([&day])?;
        if let Some(usercount) = usercounts.next()? {
            let usercount: String = usercount.get(0).unwrap();
            ret.push((day.to_string(), usercount.parse()?));
        } else {
            warn!("handle_user_total: no such row: {day}");
            break;
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("usertotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of new housenumbers / day.
fn handle_daily_new(
    ctx: &context::Context,
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
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
        let mut counts = stmt.query([&day])?;
        if let Some(count) = counts.next()? {
            let count: String = count.get(0).unwrap();
            let count: i64 = count.parse()?;
            if prev_count > 0 {
                ret.push((prev_day, count - prev_count));
            }
            prev_count = count;
            prev_day = day.to_string();
        } else {
            warn!("handle_daily_new: no count for date: {day}");
            break;
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("daily".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of invalid addr:city values / day.
fn handle_invalid_addr_cities(
    ctx: &context::Context,
    j: &mut serde_json::Value,
    day_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, i64)> = Vec::new();
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare(
        "select date, count from stats_invalid_addr_cities_counts order by date desc limit ?1;",
    )?;
    let mut rows = stmt.query([day_range])?;
    while let Some(row) = rows.next()? {
        let date: String = row.get(0).unwrap();
        let count: String = row.get(1).unwrap();
        ret.push((date, count.parse::<i64>()?));
    }
    ret.reverse();
    j.as_object_mut()
        .unwrap()
        .insert("invalidAddrCities".into(), serde_json::to_value(&ret)?);

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
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
        let date = format!("{month}-01");
        let mut counts = stmt.query([&date])?;
        if let Some(count) = counts.next()? {
            let count: String = count.get(0).unwrap();
            let count: i64 = count.parse()?;
            if prev_count > 0 {
                ret.push((prev_month, count - prev_count));
            }
            prev_count = count;
            prev_month = month.to_string();
        } else {
            warn!("handle_monthly_new: no such count: {date}");
            break;
        }
    }

    // Also show the current, incomplete month.
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    let mut month = now.format(&ymd)?;
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
    let mut counts = stmt.query([&month])?;
    if let Some(count) = counts.next()? {
        let count: String = count.get(0).unwrap();
        let count: i64 = count.parse()?;
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
    j: &mut serde_json::Value,
    day_range: i64,
) -> anyhow::Result<()> {
    let mut ret: Vec<(String, i64)> = Vec::new();
    let now = ctx.get_time().now();
    let ymd = time::format_description::parse("[year]-[month]-[day]")?;
    for day_offset in (0..=day_range).rev() {
        let day_delta = now - time::Duration::days(day_offset);
        let day = day_delta.format(&ymd)?;
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
        let mut counts = stmt.query([&day])?;
        if let Some(count) = counts.next()? {
            let count: String = count.get(0).unwrap();
            let count: i64 = count.parse()?;
            ret.push((day.to_string(), count));
        } else {
            warn!("handle_daily_total: no such row: {day}");
            break;
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("dailytotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Shows # of total housenumbers / month.
fn handle_monthly_total(
    ctx: &context::Context,
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
        let mut count_date = format!("{month}-01");
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
        let mut counts = stmt.query([&count_date])?;
        if let Some(count) = counts.next()? {
            let count: String = count.get(0).unwrap();
            let count: i64 = count.parse().unwrap();
            ret.push((prev_month.to_string(), count));

            if month_offset == 0 {
                // Current month: show today's count as well.
                month = month_delta.format(&ymd)?;
                count_date = month;
                let mut stmt = conn.prepare("select count from stats_counts where date = ?1")?;
                let mut counts = stmt.query([&count_date])?;
                let count_row = counts.next().unwrap();
                // Assume that today's count is always available.
                let count_row = count_row.unwrap();
                let count: String = count_row.get(0).unwrap();
                let count: i64 = count.parse().unwrap();
                month = month_delta.format(&ym)?;
                ret.push((month.to_string(), count));
            }
        } else {
            warn!("handle_monthly_total: no such date in stats_counts: {count_date}");
            break;
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("monthlytotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

/// Generates the stats json and writes it to `json_path`.
pub fn generate_json(ctx: &context::Context, json_path: &str) -> anyhow::Result<()> {
    let mut j = serde_json::json!({});
    handle_progress(ctx, &mut j).context("handle_progress failed")?;
    handle_capital_progress(ctx, &mut j).context("handle_capital_progress failed")?;
    handle_topusers(ctx, &mut j).context("handle_topusers failed")?;
    handle_topcities(ctx, &mut j).context("handle_topcities failed")?;
    handle_user_total(ctx, &mut j, /*day_range=*/ 13).context("handle_user_total")?;
    handle_daily_new(ctx, &mut j, /*day_range=*/ 14).context("handle_daily_new failed")?;
    handle_daily_total(ctx, &mut j, /*day_range=*/ 13).context("handle_daily_total failed")?;
    handle_monthly_new(ctx, &mut j, /*month_range=*/ 12).context("handle_monthly_new failed")?;
    handle_monthly_total(ctx, &mut j, /*month_range=*/ 11)
        .context("handle_monthly_total failed")?;
    handle_invalid_addr_cities(ctx, &mut j, /*day_range=*/ 14)
        .context("invalid_addr_cities failed")?;
    let stream = ctx.get_file_system().open_write(json_path)?;
    let mut guard = stream.borrow_mut();
    let write = guard.deref_mut();
    serde_json::to_writer(write, &j)?;

    Ok(())
}

pub fn set_sql_mtime(ctx: &context::Context, page: &str) -> anyhow::Result<()> {
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
    let modified = match rows.next()? {
        Some(row) => {
            let last_modified: String = row.get(0)?;
            let nanos: i128 = last_modified.parse()?;
            time::OffsetDateTime::from_unix_timestamp_nanos(nanos)?
        }
        None => time::OffsetDateTime::UNIX_EPOCH,
    };
    let now = ctx.get_time().now();
    Ok(modified.to_offset(now.offset()))
}

pub fn has_sql_mtime(ctx: &context::Context, page: &str) -> anyhow::Result<bool> {
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select last_modified from mtimes where page = ?1")?;
    let mut rows = stmt.query([page])?;
    Ok(rows.next()?.is_some())
}

pub fn set_sql_count(
    ctx: &context::Context,
    table: &str,
    relation: &str,
    count: usize,
) -> anyhow::Result<()> {
    let conn = ctx.get_database_connection()?;
    conn.execute(
        &format!(
            r#"insert into {} (relation, count) values (?1, ?2)
             on conflict(relation) do update set count = excluded.count"#,
            table
        ),
        [relation, &count.to_string()],
    )?;
    Ok(())
}

pub fn get_sql_count(
    ctx: &context::Context,
    table: &str,
    relation: &str,
) -> anyhow::Result<String> {
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare(&format!("select count from {} where relation = ?1", table))?;
    let mut rows = stmt.query([relation])?;
    let row = rows.next()?.context("no count")?;
    let count: String = row.get(0)?;
    Ok(count)
}

pub fn has_sql_count(ctx: &context::Context, table: &str, relation: &str) -> anyhow::Result<bool> {
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare(&format!("select count from {} where relation = ?1", table))?;
    let mut rows = stmt.query([relation])?;
    Ok(rows.next()?.is_some())
}

pub fn set_sql_json(
    ctx: &context::Context,
    table: &str,
    relation: &str,
    json: &str,
) -> anyhow::Result<()> {
    let conn = ctx.get_database_connection()?;
    conn.execute(
        &format!(
            r#"insert into {} (relation, json) values (?1, ?2)
             on conflict(relation) do update set json = excluded.json"#,
            table
        ),
        [relation, json],
    )?;
    Ok(())
}

pub fn get_sql_json(ctx: &context::Context, table: &str, relation: &str) -> anyhow::Result<String> {
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare(&format!("select json from {} where relation = ?1", table))?;
    let mut rows = stmt.query([relation])?;
    let row = rows.next()?.context("no json")?;
    let json: String = row.get(0)?;
    Ok(json)
}

// No has_sql_json(), use has_sql_mtime() instead.

struct InvalidAddrCity {
    osm_id: String,
    osm_type: String,
    postcode: String,
    city: String,
    street: String,
    housenumber: String,
    user: String,
    timestamp: String,
    fixme: String,
}

pub fn update_invalid_addr_cities(ctx: &context::Context) -> anyhow::Result<()> {
    info!("stats: updating invalid_addr_cities");
    let valid_settlements =
        util::get_valid_settlements(ctx).context("get_valid_settlements() failed")?;

    let mut invalids: Vec<InvalidAddrCity> = Vec::new();
    {
        let conn = ctx.get_database_connection()?;
        conn.execute("delete from stats_invalid_addr_cities", [])?;
        let mut stmt = conn.prepare("select postcode, city, street, housenumber, user, osm_id, osm_type, timestamp, fixme from whole_country")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let postcode: String = row.get(0).unwrap();
            let city: String = row.get(1).unwrap();
            let street: String = row.get(2).unwrap();
            let housenumber: String = row.get(3).unwrap();
            let user: String = row.get(4).unwrap();
            let osm_id: String = row.get(5).unwrap();
            let osm_type: String = row.get(6).unwrap();
            let timestamp: String = row.get(7).unwrap();
            let fixme: String = row.get(8).unwrap();
            if !valid_settlements.contains(&city) && city != "budapest" {
                let invalid = InvalidAddrCity {
                    osm_id,
                    osm_type,
                    postcode,
                    city,
                    street,
                    housenumber,
                    user,
                    timestamp,
                    fixme,
                };
                invalids.push(invalid);
            }
        }
    }

    let mut conn = ctx.get_database_connection()?;
    let tx = conn.transaction()?;
    // Also append a row in the stats_invalid_addr_cities_counts table so we can chart this.
    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;
    for row in &invalids {
        tx.execute("insert into stats_invalid_addr_cities (osm_id, osm_type, postcode, city, street, housenumber, user, timestamp, fixme) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            [&row.osm_id, &row.osm_type, &row.postcode, &row.city, &row.street, &row.housenumber, &row.user, &row.timestamp, &row.fixme])?;
    }
    // It's OK to not overwrite previous count from the same day.
    sql::ignore_primary_key_constraint(tx.execute(
        "insert into stats_invalid_addr_cities_counts (date, count) values (?1, ?2)",
        [today, invalids.len().to_string()],
    ))
    .context("insert into stats_invalid_addr_cities_counts failed")?;
    tx.commit()?;

    Ok(())
}

#[cfg(test)]
mod tests;
