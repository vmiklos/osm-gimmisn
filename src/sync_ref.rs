/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Synchronizes reference data between a public instance and a local dev instance.

use crate::context;
use crate::util;
use anyhow::Context as _;
use std::collections::HashMap;
use std::io::Write;

/// Synchronizes reference data based on config_file from url.
pub fn download(
    stream: &mut dyn Write,
    ctx: &context::Context,
    config_file: &str,
    url: &str,
) -> anyhow::Result<()> {
    let config_data = ctx.get_file_system().read_to_string(config_file)?;
    let config: context::IniConfig = toml::from_str(&config_data)?;
    let mut paths: Vec<String> = Vec::new();
    let values = config.wsgi.reference_housenumbers;
    paths.append(
        &mut values
            .split(' ')
            .map(|value| value.strip_prefix("workdir/refs/").unwrap().to_string())
            .collect(),
    );
    let value = config.wsgi.reference_street;
    paths.push(value.strip_prefix("workdir/refs/").unwrap().to_string());
    let value = config.wsgi.reference_citycounts;
    paths.push(value.strip_prefix("workdir/refs/").unwrap().to_string());
    let value = config.wsgi.reference_zipcounts;
    paths.push(value.strip_prefix("workdir/refs/").unwrap().to_string());

    let mut dests: Vec<String> = Vec::new();
    for path in &paths {
        let url = format!("{url}{path}");
        let dest = ctx.get_abspath(&format!("workdir/refs/{path}"));
        dests.push(dest.to_string());
        if ctx.get_file_system().path_exists(&dest) {
            continue;
        }

        stream.write_all(format!("sync-ref: downloading '{url}'...\n").as_bytes())?;
        let buf = ctx.get_network().urlopen(&url, "")?;
        ctx.get_file_system().write_from_string(&buf, &dest)?;
    }
    for path in ctx
        .get_file_system()
        .listdir(&ctx.get_abspath("workdir/refs"))?
    {
        if dests.contains(&path) {
            continue;
        }
        let relpath = path.strip_prefix(&ctx.get_abspath("")).unwrap();
        stream.write_all(format!("sync-ref: removing '{relpath}'...\n").as_bytes())?;
        ctx.get_file_system().unlink(&path)?;
    }

    stream.write_all("sync-ref: creating index...\n".as_bytes())?;
    let mut conn = ctx.get_database().create()?;
    conn.execute("delete from ref_housenumbers", [])?;
    util::build_reference_index(ctx, &mut conn, &paths)?;

    ctx.get_file_system()
        .write_from_string(&config_data, &ctx.get_abspath("workdir/wsgi.ini"))?;
    stream.write_all("sync-ref: ok\n".as_bytes())?;
    Ok(())
}

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
    let mode = clap::Arg::new("mode")
        .long("mode")
        .default_value("config")
        .help("update the config or download based on config [config or download]");
    let args = [url, mode];
    let app = clap::Command::new("osm-gimmisn").override_usage(
        "osm-gimmisn sync-ref [--mode download] --url https://www.example.com/osm/data/",
    );
    let args = app.args(&args).try_get_matches_from(argv)?;
    let url = args
        .get_one::<String>("url")
        .context("missing url")?
        .to_string();

    let config_file = ctx.get_abspath("data/wsgi.ini.template");
    if args.get_one::<String>("mode").unwrap() == "download" {
        return download(stream, ctx, &config_file, &url);
    }

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
        "reference_housenumbers = 'workdir/refs/hazszamok_{}.tsv workdir/refs/hazszamok_kieg_{}.tsv'",
        files["hazszamok"], files["hazszamok_kieg"]
    ));
    config.push(format!(
        "reference_street = 'workdir/refs/utcak_{}.tsv'",
        files["utcak"]
    ));
    config.push(format!(
        "reference_citycounts = 'workdir/refs/varosok_count_{}.tsv'",
        files["varosok_count"]
    ));
    config.push(format!(
        "reference_zipcounts = 'workdir/refs/irsz_count_{}.tsv'",
        files["irsz_count"]
    ));
    config.push(String::new());

    // Write config.
    ctx.get_file_system()
        .write_from_string(&config.join("\n"), &config_file)?;
    let max = files.values().max().context("empty files")?;
    stream.write_all(
        format!("Now you can run: git commit -m 'Update reference to {max}'\n").as_bytes(),
    )?;
    Ok(())
}

/// This handles the update of data/wsgi.ini.template, or download based on that.
/// Similar to plain main(), but with an interface that allows testing.
pub fn main(args: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    match our_main(args, stream, ctx) {
        Ok(_) => 0,
        Err(err) => {
            stream.write_all(format!("{err:?}\n").as_bytes()).unwrap();
            1
        }
    }
}

#[cfg(test)]
mod tests;
