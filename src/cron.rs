/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cron module allows doing nightly tasks.

use crate::area_files;
use crate::areas;
use crate::context;
use crate::overpass_query;
use crate::stats;
use crate::util;
use anyhow::Context;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufRead;
use std::io::Write;
use std::ops::DerefMut;

#[cfg(not(test))]
use log::{error, info, warn};

#[cfg(test)]
use std::{println as info, println as warn, println as error};

/// Sleeps to respect overpass rate limit.
fn overpass_sleep(ctx: &context::Context) {
    loop {
        let sleep = overpass_query::overpass_query_need_sleep(ctx);
        if sleep == 0 {
            break;
        }
        info!("overpass_sleep: waiting for {sleep} seconds");
        ctx.get_time().sleep(sleep as u64);
    }
}

/// Decides if we should retry a query or not.
fn should_retry(retry: i32) -> bool {
    retry < 20
}

/// Update the OSM street list of all relations.
fn update_osm_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    update: bool,
) -> anyhow::Result<()> {
    let active_names = relations.get_active_names();
    for relation_name in active_names.context("get_active_names() failed")? {
        let relation = relations.get_relation(&relation_name)?;
        if !update && stats::has_sql_mtime(ctx, &format!("streets/{}", relation_name))? {
            continue;
        }
        info!("update_osm_streets, json: start: {relation_name}");
        let mut retry = 0;
        while should_retry(retry) {
            if retry > 0 {
                info!("update_osm_streets, json: try #{retry}");
            }
            retry += 1;
            overpass_sleep(ctx);
            let query = relation.get_osm_streets_json_query()?;
            let buf = match overpass_query::overpass_query(ctx, &query) {
                Ok(value) => value,
                Err(err) => {
                    info!("update_osm_streets, json: http error: {err:?}");
                    continue;
                }
            };
            relation
                .get_files()
                .write_osm_json_streets(ctx, &buf)
                .context("write_osm_json_streets() failed")?;
            break;
        }
        info!("update_osm_streets, json: end: {relation_name}");
    }

    Ok(())
}

/// Update the OSM housenumber list of all relations.
fn update_osm_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    update: bool,
) -> anyhow::Result<()> {
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update && stats::has_sql_mtime(ctx, &format!("housenumbers/{}", relation_name))? {
            continue;
        }
        info!("update_osm_housenumbers, json: start: {relation_name}");
        let mut retry = 0;
        while should_retry(retry) {
            if retry > 0 {
                info!("update_osm_housenumbers, json: try #{retry}");
            }
            retry += 1;
            overpass_sleep(ctx);
            let query = relation.get_osm_housenumbers_json_query()?;
            let buf = match overpass_query::overpass_query(ctx, &query) {
                Ok(value) => value,
                Err(err) => {
                    info!("update_osm_housenumbers, json: http error: {err:?}");
                    continue;
                }
            };
            relation
                .get_files()
                .write_osm_json_housenumbers(ctx, &buf)?;
            break;
        }
        info!("update_osm_housenumbers, json: end: {relation_name}");
    }

    Ok(())
}

/// Update the reference housenumber list of all relations.
fn update_ref_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    update: bool,
) -> anyhow::Result<()> {
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update
            && ctx
                .get_file_system()
                .path_exists(&relation.get_files().get_ref_housenumbers_path())
        {
            continue;
        }
        let references = ctx.get_ini().get_reference_housenumber_paths()?;
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "only" {
            continue;
        }

        info!("update_ref_housenumbers: start: {relation_name}");
        relation.write_ref_housenumbers(&references)?;
        info!("update_ref_housenumbers: end: {relation_name}");
    }

    Ok(())
}

/// Update the relation's house number coverage stats.
fn update_missing_housenumbers(
    relations: &mut areas::Relations<'_>,
    update: bool,
) -> anyhow::Result<()> {
    info!("update_missing_housenumbers: start");
    let active_names = relations
        .get_active_names()
        .context("get_active_names() failed")?;
    for relation_name in active_names {
        let mut relation = relations
            .get_relation(&relation_name)
            .context("get_relation() failed")?;
        if !update && relation.has_osm_housenumber_coverage()? {
            continue;
        }
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "only" {
            continue;
        }

        relation
            .write_missing_housenumbers()
            .context("write_missing_housenumbers() failed")?;
    }
    info!("update_missing_housenumbers: end");

    Ok(())
}

/// Update the relation's street coverage stats.
fn update_missing_streets(
    relations: &mut areas::Relations<'_>,
    update: bool,
) -> anyhow::Result<()> {
    info!("update_missing_streets: start");
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update && relation.has_osm_street_coverage()? {
            continue;
        }
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "no" {
            continue;
        }

        relation.write_missing_streets()?;
    }
    info!("update_missing_streets: end");

    Ok(())
}

/// Update the relation's "additional streets" stats.
fn update_additional_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    update: bool,
) -> anyhow::Result<()> {
    info!("update_additional_streets: start");
    for relation_name in relations.get_active_names()? {
        let relation = relations.get_relation(&relation_name)?;
        if !update && stats::has_sql_count(ctx, "additional_streets_counts", &relation_name)? {
            continue;
        }
        let streets = relation.get_config().should_check_missing_streets();
        if streets == "no" {
            continue;
        }

        relation.write_additional_streets()?;
    }
    info!("update_additional_streets: end");

    Ok(())
}

/// Writes a daily citycount rows into the stats_citycounts SQL table.
fn write_city_count_path(
    ctx: &context::Context,
    cities: &HashMap<String, HashSet<String>>,
) -> anyhow::Result<()> {
    let mut cities: Vec<_> = cities.iter().collect();
    // Locale-aware sort, by key.
    cities.sort_by_key(|(key, _value)| util::get_sort_key(key));
    cities.dedup();
    let mut conn = ctx.get_database_connection()?;
    let tx = conn.transaction()?;
    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;
    for (key, value) in cities {
        tx.execute(
            r#"insert into stats_citycounts (date, city, count) values (?1, ?2, ?3)
            on conflict(date, city) do update set count = excluded.count"#,
            [&today, key, &value.len().to_string()],
        )?;
    }
    Ok(tx.commit()?)
}

/// Writes daily zipcount rows into the stats_zipcounts SQL table.
fn write_zip_count_path(
    ctx: &context::Context,
    zips: &HashMap<String, HashSet<String>>,
) -> anyhow::Result<()> {
    let mut zips: Vec<_> = zips.iter().collect();
    zips.sort_by_key(|(key, _value)| key.to_string());
    zips.dedup();

    let mut conn = ctx.get_database_connection()?;
    let tx = conn.transaction()?;
    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;
    for (key, value) in zips {
        tx.execute(
            r#"insert into stats_zipcounts (date, zip, count) values (?1, ?2, ?3)
            on conflict(date, zip) do update set count = excluded.count"#,
            [&today, key, &value.len().to_string()],
        )?;
    }
    Ok(tx.commit()?)
}

/// Counts the # of all house numbers as of today.
fn update_stats_count(ctx: &context::Context, today: &str) -> anyhow::Result<()> {
    let statedir = ctx.get_abspath("workdir/stats");
    let csv_path = format!("{statedir}/whole-country.csv");
    if !ctx.get_file_system().path_exists(&csv_path) {
        return Ok(());
    }
    let mut house_numbers: HashSet<String> = HashSet::new();
    let mut cities: HashMap<String, HashSet<String>> = HashMap::new();
    let mut zips: HashMap<String, HashSet<String>> = HashMap::new();
    let valid_settlements =
        util::get_valid_settlements(ctx).context("get_valid_settlements() failed")?;
    let stream = ctx.get_file_system().open_read(&csv_path)?;
    let mut guard = stream.borrow_mut();
    let mut read = std::io::BufReader::new(guard.deref_mut());
    let mut csv_reader = util::make_csv_reader(&mut read);
    for result in csv_reader.deserialize() {
        let row: util::OsmLightHouseNumber = result?;
        // This ignores the @user column.
        house_numbers.insert(
            [
                row.postcode.to_string(),
                row.city.to_string(),
                row.street.to_string(),
                row.housenumber.to_string(),
            ]
            .join("\t"),
        );
        let city_key = util::get_city_key(&row.postcode, &row.city, &valid_settlements)
            .context("get_city_key() failed")?;
        let city_value = [row.street.to_string(), row.housenumber.to_string()].join("\t");
        let entry = cities.entry(city_key).or_default();
        entry.insert(city_value);

        // Postcode.
        let zip_key = row.postcode.to_string();
        // Street name and housenumber.
        let zip_value = [row.street, row.housenumber].join("\t");
        let zip_entry = zips.entry(zip_key).or_default();
        zip_entry.insert(zip_value);
    }

    {
        let mut conn = ctx.get_database_connection()?;
        let tx = conn.transaction()?;
        tx.execute(
            r#"insert into stats_counts (date, count) values (?1, ?2)
               on conflict(date) do update set count = excluded.count"#,
            [today, &house_numbers.len().to_string()],
        )?;
        tx.commit()?;
    }

    write_city_count_path(ctx, &cities).context("write_city_count_path() failed")?;
    write_zip_count_path(ctx, &zips).context("write_zip_count_path() failed")
}

/// Counts the top housenumber editors as of today.
fn update_stats_topusers(ctx: &context::Context, today: &str) -> anyhow::Result<()> {
    let statedir = ctx.get_abspath("workdir/stats");
    let csv_path = format!("{statedir}/whole-country.csv");
    if !ctx.get_file_system().path_exists(&csv_path) {
        return Ok(());
    }
    let mut users: HashMap<String, u64> = HashMap::new();
    {
        let stream = ctx.get_file_system().open_read(&csv_path)?;
        let mut guard = stream.borrow_mut();
        let mut read = std::io::BufReader::new(guard.deref_mut());
        let mut csv_reader = util::make_csv_reader(&mut read);
        for result in csv_reader.deserialize() {
            let row: util::OsmLightHouseNumber = result?;
            // Only care about the last column.
            let user = row.user.to_string();
            let entry = users.entry(user).or_insert(0);
            (*entry) += 1;
        }
    }
    {
        let mut users: Vec<_> = users.iter().collect();
        users.sort_by_key(|i| Reverse(i.1));
        users.dedup();
        users = users[0..std::cmp::min(20, users.len())].to_vec();
        let mut conn = ctx.get_database_connection()?;
        let tx = conn.transaction()?;
        let now = ctx.get_time().now();
        let format = time::format_description::parse("[year]-[month]-[day]")?;
        let today = now.format(&format)?;
        for user in &users {
            tx.execute(
                r#"insert into stats_topusers (date, user, count) values (?1, ?2, ?3)
            on conflict(date, user) do update set count = excluded.count"#,
                [&today, user.0, &user.1.to_string()],
            )?;
        }
        tx.commit()?;
    }

    let mut conn = ctx.get_database_connection()?;
    let tx = conn.transaction()?;
    tx.execute(
        r#"insert into stats_usercounts (date, count) values (?1, ?2)
               on conflict(date) do update set count = excluded.count"#,
        [today, &users.len().to_string()],
    )?;
    Ok(tx.commit()?)
}

/// Performs the update of workdir/stats/ref.count.
fn update_stats_refcount(ctx: &context::Context, state_dir: &str) -> anyhow::Result<()> {
    let mut count = 0;
    {
        let stream = ctx
            .get_file_system()
            .open_read(&ctx.get_ini().get_reference_citycounts_path()?)?;
        let mut guard = stream.borrow_mut();
        let mut read = guard.deref_mut();
        let mut csv_reader = util::make_csv_reader(&mut read);
        for result in csv_reader.deserialize() {
            let row: util::CityCount = result?;

            count += row.count;
        }
    }

    let string = format!("{count}\n");
    let path = format!("{state_dir}/ref.count");
    ctx.get_file_system().write_from_string(&string, &path)
}

/// Performs the update of whole-country.csv.
pub fn update_stats_overpass(ctx: &context::Context) -> anyhow::Result<()> {
    // Old style: CSV.
    let query = ctx
        .get_file_system()
        .read_to_string(&ctx.get_abspath("data/street-housenumbers-hungary.overpassql"))?;
    let statedir = ctx.get_abspath("workdir/stats");
    let csv_path = format!("{statedir}/whole-country.csv");
    info!("update_stats_overpass: talking to overpass");
    let mut retry = 0;
    while should_retry(retry) {
        if retry > 0 {
            info!("update_stats_overpass: try #{retry}");
        }
        retry += 1;
        overpass_sleep(ctx);
        let response = match overpass_query::overpass_query(ctx, &query) {
            Ok(value) => value,
            Err(err) => {
                info!("update_stats_overpass: http error: {err}");
                continue;
            }
        };
        ctx.get_file_system()
            .write_from_string(&response, &csv_path)?;
        break;
    }

    // New style: JSON.
    let mut i = 0;
    let mut lines = Vec::new();
    for line in query.lines() {
        i += 1;
        if i == 1 {
            lines.push("[out:json]  [timeout:425];".to_string());
            continue;
        }

        lines.push(line.to_string());
    }
    let json_query = lines.join("\n");
    info!("update_stats_overpass: json, talking to overpass");
    let mut retry = 0;
    while should_retry(retry) {
        if retry > 0 {
            info!("update_stats_overpass: try #{retry}");
        }
        retry += 1;
        overpass_sleep(ctx);
        let response = match overpass_query::overpass_query(ctx, &json_query) {
            Ok(value) => value,
            Err(err) => {
                info!("update_stats_overpass: http error: {err}");
                continue;
            }
        };

        area_files::write_whole_country(ctx, &response)?;
        break;
    }
    Ok(())
}

/// Performs the update of country-level stats.
fn update_stats(ctx: &context::Context, overpass: bool) -> anyhow::Result<()> {
    // Fetch house numbers for the whole country.
    info!("update_stats: start, updating whole-country csv");
    let statedir = ctx.get_abspath("workdir/stats");
    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;

    if overpass {
        update_stats_overpass(ctx)?;
    }

    info!("update_stats: updating count");
    update_stats_count(ctx, &today).context("update_stats_count() failed")?;
    info!("update_stats: updating topusers");
    update_stats_topusers(ctx, &today)?;
    info!("update_stats: updating refcount");
    update_stats_refcount(ctx, &statedir)?;
    stats::update_invalid_addr_cities(ctx, &statedir)?;

    info!("update_stats: generating json");
    let json_path = format!("{}/stats.json", &statedir);
    stats::generate_json(ctx, &statedir, &json_path).context("generate_json() failed")?;

    info!("update_stats: end");

    Ok(())
}

/// Performs the actual nightly task.
fn our_main_inner(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    mode: &String,
    update: bool,
    overpass: bool,
) -> anyhow::Result<()> {
    if mode == "all" || mode == "stats" {
        update_stats(ctx, overpass).context("update_stats failed")?;
    }
    if mode == "all" || mode == "relations" {
        update_osm_streets(ctx, relations, update)?;
        update_osm_housenumbers(ctx, relations, update)?;
        update_ref_housenumbers(ctx, relations, update)?;
        update_missing_streets(relations, update)?;
        update_missing_housenumbers(relations, update)?;
        update_additional_streets(ctx, relations, update)?;
    }

    let pid = std::process::id();
    let stream = std::fs::File::open(format!("/proc/{pid}/status"))?;
    let reader = std::io::BufReader::new(stream);
    for line in reader.lines() {
        let line = line?.to_string();
        if line.starts_with("VmPeak:") {
            let vm_peak = line.trim();
            info!("our_main: {vm_peak}");
            break;
        }
    }

    ctx.get_unit().make_error()
}

/// Inner main() that is allowed to fail.
pub fn our_main(
    argv: &[String],
    _stream: &mut dyn Write,
    ctx: &context::Context,
) -> anyhow::Result<()> {
    let mut relations = areas::Relations::new(ctx)?;

    let refcounty = clap::Arg::new("refcounty")
        .long("refcounty")
        .help("limit the list of relations to a given refcounty");
    let refsettlement = clap::Arg::new("refsettlement")
        .long("refsettlement")
        .help("limit the list of relations to a given refsettlement");
    let refarea = clap::Arg::new("refarea")
        .long("refarea")
        .help("limit the list of relations to a given area name");
    // Default: true.
    let no_update = clap::Arg::new("no-update")
        .long("no-update")
        .action(clap::ArgAction::SetTrue)
        .help("don't update existing state of relations");
    let mode = clap::Arg::new("mode")
        .long("mode")
        .default_value("relations")
        .help("only perform the given sub-task or all of them [all, stats or relations]");
    let no_overpass = clap::Arg::new("no-overpass") // default: true
        .long("no-overpass")
        .action(clap::ArgAction::SetTrue)
        .help("when updating stats, don't perform any overpass update");
    let args = [
        refcounty,
        refsettlement,
        refarea,
        no_update,
        mode,
        no_overpass,
    ];
    let app = clap::Command::new("osm-gimmisn");
    let args = app.args(&args).try_get_matches_from(argv)?;

    let start = ctx.get_time().now();
    // Query inactive relations once a month.
    let now = ctx.get_time().now();
    let first_day_of_month = now.date().day() == 1;
    relations.activate_all(ctx.get_ini().get_cron_update_inactive() || first_day_of_month);
    relations.activate_new();
    relations.activate_invalid();
    let refcounty: Option<&String> = args.get_one("refcounty");
    relations.limit_to_refcounty(&refcounty)?;
    // Use map(), which handles optional values.
    let refsettlement: Option<&String> = args.get_one("refsettlement");
    relations.limit_to_refsettlement(&refsettlement)?;
    let refarea: Option<&String> = args.get_one("refarea");
    relations.limit_to_refarea(&refarea)?;
    let update = !args.get_one::<bool>("no-update").unwrap();
    let overpass = !args.get_one::<bool>("no-overpass").unwrap();
    our_main_inner(
        ctx,
        &mut relations,
        args.get_one("mode").unwrap(),
        update,
        overpass,
    )
    .context("our_main_inner failed")?;
    let duration = ctx.get_time().now() - start;
    let seconds = duration.whole_seconds() % 60;
    let minutes = duration.whole_minutes() % 60;
    let hours = duration.whole_hours();
    let duration = format!("{hours}:{minutes:0>2}:{seconds:0>2}");
    info!("main: finished in {duration}");

    Ok(())
}

/// Similar to plain main(), but with an interface that allows testing.
pub fn main(argv: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    match our_main(argv, stream, ctx) {
        Ok(_) => 0,
        Err(err) => {
            error!("main: unhandled error: {err:?}");
            1
        }
    }
}

#[cfg(test)]
mod tests;
