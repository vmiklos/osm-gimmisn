/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The area_files module contains file handling functionality, to be used by the areas module.

use crate::context;
use anyhow::Context;
use std::cell::RefCell;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;

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

    /// Build the file name of the OSM street list of a relation.
    pub fn get_osm_streets_path(&self) -> String {
        format!("{}/streets-{}.csv", self.workdir, self.name)
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

    /// Builds the file name of the house number percent file of a relation.
    pub fn get_housenumbers_percent_path(&self) -> String {
        format!("{}/{}.percent", self.workdir, self.name)
    }

    /// Builds the file name of the house number json cache file of a relation.
    pub fn get_housenumbers_jsoncache_path(&self) -> String {
        format!("{}/{}.cache.json", self.workdir, self.name)
    }

    /// Builds the file name of the additional house number json cache file of a relation.
    pub fn get_additional_housenumbers_jsoncache_path(&self) -> String {
        format!("{}/additional-cache-{}.json", self.workdir, self.name)
    }

    /// Builds the file name of the street percent file of a relation.
    pub fn get_streets_percent_path(&self) -> String {
        format!("{}/{}-streets.percent", self.workdir, self.name)
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
    pub fn get_osm_streets_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let path = self.get_osm_streets_path();
        ctx.get_file_system().open_read(&path)
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

    /// Opens the OSM street list of a relation for writing.
    fn get_osm_streets_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        let path = self.get_osm_streets_path();
        ctx.get_file_system().open_write(&path)
    }

    /// Writes the result for overpass of Relation.get_osm_streets_query().
    pub fn write_osm_streets(&self, ctx: &context::Context, result: &str) -> anyhow::Result<usize> {
        if result.starts_with("<?xml") {
            // Not a CSV, reject.
            return Ok(0);
        }

        let write = self.get_osm_streets_write_stream(ctx)?;
        let mut guard = write.borrow_mut();
        Ok(guard.write(result.as_bytes())?)
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
