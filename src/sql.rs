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

pub fn init(conn: &mut rusqlite::Connection) -> anyhow::Result<()> {
    let tx = conn.transaction()?;
    let user_version: i64 = {
        let mut stmt = tx.prepare("pragma user_version")?;
        let mut rows = stmt.query([])?;
        let row = rows.next()?.context("no row")?;
        row.get(0).context("no col")?
    };
    if user_version < 1 {
        tx.execute(
            "create table ref_housenumbers (
            county_code text not null,
            settlement_code text not null,
            street text not null,
            housenumber text not null,
            comment text not null
         )",
            [],
        )?;
        tx.execute(
            "create index idx_ref_housenumbers
            on ref_housenumbers (county_code, settlement_code, street)",
            [],
        )?;
        tx.execute(
            "create table ref_streets (
            county_code text not null,
            settlement_code text not null,
            street text not null
         )",
            [],
        )?;
        tx.execute(
            "create index idx_ref_streets
            on ref_streets (county_code, settlement_code)",
            [],
        )?;
        tx.execute(
            "create table osm_housenumber_coverages (
            relation_name text primary key not null,
            coverage text not null,
            last_modified text not null
         )",
            [],
        )?;
        tx.execute(
            "create table osm_street_coverages (
             relation_name text primary key not null,
             coverage text not null,
             last_modified text not null
         )",
            [],
        )?;
        tx.execute(
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
        tx.execute(
            "create table mtimes (
            page text primary key not null,
            last_modified text not null
        )",
            [],
        )?;
    }
    if user_version < 2 {
        tx.execute(
            "alter table stats_invalid_addr_cities add column
            timestamp text not null default ''",
            [],
        )?;
        tx.execute(
            "alter table stats_invalid_addr_cities add column
            fixme text not null default ''",
            [],
        )?;
    }
    if user_version < 3 {
        // Tracks the number of rows in the stats_invalid_addr_cities table over time.
        tx.execute(
            "create table stats_invalid_addr_cities_counts (
            date text primary key not null,
            count text not null
        )",
            [],
        )?;
    }
    if user_version < 4 {
        // Tracks the number of OSM house numbers over time.
        tx.execute(
            "create table stats_counts (
            date text primary key not null,
            count text not null
        )",
            [],
        )?;
    }
    if user_version < 5 {
        // Tracks the number of OSM house number editors over time.
        tx.execute(
            "create table stats_usercounts (
            date text primary key not null,
            count text not null
        )",
            [],
        )?;
    }
    if user_version < 6 {
        // Tracks lint results for a relation.
        tx.execute(
            "create table relation_lints (
            id integer primary key autoincrement,
            relation_name text not null,
            street_name text not null,
            source text not null,
            housenumber text not null,
            reason text not null
        )",
            [],
        )?;
    }
    if user_version < 7 {
        // OSM link for relation_lints rows.
        tx.execute(
            "alter table relation_lints add column
            object_id text not null default ''",
            [],
        )?;
        tx.execute(
            "alter table relation_lints add column
            object_type text not null default ''",
            [],
        )?;
    }
    if user_version < 8 {
        // Tracks house numbers of cities over time.
        tx.execute(
            "create table stats_citycounts (
            date text not null,
            city text not null,
            count text not null,
            unique(date, city)
        )",
            [],
        )?;
    }
    if user_version < 9 {
        // Tracks house numbers of cities over time.
        tx.execute(
            "create table stats_topusers (
            date text not null,
            user text not null,
            count text not null,
            unique(date, user)
        )",
            [],
        )?;
    }
    if user_version < 10 {
        // Tracks house numbers of ZIP areas over time.
        tx.execute(
            "create table stats_zipcounts (
            date text not null,
            zip text not null,
            count text not null,
            unique(date, zip)
        )",
            [],
        )?;
    }
    if user_version < 11 {
        // Tracks streets from OSM for a relation.
        tx.execute(
            "create table osm_streets (
            relation text not null,
            osm_id text not null,
            name text not null,
            highway text not null,
            service text not null,
            surface text not null,
            leisure text not null,
            osm_type text not null,
            unique(relation, osm_id)
        )",
            [],
        )?;
        tx.execute(
            "create index idx_osm_streets
            on osm_streets (relation)",
            [],
        )?;
    }
    if user_version < 12 {
        // Tracks housenumbers from OSM for a relation.
        tx.execute(
            "create table osm_housenumbers (
            relation text not null,
            osm_id text not null,
            street text not null,
            housenumber text not null,
            postcode text not null,
            place text not null,
            housename text not null,
            conscriptionnumber text not null,
            flats text not null,
            floor text not null,
            door text not null,
            unit text not null,
            name text not null,
            osm_type text not null,
            unique(relation, osm_id)
        )",
            [],
        )?;
        tx.execute(
            "create index idx_osm_housenumbers
            on osm_housenumbers (relation)",
            [],
        )?;
    }

    if user_version < 13 {
        // Tracks the number of additional streets for a relation.
        tx.execute_batch(
            "create table additional_streets_counts (
                    relation text not null,
                    count text not null,
                    unique(relation)
                );
            create index idx_additional_streets_counts
                on additional_streets_counts(relation);",
        )?;
    }

    if user_version < 14 {
        // Tracks the number of additional housenumbers for a relation.
        tx.execute_batch(
            "create table additional_housenumbers_counts (
                    relation text not null,
                    count text not null,
                    unique(relation)
                );
            create index idx_additional_housenumbers_counts
                on additional_housenumbers_counts(relation);",
        )?;
    }

    if user_version < 15 {
        // Tracks housenumbers for the whole country.
        tx.execute_batch(
            "create table whole_country (
                    postcode text not null,
                    city text not null,
                    street text not null,
                    housenumber text not null,
                    user text not null,
                    osm_id text not null,
                    osm_type text not null,
                    timestamp text not null,
                    place text not null,
                    unit text not null,
                    name text not null,
                    fixme text not null
                );",
        )?;
    }

    if user_version < 16 {
        // Per-relation cache for the missing-housenumbers analysis.
        tx.execute_batch(
            "create table missing_housenumbers_cache (
                    relation text not null,
                    json text not null,
                    unique(relation)
                );
            create index idx_missing_housenumbers_cache
                on missing_housenumbers_cache(relation);",
        )?;
    }

    if user_version < 17 {
        // Per-relation cache for the additional-housenumbers analysis.
        tx.execute_batch(
            "create table additional_housenumbers_cache (
                    relation text not null,
                    json text not null,
                    unique(relation)
                );
            create index idx_additional_housenumbers_cache
                on additional_housenumbers_cache(relation);",
        )?;
    }

    tx.execute("pragma user_version = 17", [])?;
    tx.commit()?;
    Ok(())
}

/// Ignores a primary key constraint violation error, but not other errors.
pub fn ignore_primary_key_constraint(
    result: Result<usize, rusqlite::Error>,
) -> Result<(), rusqlite::Error> {
    match result {
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ErrorCode::ConstraintViolation,
                extended_code: rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY,
            },
            _,
        )) => Ok(()),
        Err(err) => Err(err),
        Ok(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests;
