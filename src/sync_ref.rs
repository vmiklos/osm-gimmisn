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
use std::collections::HashMap;
use std::io::Write;

/// This handles the update of data/wsgi.ini.template, tools/sync-ref.sh has to be still invoked
/// after this.
/// TODO merge tools/sync-ref.sh into this function.
pub fn main(args: &[String], stream: &mut dyn Write, ctx: &context::Context) -> i32 {
    // Download HTML.
    let mut args_iter = args.iter();
    let _self = args_iter.next();
    let url = match args_iter.next() {
        Some(s) => s,
        None => {
            stream
                .write_all(b"usage: osm-gimmisn sync-ref https://www.example.com/osm/data/")
                .unwrap();
            return 1;
        }
    };
    // let html = std::fs::read_to_string("osm-data.html").unwrap();
    let html = ctx.get_network().urlopen(url, "").unwrap();

    // Parse the HTML.
    let dom = html_parser::Dom::parse(&html).unwrap();
    let mut dom_iter = dom.children.iter();
    let mut root = dom_iter.next().unwrap();
    if root.text().is_some() {
        // Skip a first-line comment before the real root.
        root = dom_iter.next().unwrap();
    }
    let root = root.into_iter();

    // The format is type_date.tsv, figure out the latest date for each type.
    let mut files: HashMap<String, u64> = HashMap::new();
    for node in root {
        if node.element().is_none() {
            continue;
        }

        let element = node.element().unwrap();
        if element.name != "a" {
            continue;
        }

        let href_value: Option<String> = element.attributes.get("href").unwrap().clone();
        let mut href = href_value.unwrap();
        if !href.ends_with(".tsv") {
            continue;
        }

        href = href.strip_suffix(".tsv").unwrap().into();
        let tokens: Vec<&str> = href.split('_').collect();
        let file: String = tokens[0..tokens.len() - 1].join("_");
        let href_date: u64 = tokens[tokens.len() - 1].parse().unwrap();
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
        .write_from_string(&config.join("\n"), &config_file)
        .unwrap();
    let max = files.iter().map(|(_k, v)| v).max().unwrap();
    println!(
        "Now you can run: git commit -m 'Update reference to {}'",
        max
    );
    0
}

#[cfg(test)]
mod tests;
