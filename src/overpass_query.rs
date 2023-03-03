/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The overpass_query module allows getting data out of the OSM DB without a full download.

use crate::context;

/// Posts the query string to the overpass API and returns the result string.
pub fn overpass_query(ctx: &context::Context, query: &str) -> anyhow::Result<String> {
    let url = ctx.get_ini().get_overpass_uri() + "/api/interpreter";

    ctx.get_network().urlopen(&url, query)
}

/// Checks if we need to sleep before executing an overpass query.
pub fn overpass_query_need_sleep(ctx: &context::Context) -> i32 {
    let url = ctx.get_ini().get_overpass_uri() + "/api/status";
    let status = match ctx.get_network().urlopen(&url, "") {
        Ok(value) => value,
        _ => {
            return 0;
        }
    };
    let mut sleep = 0;
    let mut available = false;
    for line in status.lines() {
        if line.starts_with("Slot available after:") {
            let re = regex::Regex::new(r".*in (-?\d+) seconds.*").unwrap();
            for cap in re.captures_iter(line) {
                // This should neve fail since the regex only allows numbers.
                sleep = cap[1].parse::<i32>().expect("parse() to i32 failed");
                // Wait one more second just to be safe.
                sleep += 1;
                if sleep <= 0 {
                    sleep = 1;
                }
            }
            break;
        }
        if line.contains("available now") {
            available = true;
        }
    }
    if available {
        return 0;
    }
    sleep
}

#[cfg(test)]
mod tests;
