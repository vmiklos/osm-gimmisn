/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
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
use crate::util;

/// Does this relation have 100% house number coverage?
fn is_complete_relation(
    relations: &mut areas::Relations,
    relation_name: &str,
) -> anyhow::Result<bool> {
    let relation = relations.get_relation(relation_name)?;
    if !std::path::Path::new(&relation.get_files().get_housenumbers_percent_path()).exists() {
        return Ok(false);
    }

    let percent = String::from_utf8(util::get_content(
        &relation.get_files().get_housenumbers_percent_path(),
    )?)?;
    Ok(percent == "100.00")
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
    let mut count_list: Vec<_> = counts.iter().map(|(key, value)| (key, value)).collect();
    // Reverse, by value.
    count_list.sort_by(|a, b| b.1.cmp(a.1));

    // Dump relations and their visit count to workdir for further inspection.
    let mut csv_stream = std::fs::File::create(format!(
        "{}/frequent-relations.csv",
        ctx.get_ini().get_workdir()?
    ))?;
    for item in count_list.iter() {
        csv_stream
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
) -> anyhow::Result<HashMap<String, chrono::NaiveDateTime>> {
    let mut ret: HashMap<String, chrono::NaiveDateTime> = HashMap::new();
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
                chrono::NaiveDateTime::from_timestamp(timestamp, 0),
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
    create_dates: &HashMap<String, chrono::NaiveDateTime>,
    name: &str,
) -> bool {
    let today = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    let month_ago = today - chrono::Duration::days(30);
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
    let workdir = ctx.get_ini().get_workdir()?;
    // List of 'city name' <-> '# of new house numbers' pairs.
    let topcities = stats::get_topcities(ctx, &format!("{}/stats", workdir))?;
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

/// Commandline interface.
pub fn main(argv: &[String], stdout: &mut dyn Write, ctx: &context::Context) -> anyhow::Result<()> {
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
                        format!("data/relation-{}.yaml: set inactive: true\n", relation_name)
                            .as_bytes(),
                    )?;
                    removals += 1;
                }
            } else {
                stdout.write_all(
                    format!(
                        "data/relation-{}.yaml: set inactive: false\n",
                        relation_name
                    )
                    .as_bytes(),
                )?;
                additions += 1;
            }
        }
    }
    stdout.write_all(
        format!(
            "Suggested {} removals and {} additions.\n",
            removals, additions
        )
        .as_bytes(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Read;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::sync::Arc;

    /// Tests check_top_edited_relations().
    #[test]
    fn test_check_top_edited_relations() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let time = context::tests::make_test_time();
        let time_arc: Arc<dyn context::Time> = Arc::new(time);
        ctx.set_time(&time_arc);
        let old_citycount = b"foo\t0\n\
city1\t0\n\
city2\t0\n\
city3\t0\n\
city4\t0\n\
bar\t0\n\
baz\t0\n";
        let old_citycount_value = context::tests::TestFileSystem::make_file();
        old_citycount_value
            .borrow_mut()
            .write_all(old_citycount)
            .unwrap();
        let new_citycount = b"foo\t1000\n\
city1\t1000\n\
city2\t1000\n\
city3\t1000\n\
city4\t1000\n\
bar\t2\n\
baz\t2\n";
        let new_citycount_value = context::tests::TestFileSystem::make_file();
        new_citycount_value
            .borrow_mut()
            .write_all(new_citycount)
            .unwrap();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[
                ("workdir/stats/2020-04-10.citycount", &old_citycount_value),
                ("workdir/stats/2020-05-10.citycount", &new_citycount_value),
            ],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        ctx.set_file_system(&file_system);

        let mut frequent_relations: HashSet<String> = ["foo".to_string(), "bar".to_string()]
            .iter()
            .cloned()
            .collect();
        check_top_edited_relations(&ctx, &mut frequent_relations).unwrap();

        assert_eq!(frequent_relations.contains("foo"), true);
        assert_eq!(frequent_relations.contains("city1"), true);
        assert_eq!(frequent_relations.contains("city2"), true);
        assert_eq!(frequent_relations.contains("city3"), true);
        assert_eq!(frequent_relations.contains("city4"), true);
        assert_eq!(frequent_relations.contains("bar"), false);
        assert_eq!(frequent_relations.contains("baz"), false);
    }

    /// Tests is_complete_relation().
    #[test]
    fn test_is_complete_relation() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = areas::Relations::new(&ctx).unwrap();
        assert_eq!(
            is_complete_relation(&mut relations, "gazdagret").unwrap(),
            false
        );
    }

    /// Tests main().
    #[test]
    fn test_main() {
        let argv = ["".to_string(), "tests/mock/access_log".to_string()];
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let mut ctx = context::tests::make_test_context().unwrap();
        let time = context::tests::make_test_time();
        let time_arc: Arc<dyn context::Time> = Arc::new(time);
        ctx.set_time(&time_arc);
        let relations_path = ctx.get_abspath("data/relations.yaml");
        // 2020-05-09, so this will be recent
        let expected_args = format!("git blame --line-porcelain {}", relations_path);
        let expected_out = "\n\
author-time 1588975200\n\
\tujbuda:\n"
            .to_string();
        let outputs: HashMap<_, _> = vec![(expected_args, expected_out)].into_iter().collect();
        let subprocess = context::tests::TestSubprocess::new(&outputs);
        let subprocess_arc: Arc<dyn context::Subprocess> = Arc::new(subprocess);
        ctx.set_subprocess(&subprocess_arc);

        main(&argv, &mut buf, &ctx).unwrap();

        buf.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        buf.read_to_end(&mut actual).unwrap();
        let actual = String::from_utf8(actual).unwrap();
        assert_eq!(
            actual.contains("data/relation-inactiverelation.yaml: set inactive: false\n"),
            true
        );
        assert_eq!(
            actual.contains("data/relation-gazdagret.yaml: set inactive: true\n"),
            true
        );
        assert_eq!(
            actual.contains("data/relation-nosuchrelation.yaml: set inactive: "),
            false
        );

        // This is not in the output because it's considered as a recent relation.
        assert_eq!(
            actual.contains("data/relation-ujbuda.yaml: set inactive: "),
            false
        );

        // This is not in the output as it's not a valid relation name.
        assert_eq!(actual.contains("budafokxxx"), false);

        // This is not in the output as it's a search bot, so such visits don't count.
        // Also, if this would be not ignored, it would push 'inactiverelation' out of the active
        // relation list.
        assert_eq!(actual.contains("gyomaendrod"), false);
    }
}
