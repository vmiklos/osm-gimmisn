/*
 * Copyright 2023 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Database schema creation / migration.

pub fn init(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute(
        "create table if not exists ref_housenumbers (
            county_code text not null,
            settlement_code text not null,
            street text not null,
            housenumber text not null,
            comment text not null
         )",
        [],
    )?;
    conn.execute(
        "create index if not exists idx_ref_housenumbers
            on ref_housenumbers (county_code, settlement_code, street)",
        [],
    )?;
    conn.execute(
        "create table if not exists ref_streets (
            county_code text not null,
            settlement_code text not null,
            street text not null
         )",
        [],
    )?;
    conn.execute(
        "create index if not exists idx_ref_streets
            on ref_streets (county_code, settlement_code)",
        [],
    )?;
    conn.execute(
        "create table if not exists osm_housenumber_coverages (
            relation_name text primary key not null,
            coverage text not null,
            last_modified text not null
         )",
        [],
    )?;
    conn.execute(
        "create table if not exists osm_street_coverages (
             relation_name text primary key not null,
             coverage text not null,
             last_modified text not null
         )",
        [],
    )?;
    conn.execute(
        "create table if not exists stats_invalid_addr_cities (
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
        "create table if not exists mtimes (
            page text primary key not null,
            last_modified text not null
        )",
        [],
    )?;
    conn.execute("pragma user_version = 1", [])?;
    Ok(())
}
