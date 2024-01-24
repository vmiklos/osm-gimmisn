/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Parses the Apache access log of osm-gimmisn for 1 month.

use anyhow::Context;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufRead;
use std::io::Write;

use crate::areas;
use crate::context;
use crate::stats;

/// Does this relation have 100% house number coverage?
fn is_complete_relation(
    relations: &mut areas::Relations<'_>,
    relation_name: &str,
) -> anyhow::Result<bool> {
    let relation = relations.get_relation(relation_name)?;
    if !relation.has_osm_housenumber_coverage()? {
        return Ok(false);
    }

    Ok(relation.get_osm_housenumber_coverage()? == "100.00")
}

/// Determine if 'line' has a user agent which looks like a search bot.
fn is_search_bot(line: &str) -> bool {
    let search_bots = vec![
        "AhrefsBot",
        "AhrefsBot",
        "CCBot",
        "Googlebot",
        "SemrushBot",
        "YandexBot",
        "bingbot",
    ];
    for search_bot in search_bots {
        if line.contains(search_bot) {
            return true;
        }
    }

    false
}

/// Determine the top 20%: set of frequently visited relations.
fn get_frequent_relations(
    ctx: &context::Context,
    log_file: &str,
) -> anyhow::Result<HashSet<String>> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    let log_stream = std::io::BufReader::new(std::fs::File::open(log_file)?);
    // Example line:
    // a.b.c.d - - [01/Jul/2020:00:08:01 +0200] "GET /osm/street-housenumbers/budapest_12/update-result HTTP/1.1" 200 1747 "-" "Mozilla/5.0 ..."
    for line in log_stream.lines() {
        let line = line?;
        if is_search_bot(&line) {
            continue;
        }
        let regex = regex::Regex::new(".*\"GET ([^ ]+) .*")?;
        let mut captures_iter = regex.captures_iter(&line);
        let group = captures_iter.next();
        if group.is_none() {
            // Not GET.
            continue;
        }
        let request_uri = &group.unwrap()[1];
        if !request_uri.starts_with("/osm") {
            continue;
        }

        // Expect: /osm/missing-streets/budapest_01/view-turbo
        let tokens: Vec<String> = request_uri.split('/').map(|i| i.to_string()).collect();
        if tokens.len() != 5 {
            continue;
        }
        let relation_name = tokens[3].to_string();
        let entry = counts.entry(relation_name).or_insert(0);
        (*entry) += 1;
    }
    let mut count_list: Vec<_> = counts.iter().collect();
    // Reverse, by value.
    count_list.sort_by(|a, b| b.1.cmp(a.1));

    // Dump relations and their visit count to workdir for further inspection.
    let csv_stream = ctx.get_file_system().open_write(&format!(
        "{}/frequent-relations.csv",
        ctx.get_ini().get_workdir()
    ))?;
    let mut guard = csv_stream.borrow_mut();
    for item in count_list.iter() {
        guard
            .write_all(format!("{}\t{}\n", item.0, item.1).as_bytes())
            .context("write_all() failed")?;
    }

    let relation_count = count_list.len() as f64;
    let frequent_count = (relation_count * 0.2).round() as usize;
    let count_list = &count_list[..frequent_count];
    let frequent_relations: HashSet<String> = count_list.iter().map(|i| i.0.to_string()).collect();
    Ok(frequent_relations)
}

/// Builds a name -> create_date dictionary for relations.
fn get_relation_create_dates(
    ctx: &context::Context,
) -> anyhow::Result<HashMap<String, time::OffsetDateTime>> {
    let mut ret: HashMap<String, time::OffsetDateTime> = HashMap::new();
    let relations_path = ctx.get_abspath("data/relations.yaml");
    let process_stdout = ctx.get_subprocess().run(vec![
        "git".into(),
        "blame".into(),
        "--line-porcelain".into(),
        relations_path,
    ])?;
    let mut timestamp = 0_i64;

    for line in process_stdout.lines() {
        let regex = regex::Regex::new("\t([^ :]+):")?;
        let mut captures_iter = regex.captures_iter(line);
        let group = captures_iter.next();
        if let Some(matches) = group {
            let name = &matches[1];
            ret.insert(
                name.to_string(),
                time::OffsetDateTime::from_unix_timestamp(timestamp).unwrap(),
            );
            continue;
        }

        let author_regex = regex::Regex::new("author-time ([0-9]+)")?;
        let mut captures_iter = author_regex.captures_iter(line);
        let group = captures_iter.next();
        if let Some(matches) = group {
            timestamp = matches[1].parse()?;
        }
    }

    Ok(ret)
}

/// Decides if the given relation is recent, based on create_dates.
fn is_relation_recently_added(
    ctx: &context::Context,
    create_dates: &HashMap<String, time::OffsetDateTime>,
    name: &str,
) -> bool {
    let today = ctx.get_time().now();
    let month_ago = today - time::Duration::days(30);
    create_dates.contains_key(name) && create_dates[name] > month_ago
}

/// Update frequent_relations based on get_topcities():
/// 1) The top 5 edited cities count as frequent, even if they have ~no visitors.
/// 2) If a relation got <5 house numbers in the last 30 days, then they are not frequent, even with
/// lots of visitors.
fn check_top_edited_relations(
    ctx: &context::Context,
    frequent_relations: &mut HashSet<String>,
) -> anyhow::Result<()> {
    let workdir = ctx.get_ini().get_workdir();
    // List of 'city name' <-> '# of new house numbers' pairs.
    let topcities = stats::get_topcities(ctx, &format!("{workdir}/stats"))?;
    let topcities: Vec<_> = topcities
        .iter()
        .map(|city| (unidecode::unidecode(&city.0), city.1))
        .collect();
    // Top 5: these should be frequent.
    for city in &topcities[..std::cmp::min(topcities.len(), 5)] {
        frequent_relations.insert(city.0.clone());
    }
    // Bottom: anything with <5 new house numbers is not frequent.
    let bottomcities: Vec<_> = topcities.iter().filter(|city| city.1 < 5).collect();
    for city in bottomcities {
        if frequent_relations.contains(&city.0) {
            frequent_relations.remove(&city.0);
        }
    }

    Ok(())
}

/// Inner main() that is allowed to fail.
pub fn our_main(
    argv: &[String],
    stdout: &mut dyn Write,
    ctx: &context::Context,
) -> anyhow::Result<()> {
    if argv.len() < 2 {
        return Err(anyhow::anyhow!("missing parameter: logfile"));
    }

    let log_file = &argv[1];

    let relation_create_dates = get_relation_create_dates(ctx)?;

    let mut relations = areas::Relations::new(ctx)?;
    let mut frequent_relations = get_frequent_relations(ctx, log_file)?;
    check_top_edited_relations(ctx, &mut frequent_relations)?;

    // Now suggest what to change.
    let mut removals = 0;
    let mut additions = 0;
    for relation_name in relations.get_names() {
        let relation = relations.get_relation(&relation_name)?;
        let actual = relation.get_config().is_active();
        let expected = frequent_relations.contains(&relation_name)
            && !is_complete_relation(&mut relations, &relation_name)?;
        if actual != expected {
            if actual {
                if !is_relation_recently_added(ctx, &relation_create_dates, &relation_name) {
                    stdout.write_all(
                        format!("data/relation-{relation_name}.yaml: set inactive: true\n")
                            .as_bytes(),
                    )?;
                    removals += 1;
                }
            } else {
                stdout.write_all(
                    format!("data/relation-{relation_name}.yaml: set inactive: false\n").as_bytes(),
                )?;
                additions += 1;
            }
        }
    }
    stdout.write_all(
        format!("Suggested {removals} removals and {additions} additions.\n").as_bytes(),
    )?;

    ctx.get_unit().make_error()
}

/// Similar to plain main(), but with an interface that allows testing.
pub fn main(argv: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    match our_main(argv, stream, ctx) {
        Ok(_) => 0,
        Err(err) => {
            stream.write_all(format!("{err:?}\n").as_bytes()).unwrap();
            1
        }
    }
}

#[cfg(test)]
mod tests;
