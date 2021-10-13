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
use anyhow::Context;
use chrono::Datelike;
use pyo3::prelude::*;
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
    let num_ref: f64 = std::fs::read_to_string(format!("{}/ref.count", src_root))?
        .trim()
        .parse()
        .context("failed to parse ref.count")?;
    let today = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let mut num_osm = 0_f64;
    let count_path = format!("{}/{}.count", src_root, today);
    if std::path::Path::new(&count_path).exists() {
        num_osm = std::fs::read_to_string(&count_path)?
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
    if std::path::Path::new(&topusers_path).exists() {
        let stream = std::io::BufReader::new(std::fs::File::open(topusers_path)?);
        for line in stream.lines() {
            let line = line?.trim().to_string();
            let mut tokens = line.split(' ');
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

#[pyfunction]
fn py_handle_topusers(ctx: context::PyContext, src_root: &str, j: &str) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_topusers(&ctx.context, src_root, &mut j).context("handle_topusers() failed") {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Generates a list of cities, sorted by how many new hours numbers they got recently.
fn get_topcities(ctx: &context::Context, src_root: &str) -> anyhow::Result<Vec<(String, u64)>> {
    let new_day = {
        let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
        now.format("%Y-%m-%d").to_string()
    };
    let day_delta =
        chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0) - chrono::Duration::days(30);
    let old_day = day_delta.format("%Y-%m-%d").to_string();
    let mut old_counts: HashMap<String, u64> = HashMap::new();
    let mut counts: Vec<(String, u64)> = Vec::new();

    let old_count_path = format!("{}/{}.citycount", src_root, old_day);
    if !ctx.get_file_system().path_exists(&old_count_path) {
        return Ok(vec![]);
    }
    let stream = ctx.get_file_system().open_read(&old_count_path)?;
    let mut guard = stream.lock().unwrap();
    let read = std::io::BufReader::new(guard.deref_mut());
    for result in read.lines() {
        let line = result?;
        let mut tokens = line.trim().split('\t');
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
        if !count.is_empty() {
            let count: u64 = count.parse()?;
            old_counts.insert(city.into(), count);
        }
    }

    let new_count_path = format!("{}/{}.citycount", src_root, new_day);
    if !ctx.get_file_system().path_exists(&new_count_path) {
        return Ok(vec![]);
    }
    let stream = ctx.get_file_system().open_read(&new_count_path)?;
    let mut guard = stream.lock().unwrap();
    let read = std::io::BufReader::new(guard.deref_mut());
    for result in read.lines() {
        let line = result?;
        let mut tokens = line.trim().split('\t');
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
            let count: u64 = count.parse()?;
            counts.push((city.into(), count - old_counts[city]));
        }
    }
    counts.sort_by_key(|x| x.1);
    counts.reverse();
    Ok(counts)
}

#[pyfunction]
fn py_get_topcities(ctx: context::PyContext, src_root: &str) -> PyResult<Vec<(String, u64)>> {
    match get_topcities(&ctx.context, src_root).context("get_topcities() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

#[pyfunction]
fn py_handle_topcities(ctx: context::PyContext, src_root: &str, j: &str) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_topcities(&ctx.context, src_root, &mut j).context("handle_topcities() failed") {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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
        if !std::path::Path::new(&count_path).exists() {
            break;
        }
        let count: u64 = std::fs::read_to_string(count_path)?.trim().parse()?;
        ret.push((day.to_string(), count));
    }
    j.as_object_mut()
        .unwrap()
        .insert("usertotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

#[pyfunction]
fn py_handle_user_total(
    ctx: context::PyContext,
    src_root: &str,
    j: &str,
    day_range: i64,
) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_user_total(&ctx.context, src_root, &mut j, day_range)
        .context("handle_user_total() failed")
    {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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
        if !std::path::Path::new(&count_path).exists() {
            break;
        }
        let count: i64 = std::fs::read_to_string(count_path)?.trim().parse()?;
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

#[pyfunction]
fn py_handle_daily_new(
    ctx: context::PyContext,
    src_root: &str,
    j: &str,
    day_range: i64,
) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_daily_new(&ctx.context, src_root, &mut j, day_range)
        .context("handle_daily_new() failed")
    {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

#[pyfunction]
fn py_get_previous_month(today: i64, months: i64) -> PyResult<i64> {
    match get_previous_month(today, months).context("get_previous_month() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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
        let count: i64 = std::fs::read_to_string(count_path)?.trim().parse()?;
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
        let count: i64 = std::fs::read_to_string(count_path)?.trim().parse()?;
        month = now.format("%Y-%m").to_string();
        ret.push((month, count - prev_count));
    }

    j.as_object_mut()
        .unwrap()
        .insert("monthly".into(), serde_json::to_value(&ret)?);

    Ok(())
}

#[pyfunction]
fn py_handle_monthly_new(
    ctx: context::PyContext,
    src_root: &str,
    j: &str,
    day_range: i64,
) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_monthly_new(&ctx.context, src_root, &mut j, day_range)
        .context("handle_monthly_new() failed")
    {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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
        if !std::path::Path::new(&count_path).exists() {
            break;
        }
        let count: i64 = std::fs::read_to_string(count_path)?.trim().parse()?;
        ret.push((day.to_string(), count));
    }
    j.as_object_mut()
        .unwrap()
        .insert("dailytotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

#[pyfunction]
fn py_handle_daily_total(
    ctx: context::PyContext,
    src_root: &str,
    j: &str,
    day_range: i64,
) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_daily_total(&ctx.context, src_root, &mut j, day_range)
        .context("handle_daily_total() failed")
    {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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
        let count: i64 = std::fs::read_to_string(count_path)?.trim().parse()?;
        ret.push((prev_month.to_string(), count));

        if month_offset == 0 {
            // Current month: show today's count as well.
            month = month_delta.format("%Y-%m-%d");
            count_path = format!("{}/{}.count", src_root, month);
            let count: i64 = std::fs::read_to_string(count_path)?.trim().parse()?;
            month = month_delta.format("%Y-%m");
            ret.push((month.to_string(), count));
        }
    }
    j.as_object_mut()
        .unwrap()
        .insert("monthlytotal".into(), serde_json::to_value(&ret)?);

    Ok(())
}

#[pyfunction]
fn py_handle_monthly_total(
    ctx: context::PyContext,
    src_root: &str,
    j: &str,
    month_range: i64,
) -> PyResult<String> {
    let mut j = serde_json::from_str(j).unwrap();
    match handle_monthly_total(&ctx.context, src_root, &mut j, month_range)
        .context("handle_monthly_total() failed")
    {
        Ok(_) => Ok(serde_json::to_string(&j).unwrap()),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Generates the stats json and writes it to `json_path`.
fn generate_json(ctx: &context::Context, state_dir: &str, json_path: &str) -> anyhow::Result<()> {
    let mut j = serde_json::json!({});
    handle_progress(ctx, state_dir, &mut j)?;
    handle_topusers(ctx, state_dir, &mut j)?;
    handle_topcities(ctx, state_dir, &mut j)?;
    handle_user_total(ctx, state_dir, &mut j, /*day_range=*/ 13)?;
    handle_daily_new(ctx, state_dir, &mut j, /*day_range=*/ 14)?;
    handle_daily_total(ctx, state_dir, &mut j, /*day_range=*/ 13)?;
    handle_monthly_new(ctx, state_dir, &mut j, /*month_range=*/ 12)?;
    handle_monthly_total(ctx, state_dir, &mut j, /*month_range=*/ 11)?;
    let stream = ctx.get_file_system().open_write(json_path)?;
    let mut guard = stream.lock().unwrap();
    let write = guard.deref_mut();
    serde_json::to_writer(write, &j)?;

    Ok(())
}

#[pyfunction]
fn py_generate_json(ctx: context::PyContext, state_dir: &str, json_path: &str) -> PyResult<()> {
    match generate_json(&ctx.context, state_dir, json_path).context("generate_json() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Registers Python wrappers of Rust structs into the Python module.
pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_handle_topusers, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_topcities, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_topcities, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_user_total, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_daily_new, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_previous_month, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_monthly_new, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_daily_total, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_monthly_total, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_generate_json, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
        let src_root = ctx.get_abspath("workdir/stats").unwrap();
        let mut j = serde_json::json!({});
        handle_progress(&ctx, &src_root, &mut j).unwrap();
        let progress = &j.as_object().unwrap()["progress"];
        assert_eq!(progress["date"], "2020-05-10");
        // 254651 / 300 * 100
        assert_eq!(progress["percentage"], 84883.67);
    }

    /// Tests handle_progress(): the case when the .count file doesn't exist for a date.
    #[test]
    fn test_handle_progress_old_time() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let time = make_test_time_old();
        let time_arc: Arc<dyn context::Time> = Arc::new(time);
        ctx.set_time(&time_arc);
        let src_root = ctx.get_abspath("workdir/stats").unwrap();
        let mut j = serde_json::json!({});
        handle_progress(&ctx, &src_root, &mut j).unwrap();
        let progress = &j.as_object().unwrap()["progress"];
        assert_eq!(progress["date"], "1970-01-01");
    }

    /// Tests handle_topusers().
    #[test]
    fn test_handle_topusers() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let time = context::tests::make_test_time();
        let time_arc: Arc<dyn context::Time> = Arc::new(time);
        ctx.set_time(&time_arc);
        let src_root = ctx.get_abspath("workdir/stats").unwrap();
        let mut j = serde_json::json!({});
        handle_topusers(&ctx, &src_root, &mut j).unwrap();
        let topusers = &j.as_object().unwrap()["topusers"].as_array().unwrap();
        assert_eq!(topusers.len(), 20);
        assert_eq!(topusers[0], serde_json::json!(["user1", "68885"]));
    }
}
