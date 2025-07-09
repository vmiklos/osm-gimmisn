/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Compares reference house numbers with OSM ones and shows the diff.

use crate::areas;
use crate::context;
use crate::util;
use std::io::Write;

/// Inner main() that is allowed to fail.
pub fn our_main(
    argv: &[String],
    stream: &mut dyn Write,
    ctx: &context::Context,
) -> anyhow::Result<()> {
    let relation_name = argv[1].clone();

    let mut relations = areas::Relations::new(ctx)?;
    let mut relation = relations.get_relation(&relation_name)?;
    let ongoing_streets = relation.get_missing_housenumbers()?.ongoing_streets;

    for result in ongoing_streets {
        // House number, # of only_in_reference items.
        let range_list = util::get_housenumber_ranges(&result.house_numbers);
        let mut range_strings: Vec<&String> = range_list.iter().map(|i| i.get_number()).collect();
        range_strings.sort_by_key(|i| util::split_house_number(i));
        let buf = format!(
            "{}\t{}\n",
            result.street.get_osm_name(),
            range_strings.len()
        );
        stream.write_all(buf.as_bytes())?;
        // only_in_reference items.
        stream.write_all(format!("{range_strings:?}\n").as_bytes())?;
    }

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
