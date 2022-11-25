/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Synchronizes reference data between a public instance and a local dev instance.

use crate::context;
use anyhow::Context as _;
use std::collections::HashMap;
use std::io::Write;

/// Inner main() that is allowed to fail.
pub fn our_main(
    argv: &[String],
    stream: &mut dyn Write,
    ctx: &context::Context,
) -> anyhow::Result<()> {
    let url = clap::Arg::new("url")
        .long("url")
        .required(true)
        .help("public instance URL");
    let args = [url];
    let app = clap::Command::new("osm-gimmisn")
        .override_usage("osm-gimmisn sync-ref --url https://www.example.com/osm/data/");
    let args = app.args(&args).try_get_matches_from(argv)?;
    let url = args
        .get_one::<String>("url")
        .context("missing url")?
        .to_string();

    // Download HTML.
    let html = ctx.get_network().urlopen(&url, "")?;

    // Parse the HTML.
    let dom = html_parser::Dom::parse(&html)?;
    let mut dom_iter = dom.children.iter();
    let mut root = dom_iter
        .next()
        .context("failed to get first child of dom")?;
    if root.text().is_some() {
        // Skip a first-line comment before the real root.
        root = dom_iter.next().unwrap();
    }
    let root = root.into_iter();

    // The format is type_date.tsv, figure out the latest date for each type.
    let mut files: HashMap<String, u64> = HashMap::new();
    for node in root {
        let element = match node.element() {
            Some(value) => value,
            None => {
                continue;
            }
        };
        if element.name != "a" {
            continue;
        }

        let href_value = element.attributes.get("href").unwrap().clone();
        let mut href = href_value.unwrap();
        href = match href.strip_suffix(".tsv") {
            Some(value) => value.into(),
            None => {
                // Does not end with ".tsv".
                continue;
            }
        };
        let tokens: Vec<&str> = href.split('_').collect();
        let file: String = tokens[0..tokens.len() - 1].join("_");
        let href_date: u64 = tokens[tokens.len() - 1].parse()?;
        files
            .entry(file)
            .and_modify(|date| *date = std::cmp::max(*date, href_date))
            .or_insert(href_date);
    }

    // Generate config.
    let mut config: Vec<String> = Vec::new();
    config.push("[wsgi]".into());
    config.push(format!(
        "reference_housenumbers = refdir/hazszamok_{}.tsv refdir/hazszamok_kieg_{}.tsv",
        files["hazszamok"], files["hazszamok_kieg"]
    ));
    config.push(format!(
        "reference_street = refdir/utcak_{}.tsv",
        files["utcak"]
    ));
    config.push(format!(
        "reference_citycounts = refdir/varosok_count_{}.tsv",
        files["varosok_count"]
    ));
    config.push(format!(
        "reference_zipcounts = refdir/irsz_count_{}.tsv",
        files["irsz_count"]
    ));
    config.push(String::new());

    // Write config.
    let config_file = ctx.get_abspath("data/wsgi.ini.template");
    ctx.get_file_system()
        .write_from_string(&config.join("\n"), &config_file)?;
    let max = files.iter().map(|(_k, v)| v).max().context("empty files")?;
    stream.write_all(
        format!(
            "Now you can run: git commit -m 'Update reference to {}'\n",
            max
        )
        .as_bytes(),
    )?;
    Ok(())
}

/// This handles the update of data/wsgi.ini.template, tools/sync-ref.sh has to be still invoked
/// after this.
/// TODO merge tools/sync-ref.sh into this function.
/// Similar to plain main(), but with an interface that allows testing.
pub fn main(args: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    match our_main(args, stream, ctx) {
        Ok(_) => 0,
        Err(err) => {
            stream.write_all(format!("{:?}\n", err).as_bytes()).unwrap();
            1
        }
    }
}

#[cfg(test)]
mod tests;
