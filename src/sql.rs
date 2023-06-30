/*
 * Copyright 2023 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Database schema creation / migration.

use anyhow::Context as _;

pub fn init(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("pragma user_version")?;
    let mut rows = stmt.query([])?;
    let row = rows.next()?.context("no row")?;
    let user_version: i64 = row.get(0).context("no col")?;
    if user_version < 1 {
        conn.execute(
            "create table ref_housenumbers (
            county_code text not null,
            settlement_code text not null,
            street text not null,
            housenumber text not null,
            comment text not null
         )",
            [],
        )?;
        conn.execute(
            "create index idx_ref_housenumbers
            on ref_housenumbers (county_code, settlement_code, street)",
            [],
        )?;
        conn.execute(
            "create table ref_streets (
            county_code text not null,
            settlement_code text not null,
            street text not null
         )",
            [],
        )?;
        conn.execute(
            "create index idx_ref_streets
            on ref_streets (county_code, settlement_code)",
            [],
        )?;
        conn.execute(
            "create table osm_housenumber_coverages (
            relation_name text primary key not null,
            coverage text not null,
            last_modified text not null
         )",
            [],
        )?;
        conn.execute(
            "create table osm_street_coverages (
             relation_name text primary key not null,
             coverage text not null,
             last_modified text not null
         )",
            [],
        )?;
        conn.execute(
            "create table stats_invalid_addr_cities (
            osm_id text not null,
            osm_type text not null,
            postcode text not null,
            city text not null,
            street text not null,
            housenumber text not null,
            user text not null
        )",
            [],
        )?;
        conn.execute(
            "create table mtimes (
            page text primary key not null,
            last_modified text not null
        )",
            [],
        )?;
    }
    if user_version < 2 {
        conn.execute(
            "alter table stats_invalid_addr_cities add column
            timestamp text not null default ''",
            [],
        )?;
        conn.execute(
            "alter table stats_invalid_addr_cities add column
            fixme text not null default ''",
            [],
        )?;
    }
    if user_version < 3 {
        // Tracks the number of rows in the stats_invalid_addr_cities table over time.
        conn.execute(
            "create table stats_invalid_addr_cities_counts (
            date text primary key not null,
            count text not null
        )",
            [],
        )?;
    }
    if user_version < 4 {
        // Tracks the number of OSM house numbers over time.
        conn.execute(
            "create table stats_counts (
            date text primary key not null,
            count text not null
        )",
            [],
        )?;
    }
    if user_version < 5 {
        // Tracks the number of OSM house number editors over time.
        conn.execute(
            "create table stats_usercounts (
            date text primary key not null,
            count text not null
        )",
            [],
        )?;
    }
    conn.execute("pragma user_version = 5", [])?;
    Ok(())
}

#[cfg(test)]
mod tests;
