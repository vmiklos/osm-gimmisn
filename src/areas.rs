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
use std::collections::HashMap;
use std::io::BufRead;
use std::io::Read;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

/// A relation configuration comes directly from static data, not a result of some external query.
#[derive(Clone)]
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

    /// Return value can be 'yes', 'no' and 'only'.
    fn should_check_missing_streets(&self) -> String {
        match self.get_property("missing-streets") {
            Some(value) => value.as_str().unwrap().into(),
            None => "yes".into(),
        }
    }

    /// Sets the housenumber_letters property from code.
    fn set_housenumber_letters(&mut self, housenumber_letters: bool) {
        self.set_property(
            "housenumber-letters",
            &serde_json::json!(housenumber_letters),
        )
    }

    /// Do we care if 42/B is missing when 42/A is provided?
    fn should_check_housenumber_letters(&self) -> bool {
        match self.get_property("housenumber-letters") {
            Some(value) => value.as_bool().unwrap(),
            None => false,
        }
    }

    /// Do we care if 42 is in OSM when it's not in the ref?
    fn should_check_additional_housenumbers(&self) -> bool {
        match self.get_property("additional-housenumbers") {
            Some(value) => value.as_bool().unwrap(),
            None => false,
        }
    }

    /// Sets the letter suffix style.
    fn set_letter_suffix_style(&mut self, letter_suffix_style: i32) {
        self.set_property(
            "letter-suffix-style",
            &serde_json::json!(letter_suffix_style),
        )
    }

    /// Gets the letter suffix style.
    fn get_letter_suffix_style(&self) -> i32 {
        match self.get_property("letter-suffix-style") {
            Some(value) => value.as_i64().unwrap() as i32,
            None => crate::util::LetterSuffixStyle::Upper as i32,
        }
    }

    /// Returns an OSM name -> ref name map.
    fn get_refstreets(&self) -> HashMap<String, String> {
        let refstreets = match self.get_property("refstreets") {
            Some(value) => value,
            None => {
                return HashMap::new();
            }
        };

        let mut ret: HashMap<String, String> = HashMap::new();
        for (key, value) in refstreets.as_object().unwrap() {
            ret.insert(key.into(), value.as_str().unwrap().into());
        }
        ret
    }

    /// Sets the 'filters' key from code.
    fn set_filters(&mut self, filters: &serde_json::Value) {
        self.set_property("filters", filters)
    }

    /// Returns a street name -> properties map.
    fn get_filters(&self) -> Option<serde_json::Value> {
        self.get_property("filters")
    }

    /// Returns a street from relation filters.
    fn get_filter_street(&self, street: &str) -> serde_json::Value {
        let filters = match self.get_filters() {
            Some(value) => value,
            None => {
                return serde_json::json!({});
            }
        };
        let filters_obj = match filters.as_object() {
            Some(value) => value,
            None => {
                return serde_json::json!({});
            }
        };

        match filters_obj.get(street) {
            Some(value) => value.clone(),
            None => serde_json::json!({}),
        }
    }

    /// Determines in a relation's street is interpolation=all or not.
    fn get_street_is_even_odd(&self, street: &str) -> bool {
        let value = self.get_filter_street(street);
        let street_props = value.as_object().unwrap();
        let mut interpolation_all = false;
        if let Some(value) = street_props.get("interpolation") {
            if value == "all" {
                interpolation_all = true;
            }
        }
        !interpolation_all
    }

    /// Decides is a ref street should be shown for an OSM street.
    fn should_show_ref_street(&self, osm_street_name: &str) -> bool {
        let value = self.get_filter_street(osm_street_name);
        let street_props = value.as_object().unwrap();
        let mut show_ref_street = true;
        if let Some(value) = street_props.get("show-refstreet") {
            show_ref_street = value.as_bool().unwrap();
        }

        show_ref_street
    }

    /// Returns a list of refsettlement values specific to a street.
    fn get_street_refsettlement(&self, street: &str) -> Vec<String> {
        let mut ret: Vec<String> = vec![self.get_refsettlement()];
        let filters = match self.get_filters() {
            Some(value) => value,
            None => {
                return ret;
            }
        };

        let filters = filters.as_object().unwrap();
        for (filter_street, value) in filters {
            if filter_street != street {
                continue;
            }

            let value = value.as_object().unwrap();

            if value.contains_key("refsettlement") {
                let refsettlement: String =
                    value.get("refsettlement").unwrap().as_str().unwrap().into();
                ret = vec![refsettlement];
            }
            if value.contains_key("ranges") {
                let ranges = value.get("ranges").unwrap().as_array().unwrap();
                for street_range in ranges {
                    let street_range_dict = street_range.as_object().unwrap();
                    if street_range_dict.contains_key("refsettlement") {
                        ret.push(
                            street_range_dict
                                .get("refsettlement")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .into(),
                        );
                    }
                }
            }
        }

        ret.sort();
        ret.dedup();
        ret
    }

    /// Gets list of streets which are only in reference, but have to be filtered out.
    fn get_street_filters(&self) -> Vec<String> {
        let street_filters = match self.get_property("street-filters") {
            Some(value) => value,
            None => {
                return vec![];
            }
        };
        street_filters
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.as_str().unwrap().into())
            .collect()
    }

    /// Gets list of streets which are only in OSM, but have to be filtered out.
    fn get_osm_street_filters(&self) -> Vec<String> {
        let osm_street_filters = match self.get_property("osm-street-filters") {
            Some(value) => value,
            None => {
                return vec![];
            }
        };
        osm_street_filters
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.as_str().unwrap().into())
            .collect()
    }

    /// Builds a list of streets from a reference cache.
    fn build_ref_streets(
        &self,
        reference: &HashMap<String, HashMap<String, Vec<String>>>,
    ) -> Vec<String> {
        let refcounty = self.get_refcounty();
        let refsettlement = self.get_refsettlement();
        reference
            .get(&refcounty)
            .unwrap()
            .get(&refsettlement)
            .unwrap()
            .clone()
    }

    /// Maps an OSM street name to a ref street name.
    fn get_ref_street_from_osm_street(&self, osm_street_name: &str) -> String {
        let refstreets = self.get_refstreets();
        match refstreets.get(osm_street_name) {
            Some(value) => value.into(),
            None => osm_street_name.into(),
        }
    }

    /// Maps a reference street name to an OSM street name.
    fn get_osm_street_from_ref_street(&self, ref_street_name: &str) -> String {
        let refstreets = self.get_refstreets();
        let reverse: HashMap<String, String> = refstreets
            .iter()
            .map(|(key, value)| (value.clone(), key.clone()))
            .collect();

        match reverse.get(ref_street_name) {
            Some(value) => value.into(),
            None => ref_street_name.into(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
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

    fn should_check_missing_streets(&self) -> String {
        self.relation_config.should_check_missing_streets()
    }

    fn should_check_housenumber_letters(&self) -> bool {
        self.relation_config.should_check_housenumber_letters()
    }

    fn should_check_additional_housenumbers(&self) -> bool {
        self.relation_config.should_check_additional_housenumbers()
    }

    fn set_housenumber_letters(&mut self, housenumber_letters: bool) {
        self.relation_config
            .set_housenumber_letters(housenumber_letters)
    }

    fn set_letter_suffix_style(&mut self, letter_suffix_style: i32) {
        self.relation_config
            .set_letter_suffix_style(letter_suffix_style)
    }

    fn get_letter_suffix_style(&self) -> i32 {
        self.relation_config.get_letter_suffix_style()
    }

    fn get_refstreets(&self) -> HashMap<String, String> {
        self.relation_config.get_refstreets()
    }

    fn set_filters(&mut self, filters: String) -> PyResult<()> {
        let serde_value: serde_json::Value = match serde_json::from_str(&filters) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "failed to parse value: {}",
                    err.to_string()
                )));
            }
        };
        self.relation_config.set_filters(&serde_value);
        Ok(())
    }

    fn get_filters(&self) -> PyResult<Option<String>> {
        let ret = match self.relation_config.get_filters() {
            Some(value) => value,
            None => {
                return Ok(None);
            }
        };
        match serde_json::to_string(&ret) {
            Ok(value) => Ok(Some(value)),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "serde_json::to_string() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_street_is_even_odd(&self, street: String) -> bool {
        self.relation_config.get_street_is_even_odd(&street)
    }

    fn should_show_ref_street(&self, osm_street_name: String) -> bool {
        self.relation_config
            .should_show_ref_street(&osm_street_name)
    }

    fn get_street_refsettlement(&self, street: String) -> Vec<String> {
        self.relation_config.get_street_refsettlement(&street)
    }

    fn get_street_filters(&self) -> Vec<String> {
        self.relation_config.get_street_filters()
    }

    fn get_osm_street_filters(&self) -> Vec<String> {
        self.relation_config.get_osm_street_filters()
    }

    fn build_ref_streets(
        &self,
        reference: HashMap<String, HashMap<String, Vec<String>>>,
    ) -> Vec<String> {
        self.relation_config.build_ref_streets(&reference)
    }

    fn get_ref_street_from_osm_street(&self, osm_street_name: String) -> String {
        self.relation_config
            .get_ref_street_from_osm_street(&osm_street_name)
    }

    fn get_osm_street_from_ref_street(&self, ref_street_name: String) -> String {
        self.relation_config
            .get_osm_street_from_ref_street(&ref_street_name)
    }
}

/// A relation is a closed polygon on the map.
struct Relation {
    ctx: crate::context::Context,
    name: String,
    file: crate::area_files::RelationFiles,
    config: RelationConfig,
    osm_housenumbers: HashMap<String, Vec<crate::util::HouseNumber>>,
}

impl Relation {
    fn new(
        ctx: &crate::context::Context,
        name: &str,
        parent_config: &serde_json::Value,
        yaml_cache: &serde_json::Value,
    ) -> anyhow::Result<Self> {
        let mut my_config = serde_json::json!({});
        let file = crate::area_files::RelationFiles::new(&ctx.get_ini().get_workdir()?, name);
        let relation_path = format!("relation-{}.yaml", name);
        // Intentionally don't require this cache to be present, it's fine to omit it for simple
        // relations.
        if let Some(value) = yaml_cache.as_object().unwrap().get(&relation_path) {
            my_config = value.clone();
        }
        let config = RelationConfig::new(parent_config, &my_config);
        // osm street name -> house number list map, so we don't have to read the on-disk list of the
        // relation again and again for each street.
        let osm_housenumbers: HashMap<String, Vec<crate::util::HouseNumber>> = HashMap::new();
        Ok(Relation {
            ctx: ctx.clone(),
            name: name.into(),
            file,
            config,
            osm_housenumbers,
        })
    }

    /// Gets the name of the relation.
    fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Gets access to the file interface.
    fn get_files(&self) -> crate::area_files::RelationFiles {
        self.file.clone()
    }

    /// Gets access to the config interface.
    fn get_config(&self) -> RelationConfig {
        self.config.clone()
    }

    /// Sets the config interface.
    fn set_config(&mut self, config: &RelationConfig) {
        self.config = config.clone()
    }

    /// Gets a street name -> ranges map, which allows silencing false positives.
    fn get_street_ranges(&self) -> HashMap<String, crate::ranges::Ranges> {
        let mut filter_dict: HashMap<String, crate::ranges::Ranges> = HashMap::new();

        let mut filters: HashMap<String, serde_json::Value> = HashMap::new();
        if let Some(value) = self.config.get_filters() {
            for (key, value) in value.as_object().unwrap() {
                filters.insert(key.into(), value.clone());
            }
        }
        for street in filters.keys() {
            let mut interpolation = "";
            let filter = filters.get(street).unwrap().as_object().unwrap();
            if let Some(value) = filter.get("interpolation") {
                interpolation = value.as_str().unwrap();
            }
            let mut i: Vec<crate::ranges::Range> = Vec::new();
            if let Some(value) = filter.get("ranges") {
                for start_end in value.as_array().unwrap() {
                    let start_end_obj = start_end.as_object().unwrap();
                    let start = start_end_obj
                        .get("start")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse::<i64>()
                        .unwrap();
                    let end = start_end_obj
                        .get("end")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .parse::<i64>()
                        .unwrap();
                    i.push(crate::ranges::Range::new(start, end, interpolation));
                }
                filter_dict.insert(street.into(), crate::ranges::Ranges::new(i));
            }
        }

        filter_dict
    }

    /// Gets streets from reference.
    fn get_ref_streets(&self) -> anyhow::Result<Vec<String>> {
        let mut streets: Vec<String> = Vec::new();
        let read: Arc<Mutex<dyn Read + Send>> = self.file.get_ref_streets_read_stream(&self.ctx)?;
        let mut guard = read.lock().unwrap();
        let stream = std::io::BufReader::new(guard.deref_mut());
        for line in stream.lines() {
            streets.push(line?);
        }
        streets.sort();
        streets.dedup();
        Ok(streets)
    }

    /// Gets the OSM house number list of a street.
    fn get_osm_housenumbers(&mut self, street_name: &str) -> Vec<crate::util::HouseNumber> {
        if self.osm_housenumbers.is_empty() {
            // TODO impl this
        }
        match self.osm_housenumbers.get(street_name) {
            Some(value) => value.clone(),
            None => {
                self.osm_housenumbers.insert(street_name.into(), vec![]);
                vec![]
            }
        }
    }
}

#[pyclass]
struct PyRelation {
    relation: Relation,
}

#[pymethods]
impl PyRelation {
    #[new]
    fn new(
        ctx: PyObject,
        name: String,
        parent_config: String,
        yaml_cache: String,
    ) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, crate::context::PyContext> = ctx.extract(gil.python())?;

        let parent_value: serde_json::Value = match serde_json::from_str(&parent_config) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "failed to parse parent_config: {}",
                    err.to_string()
                )));
            }
        };
        let cache_value: serde_json::Value = match serde_json::from_str(&yaml_cache) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "failed to parse yaml_cache: {}",
                    err.to_string()
                )));
            }
        };
        let relation = match Relation::new(&ctx.context, &name, &parent_value, &cache_value) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "Relation::new() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(PyRelation { relation })
    }

    fn get_name(&self) -> String {
        self.relation.get_name()
    }

    fn get_files(&self) -> crate::area_files::PyRelationFiles {
        let relation_files = self.relation.get_files();
        crate::area_files::PyRelationFiles { relation_files }
    }

    fn get_config(&self) -> PyRelationConfig {
        let relation_config = self.relation.get_config();
        PyRelationConfig { relation_config }
    }

    fn set_config(&mut self, config: PyRelationConfig) {
        self.relation.set_config(&config.relation_config)
    }

    fn get_ref_streets(&self) -> PyResult<Vec<String>> {
        match self.relation.get_ref_streets() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_ref_streets() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_osm_housenumbers(&mut self, street_name: String) -> Vec<crate::util::PyHouseNumber> {
        self.relation
            .get_osm_housenumbers(&street_name)
            .iter()
            .map(|i| {
                let house_number = i.clone();
                crate::util::PyHouseNumber { house_number }
            })
            .collect()
    }

    fn get_street_ranges(&self) -> HashMap<String, crate::ranges::PyRanges> {
        let mut ret: HashMap<String, crate::ranges::PyRanges> = HashMap::new();
        for (key, value) in self.relation.get_street_ranges() {
            ret.insert(key, crate::ranges::PyRanges { ranges: value });
        }
        ret
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyRelationConfig>()?;
    module.add_class::<PyRelation>()?;
    Ok(())
}
