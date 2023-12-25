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
}

/// OverpassElement represents one result from Overpass.
#[derive(serde::Deserialize)]
struct OverpassElement {
    id: u64,
    #[serde(rename(deserialize = "type"))]
    osm_type: String,
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

    /// Build the file name of the reference street list of a relation.
    pub fn get_ref_streets_path(&self) -> String {
        format!("{}/streets-reference-{}.lst", self.workdir, self.name)
    }

    /// Build the file name of the OSM house number list of a relation.
    pub fn get_osm_housenumbers_path(&self) -> String {
        format!("{}/street-housenumbers-{}.csv", self.workdir, self.name)
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

    /// Builds the file name of the street additional count file of a relation.
    pub fn get_streets_additional_count_path(&self) -> String {
        format!("{}/{}-additional-streets.count", self.workdir, self.name)
    }

    /// Builds the file name of the housenumber additional count file of a relation.
    pub fn get_housenumbers_additional_count_path(&self) -> String {
        format!(
            "{}/{}-additional-housenumbers.count",
            self.workdir, self.name
        )
    }

    /// Opens the reference street list of a relation for reading.
    pub fn get_ref_streets_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let path = self.get_ref_streets_path();
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the reference street list of a relation for wrtiting.
    pub fn get_ref_streets_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        let path = self.get_ref_streets_path();
        ctx.get_file_system().open_write(&path)
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
    pub fn get_osm_housenumbers_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let path = self.get_osm_housenumbers_path();
        ctx.get_file_system().open_read(&path)
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

    /// Opens the housenumbers additional count file of a relation for reading.
    pub fn get_housenumbers_additional_count_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let path = self.get_housenumbers_additional_count_path();
        ctx.get_file_system().open_read(&path)
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

    /// Opens the OSM house number list of a relation for writing.
    fn get_osm_housenumbers_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        let path = self.get_osm_housenumbers_path();
        ctx.get_file_system().open_write(&path)
    }

    /// Writes the result for overpass of Relation.get_osm_housenumbers_query().
    pub fn write_osm_housenumbers(
        &self,
        ctx: &context::Context,
        result: &str,
    ) -> anyhow::Result<usize> {
        if result.starts_with("<?xml") {
            // Not a CSV, reject.
            return Ok(0);
        }

        let write = self.get_osm_housenumbers_write_stream(ctx)?;
        let mut guard = write.borrow_mut();
        Ok(guard.write(result.as_bytes())?)
    }
}

#[cfg(test)]
mod tests;
