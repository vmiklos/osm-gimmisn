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
use crate::i18n;
use anyhow::anyhow;
use anyhow::Context;
use pyo3::prelude::*;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

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
    pub fn get_ref_streets_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("streets-reference-{}.lst", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Build the file name of the OSM street list of a relation.
    pub fn get_osm_streets_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("streets-{}.csv", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Build the file name of the OSM house number list of a relation.
    pub fn get_osm_housenumbers_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("street-housenumbers-{}.csv", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Build the file name of the reference house number list of a relation.
    pub fn get_ref_housenumbers_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("street-housenumbers-reference-{}.lst", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the house number percent file of a relation.
    pub fn get_housenumbers_percent_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("{}.percent", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the house number HTML cache file of a relation.
    pub fn get_housenumbers_htmlcache_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("{}.htmlcache.{}", self.name, i18n::get_language()))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the house number plain text cache file of a relation.
    pub fn get_housenumbers_txtcache_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("{}.txtcache", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the street percent file of a relation.
    pub fn get_streets_percent_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("{}-streets.percent", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the street additional count file of a relation.
    pub fn get_streets_additional_count_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("{}-additional-streets.count", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the housenumber additional count file of a relation.
    pub fn get_housenumbers_additional_count_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!("{}-additional-housenumbers.count", self.name))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Builds the file name of the additional house number HTML cache file of a relation.
    pub fn get_additional_housenumbers_htmlcache_path(&self) -> anyhow::Result<String> {
        let path = std::path::Path::new(&self.workdir);
        Ok(path
            .join(format!(
                "{}.additional-htmlcache.{}",
                self.name,
                i18n::get_language()
            ))
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .into())
    }

    /// Opens the reference street list of a relation for reading.
    pub fn get_ref_streets_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_ref_streets_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the reference street list of a relation for wrtiting.
    pub fn get_ref_streets_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_ref_streets_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Opens the OSM street list of a relation for reading.
    pub fn get_osm_streets_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_osm_streets_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the OSM house number list of a relation for reading.
    pub fn get_osm_housenumbers_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_osm_housenumbers_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the reference house number list of a relation for reading.
    pub fn get_ref_housenumbers_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_ref_housenumbers_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the reference house number list of a relation for writing.
    pub fn get_ref_housenumbers_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_ref_housenumbers_path()?;
        ctx.get_file_system()
            .open_write(&path)
            .context("open_write() failed")
    }

    /// Opens the house number percent file of a relation for reading.
    pub fn get_housenumbers_percent_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_housenumbers_percent_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the house number percent file of a relation for writing.
    pub fn get_housenumbers_percent_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_housenumbers_percent_path()?;
        ctx.get_file_system()
            .open_write(&path)
            .with_context(|| format!("failed to open {} for writing", path))
    }

    /// Opens the house number HTML cache file of a relation for reading.
    pub fn get_housenumbers_htmlcache_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_housenumbers_htmlcache_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the house number HTML cache file of a relation for writing.
    pub fn get_housenumbers_htmlcache_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_housenumbers_htmlcache_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Opens the house number plain text cache file of a relation for reading.
    pub fn get_housenumbers_txtcache_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_housenumbers_txtcache_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the house number plain text cache file of a relation for writing.
    pub fn get_housenumbers_txtcache_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_housenumbers_txtcache_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Opens the street percent file of a relation for reading.
    pub fn get_streets_percent_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_streets_percent_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the street percent file of a relation for writing.
    pub fn get_streets_percent_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_streets_percent_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Opens the street additional count file of a relation for reading.
    pub fn get_streets_additional_count_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_streets_additional_count_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the street additional count file of a relation for writing.
    pub fn get_streets_additional_count_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_streets_additional_count_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Opens the housenumbers additional count file of a relation for reading.
    pub fn get_housenumbers_additional_count_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_housenumbers_additional_count_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the housenumbers additional count file of a relation for writing.
    pub fn get_housenumbers_additional_count_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_housenumbers_additional_count_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Opens the OSM street list of a relation for writing.
    fn get_osm_streets_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_osm_streets_path()?;
        ctx.get_file_system().open_write(&path)
    }

    /// Writes the result for overpass of Relation.get_osm_streets_query().
    pub fn write_osm_streets(&self, ctx: &context::Context, result: &str) -> anyhow::Result<usize> {
        if result.starts_with("<?xml") {
            // Not a CSV, reject.
            return Ok(0);
        }

        let write = self.get_osm_streets_write_stream(ctx)?;
        let mut guard = write.lock().unwrap();
        Ok(guard.write(result.as_bytes())?)
    }

    /// Opens the OSM house number list of a relation for writing.
    fn get_osm_housenumbers_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_osm_housenumbers_path()?;
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
        let mut guard = write.lock().unwrap();
        Ok(guard.write(result.as_bytes())?)
    }

    /// Opens the additional house number HTML cache file of a relation for reading.
    pub fn get_additional_housenumbers_htmlcache_read_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let path = self.get_additional_housenumbers_htmlcache_path()?;
        ctx.get_file_system().open_read(&path)
    }

    /// Opens the additional house number HTML cache file of a relation for writing.
    pub fn get_additional_housenumbers_htmlcache_write_stream(
        &self,
        ctx: &context::Context,
    ) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let path = self.get_additional_housenumbers_htmlcache_path()?;
        ctx.get_file_system().open_write(&path)
    }
}

#[pyclass]
pub struct PyRelationFiles {
    pub relation_files: RelationFiles,
}

#[pymethods]
impl PyRelationFiles {
    #[new]
    fn new(workdir: String, name: String) -> Self {
        let relation_files = RelationFiles::new(&workdir, &name);
        PyRelationFiles { relation_files }
    }

    fn get_ref_streets_path(&self) -> PyResult<String> {
        match self.relation_files.get_ref_streets_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_ref_streets_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_osm_streets_path(&self) -> PyResult<String> {
        match self.relation_files.get_osm_streets_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_osm_streets_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_osm_housenumbers_path(&self) -> PyResult<String> {
        match self.relation_files.get_osm_housenumbers_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_osm_housenumbers_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_ref_housenumbers_path(&self) -> PyResult<String> {
        match self.relation_files.get_ref_housenumbers_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_ref_housenumbers_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_housenumbers_percent_path(&self) -> PyResult<String> {
        match self.relation_files.get_housenumbers_percent_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_housenumbers_percent_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_housenumbers_htmlcache_path(&self) -> PyResult<String> {
        match self.relation_files.get_housenumbers_htmlcache_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_housenumbers_htmlcache_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_streets_percent_path(&self) -> PyResult<String> {
        match self.relation_files.get_streets_percent_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_streets_percent_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_streets_additional_count_path(&self) -> PyResult<String> {
        match self.relation_files.get_streets_additional_count_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_streets_additional_count_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_housenumbers_additional_count_path(&self) -> PyResult<String> {
        match self.relation_files.get_housenumbers_additional_count_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_housenumbers_additional_count_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_ref_streets_read_stream(&self, ctx: PyObject) -> PyResult<context::PyRead> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_ref_streets_read_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_ref_streets_read_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyRead { read: ret })
    }

    fn get_osm_streets_read_stream(&self, ctx: PyObject) -> PyResult<context::PyRead> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_osm_streets_read_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_osm_streets_read_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyRead { read: ret })
    }

    fn get_osm_housenumbers_read_stream(&self, ctx: PyObject) -> PyResult<context::PyRead> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_osm_housenumbers_read_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_osm_housenumbers_read_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyRead { read: ret })
    }

    fn get_ref_housenumbers_read_stream(&self, ctx: PyObject) -> PyResult<context::PyRead> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_ref_housenumbers_read_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_ref_housenumbers_read_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyRead { read: ret })
    }

    fn get_streets_percent_read_stream(&self, ctx: PyObject) -> PyResult<context::PyRead> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_streets_percent_read_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_streets_percent_read_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyRead { read: ret })
    }

    fn get_streets_additional_count_write_stream(
        &self,
        ctx: PyObject,
    ) -> PyResult<context::PyWrite> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_streets_additional_count_write_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_streets_additional_count_write_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyWrite { write: ret })
    }

    fn get_housenumbers_additional_count_write_stream(
        &self,
        ctx: PyObject,
    ) -> PyResult<context::PyWrite> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let ret = match self
            .relation_files
            .get_housenumbers_additional_count_write_stream(&ctx.context)
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_housenumbers_additional_count_write_stream() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(context::PyWrite { write: ret })
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyRelationFiles>()?;
    Ok(())
}
