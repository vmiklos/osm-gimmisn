/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The areas module contains the Relations class and associated functionality.

use pyo3::prelude::*;

/// A relation configuration comes directly from static data, not a result of some external query.
struct RelationConfig {
    parent: serde_json::Value,
    dict: serde_json::Value,
}

impl RelationConfig {
    pub fn new(parent_config: &serde_json::Value, my_config: &serde_json::Value) -> Self {
        RelationConfig {
            parent: parent_config.clone(),
            dict: my_config.clone(),
        }
    }

    /// Gets the untyped value of a property transparently.
    fn get_property(&self, key: &str) -> Option<serde_json::Value> {
        if let Some(value) = self.dict.get(key) {
            return Some(value.clone());
        }

        if let Some(value) = self.parent.get(key) {
            return Some(value.clone());
        }

        None
    }

    /// Sets an untyped value.
    fn set_property(&mut self, key: &str, value: &serde_json::Value) {
        self.dict
            .as_object_mut()
            .unwrap()
            .insert(key.into(), value.clone());
    }

    /// Sets if the relation is active.
    fn set_active(&mut self, active: bool) {
        self.set_property("inactive", &serde_json::json!(!active))
    }

    /// Gets if the relation is active.
    fn is_active(&self) -> bool {
        match self.get_property("inactive") {
            Some(value) => !value.as_bool().unwrap(),
            None => true,
        }
    }

    /// Gets the OSM relation object's ID.
    fn get_osmrelation(&self) -> u64 {
        self.get_property("osmrelation").unwrap().as_u64().unwrap()
    }

    /// Gets the relation's refcounty identifier from reference.
    fn get_refcounty(&self) -> String {
        match self.get_property("refcounty") {
            Some(value) => value.as_str().unwrap().into(),
            None => "".into(),
        }
    }

    /// Gets the relation's refsettlement identifier from reference.
    fn get_refsettlement(&self) -> String {
        self.get_property("refsettlement")
            .unwrap()
            .as_str()
            .unwrap()
            .into()
    }

    /// Gets the alias(es) of the relation: alternative names which are also accepted.
    fn get_alias(&self) -> Vec<String> {
        match self.get_property("alias") {
            Some(value) => {
                let aliases = value.as_array().unwrap();
                aliases
                    .iter()
                    .map(|alias| alias.as_str().unwrap().into())
                    .collect()
            }
            None => Vec::new(),
        }
    }
}

#[pyclass]
struct PyRelationConfig {
    relation_config: RelationConfig,
}

#[pymethods]
impl PyRelationConfig {
    #[new]
    fn new(parent_config: String, my_config: String) -> PyResult<Self> {
        let parent_value: serde_json::Value = match serde_json::from_str(&parent_config) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "failed to parse parent_config: {}",
                    err.to_string()
                )));
            }
        };
        let my_value: serde_json::Value = match serde_json::from_str(&my_config) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "failed to parse my_config: {}",
                    err.to_string()
                )));
            }
        };
        let relation_config = RelationConfig::new(&parent_value, &my_value);
        Ok(PyRelationConfig { relation_config })
    }

    fn get_property(&self, key: String) -> PyResult<Option<String>> {
        let ret = match self.relation_config.get_property(&key) {
            Some(value) => value,
            None => {
                return Ok(None);
            }
        };
        match serde_json::to_string(&ret) {
            Ok(value) => Ok(Some(value)),
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "serde_json::to_string() failed: {}",
                    err.to_string()
                )));
            }
        }
    }

    fn set_property(&mut self, key: String, value: String) -> PyResult<()> {
        let serde_value: serde_json::Value = match serde_json::from_str(&value) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "failed to parse value: {}",
                    err.to_string()
                )));
            }
        };
        self.relation_config.set_property(&key, &serde_value);
        Ok(())
    }

    fn set_active(&mut self, active: bool) {
        self.relation_config.set_active(active)
    }

    fn is_active(&self) -> bool {
        self.relation_config.is_active()
    }

    fn get_osmrelation(&self) -> u64 {
        self.relation_config.get_osmrelation()
    }

    fn get_refcounty(&self) -> String {
        self.relation_config.get_refcounty()
    }

    fn get_refsettlement(&self) -> String {
        self.relation_config.get_refsettlement()
    }

    fn get_alias(&self) -> Vec<String> {
        self.relation_config.get_alias()
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyRelationConfig>()?;
    Ok(())
}
