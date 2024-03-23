/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The area_files module contains file handling functionality, to be used by the areas module.

use crate::context;
use crate::stats;
use crate::util;
use anyhow::Context;
use std::cell::RefCell;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;

#[cfg(not(test))]
use log::info;

#[cfg(test)]
use std::println as info;

/// OverpassTags contains various tags about one Overpass element.
#[derive(serde::Deserialize)]
struct OverpassTags {
    name: Option<String>,
    highway: Option<String>,
    service: Option<String>,
    surface: Option<String>,
    leisure: Option<String>,

    // region: housenumbers
    #[serde(rename(deserialize = "addr:street"))]
    street: Option<String>,
    #[serde(rename(deserialize = "addr:housenumber"))]
    housenumber: Option<String>,
    #[serde(rename(deserialize = "addr:postcode"))]
    postcode: Option<String>,
    #[serde(rename(deserialize = "addr:place"))]
    place: Option<String>,
    #[serde(rename(deserialize = "addr:housename"))]
    housename: Option<String>,
    #[serde(rename(deserialize = "addr:conscriptionnumber"))]
    conscriptionnumber: Option<String>,
    #[serde(rename(deserialize = "addr:flats"))]
    flats: Option<String>,
    #[serde(rename(deserialize = "addr:floor"))]
    floor: Option<String>,
    #[serde(rename(deserialize = "addr:door"))]
    door: Option<String>,
    #[serde(rename(deserialize = "addr:unit"))]
    unit: Option<String>,
    #[serde(rename(deserialize = "addr:city"))]
    city: Option<String>,
    // endregion housenumbers
    fixme: Option<String>,
}

/// OverpassElement represents one result from Overpass.
#[derive(serde::Deserialize)]
struct OverpassElement {
    id: u64,
    #[serde(rename(deserialize = "type"))]
    osm_type: String,
    pub user: Option<String>,
    pub timestamp: Option<String>,
    tags: OverpassTags,
}

#[derive(serde::Deserialize)]
struct OverpassTimes {
    #[serde(with = "time::serde::rfc3339")]
    timestamp_osm_base: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    timestamp_areas_base: time::OffsetDateTime,
}

/// OverpassResult is the result from Overpass.
#[derive(serde::Deserialize)]
struct OverpassResult {
    osm3s: OverpassTimes,
    elements: Vec<OverpassElement>,
}

/// One row in the `osm_streets` SQL table for a relation. Keep this in sync with data/streets-template.overpassql.
pub struct OsmStreet {
    /// Object ID.
    pub id: u64,
    /// Street name.
    pub name: String,
    /// Object type.
    pub object_type: Option<String>,
}

impl OsmStreet {
    fn new(id: u64, name: &str, object_type: &Option<String>) -> Self {
        let name = name.to_string();
        let object_type = object_type.clone();
        OsmStreet {
            id,
            name,
            object_type,
        }
    }
}

/// A relation's file interface provides access to files associated with a relation.
#[derive(Clone)]
pub struct RelationFiles {
    workdir: String,
    name: String,
}

impl RelationFiles {
    pub fn new(workdir: &str, name: &str) -> Self {
        RelationFiles {
            workdir: workdir.into(),
            name: name.into(),
        }
    }

    /// Build the file name of the reference house number list of a relation.
    pub fn get_ref_housenumbers_path(&self) -> String {
        format!(
            "{}/street-housenumbers-reference-{}.lst",
            self.workdir, self.name
        )
    }

    /// Builds the file name of the house number json cache file of a relation.
    pub fn get_housenumbers_jsoncache_path(&self) -> String {
        format!("{}/cache-{}.json", self.workdir, self.name)
    }

    /// Builds the file name of the additional house number json cache file of a relation.
    pub fn get_additional_housenumbers_jsoncache_path(&self) -> String {
        format!("{}/additional-cache-{}.json", self.workdir, self.name)
    }

    /// Opens the OSM street list of a relation for reading.
    pub fn get_osm_json_streets(&self, ctx: &context::Context) -> anyhow::Result<Vec<OsmStreet>> {
        let mut ret: Vec<OsmStreet> = Vec::new();
        let conn = ctx.get_database_connection()?;
        let mut stmt =
            conn.prepare("select osm_id, name, osm_type from osm_streets where relation = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0).unwrap();
            let name: String = row.get(1).unwrap();
            let object_type: String = row.get(2).unwrap();
            ret.push(OsmStreet::new(id.parse()?, &name, &Some(object_type)));
        }
        Ok(ret)
    }

    /// Opens the OSM house number list of a relation for reading.
    pub fn get_osm_json_housenumbers(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Vec<util::OsmHouseNumber>> {
        let mut ret: Vec<util::OsmHouseNumber> = Vec::new();
        let conn = ctx.get_database_connection()?;
        let mut stmt =
            conn.prepare("select osm_id, housenumber, conscriptionnumber, street, place, osm_type from osm_housenumbers where relation = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0).unwrap();
            let housenumber: String = row.get(1).unwrap();
            let conscriptionnumber: String = row.get(2).unwrap();
            let street: String = row.get(3).unwrap();
            let place: String = row.get(4).unwrap();
            let object_type: String = row.get(5).unwrap();
            ret.push(util::OsmHouseNumber::new(
                id.parse()?,
                &housenumber,
                &conscriptionnumber,
                &street,
                &Some(place),
                &object_type,
            ));
        }
        Ok(ret)
    }

    /// Opens the reference house number list of a relation for reading.
    pub fn get_ref_housenumbers_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let path = self.get_ref_housenumbers_path();
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the reference house number list of a relation for writing.
    pub fn get_ref_housenumbers_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        let path = self.get_ref_housenumbers_path();
        ctx.get_file_system()
            .open_write(&path)
            .context("open_write() failed")
    }

    /// Writes the result for overpass of Relation.get_osm_streets_json_query().
    pub fn write_osm_json_streets(
        &self,
        ctx: &context::Context,
        result: &str,
    ) -> anyhow::Result<()> {
        let overpass: OverpassResult = match serde_json::from_str(result) {
            Ok(value) => value,
            // Not a JSON, ignore.
            Err(_) => {
                return Ok(());
            }
        };

        // Insert or update the mtime for the osm streets of this relation.
        stats::set_sql_mtime(ctx, &format!("streets/{}", self.name))?;

        let mut conn = ctx.get_database_connection()?;
        let tx = conn.transaction()?;
        tx.execute(
            "delete from osm_streets where relation = ?1",
            [self.name.to_string()],
        )?;
        for element in overpass.elements {
            let relation = self.name.to_string();
            let osm_id = element.id.to_string();
            let name = element.tags.name.unwrap_or("".into());
            let highway = element.tags.highway.unwrap_or("".into());
            let service = element.tags.service.unwrap_or("".into());
            let surface = element.tags.surface.unwrap_or("".into());
            let leisure = element.tags.leisure.unwrap_or("".into());
            let osm_type = element.osm_type.to_string();
            let ret = tx.execute(
                "insert into osm_streets (relation, osm_id, name, highway, service, surface, leisure, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                [relation, osm_id, name, highway, service, surface, leisure, osm_type],
            );
            if ret.is_err() {
                info!("write_osm_json_streets: ignoring duplicated street: relation is '{}', id is '{}'", self.name, element.id);
            }
        }

        let osm_page = format!("streets/{}/osm-base", self.name);
        let osm_time = overpass.osm3s.timestamp_osm_base.unix_timestamp_nanos();
        tx.execute(
            r#"insert into mtimes (page, last_modified) values (?1, ?2)
                 on conflict(page) do update set last_modified = excluded.last_modified"#,
            [osm_page, osm_time.to_string()],
        )?;

        let areas_page = format!("streets/{}/areas-base", self.name);
        let areas_time = overpass.osm3s.timestamp_areas_base.unix_timestamp_nanos();
        tx.execute(
            r#"insert into mtimes (page, last_modified) values (?1, ?2)
                 on conflict(page) do update set last_modified = excluded.last_modified"#,
            [areas_page, areas_time.to_string()],
        )?;
        tx.commit()?;

        Ok(())
    }

    /// Writes the result for overpass of Relation.get_osm_housenumbers_json_query().
    pub fn write_osm_json_housenumbers(
        &self,
        ctx: &context::Context,
        result: &str,
    ) -> anyhow::Result<()> {
        let overpass: OverpassResult = match serde_json::from_str(result) {
            Ok(value) => value,
            // Not a JSON, ignore.
            Err(_) => {
                return Ok(());
            }
        };

        // Insert or update the mtime for the osm housenumbers of this relation.
        stats::set_sql_mtime(ctx, &format!("housenumbers/{}", self.name))?;

        let mut conn = ctx.get_database_connection()?;
        let tx = conn.transaction()?;
        tx.execute(
            "delete from osm_housenumbers where relation = ?1",
            [self.name.to_string()],
        )?;
        for element in overpass.elements {
            let relation = self.name.to_string();
            let osm_id = element.id.to_string();
            let street = element.tags.street.unwrap_or("".into());
            let housenumber = element.tags.housenumber.unwrap_or("".into());
            let postcode = element.tags.postcode.unwrap_or("".into());
            let place = element.tags.place.unwrap_or("".into());
            let housename = element.tags.housename.unwrap_or("".into());
            let conscriptionnumber = element.tags.conscriptionnumber.unwrap_or("".into());
            let flats = element.tags.flats.unwrap_or("".into());
            let floor = element.tags.floor.unwrap_or("".into());
            let door = element.tags.door.unwrap_or("".into());
            let unit = element.tags.unit.unwrap_or("".into());
            let name = element.tags.name.unwrap_or("".into());
            let osm_type = element.osm_type.to_string();
            let ret = tx.execute(
                "insert into osm_housenumbers (relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                [relation, osm_id, street, housenumber, postcode, place, housename, conscriptionnumber, flats, floor, door, unit, name, osm_type],
            );
            if ret.is_err() {
                info!("write_osm_json_housenumbers: ignoring duplicated housenumber: relation is '{}', id is '{}'", self.name, element.id);
            }
        }

        let osm_page = format!("housenumbers/{}/osm-base", self.name);
        let osm_time = overpass.osm3s.timestamp_osm_base.unix_timestamp_nanos();
        tx.execute(
            r#"insert into mtimes (page, last_modified) values (?1, ?2)
                 on conflict(page) do update set last_modified = excluded.last_modified"#,
            [osm_page, osm_time.to_string()],
        )?;

        let areas_page = format!("housenumbers/{}/areas-base", self.name);
        let areas_time = overpass.osm3s.timestamp_areas_base.unix_timestamp_nanos();
        tx.execute(
            r#"insert into mtimes (page, last_modified) values (?1, ?2)
                 on conflict(page) do update set last_modified = excluded.last_modified"#,
            [areas_page, areas_time.to_string()],
        )?;
        tx.commit()?;

        Ok(())
    }
}

pub fn write_whole_country(ctx: &context::Context, result: &str) -> anyhow::Result<()> {
    let overpass: OverpassResult = match serde_json::from_str(result) {
        Ok(value) => value,
        // Not a JSON, ignore.
        Err(_) => {
            return Ok(());
        }
    };

    let mut conn = ctx.get_database_connection()?;
    let tx = conn.transaction()?;
    tx.execute("delete from whole_country", [])?;
    for element in overpass.elements {
        let postcode = element.tags.postcode.unwrap_or("".into());
        let city = element.tags.city.unwrap_or("".into());
        let street = element.tags.street.unwrap_or("".into());
        let housenumber = element.tags.housenumber.unwrap_or("".into());
        let user = element.user.unwrap_or("".into());
        let osm_id = element.id.to_string();
        let osm_type = element.osm_type.to_string();
        let timestamp = element.timestamp.unwrap_or("".into());
        let place = element.tags.place.unwrap_or("".into());
        let unit = element.tags.unit.unwrap_or("".into());
        let name = element.tags.name.unwrap_or("".into());
        let fixme = element.tags.fixme.unwrap_or("".into());
        tx.execute(
                "insert into whole_country (postcode, city, street, housenumber, user, osm_id, osm_type, timestamp, place, unit, name, fixme) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                [postcode, city, street, housenumber, user, osm_id, osm_type, timestamp, place, unit, name, fixme],
            )?;
    }

    let osm_time = overpass.osm3s.timestamp_osm_base.unix_timestamp_nanos();
    tx.execute(
        r#"insert into mtimes (page, last_modified) values ('whole-country/osm-base', ?1)
                 on conflict(page) do update set last_modified = excluded.last_modified"#,
        [osm_time.to_string()],
    )?;

    let areas_time = overpass.osm3s.timestamp_areas_base.unix_timestamp_nanos();
    tx.execute(
        r#"insert into mtimes (page, last_modified) values ('whole-country/areas-base', ?1)
                 on conflict(page) do update set last_modified = excluded.last_modified"#,
        [areas_time.to_string()],
    )?;
    tx.commit()?;

    Ok(())
}

#[cfg(test)]
mod tests;
