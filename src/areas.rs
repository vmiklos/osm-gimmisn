/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The areas module contains the Relations class and associated functionality.

use crate::area_files;
use crate::context;
use crate::i18n::translate as tr;
use crate::ranges;
use crate::util;
use crate::yattag;
use anyhow::Context;
use itertools::Itertools;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::BufRead;
use std::io::Read;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

/// A relation configuration comes directly from static data, not a result of some external query.
#[derive(Clone)]
pub struct RelationConfig {
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
    pub fn get_osmrelation(&self) -> u64 {
        self.get_property("osmrelation").unwrap().as_u64().unwrap()
    }

    /// Gets the relation's refcounty identifier from reference.
    pub fn get_refcounty(&self) -> String {
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
    pub fn should_check_missing_streets(&self) -> String {
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
    pub fn should_check_additional_housenumbers(&self) -> bool {
        match self.get_property("additional-housenumbers") {
            Some(value) => value.as_bool().unwrap(),
            None => false,
        }
    }

    /// Sets the letter suffix style.
    pub fn set_letter_suffix_style(&mut self, letter_suffix_style: i32) {
        self.set_property(
            "letter-suffix-style",
            &serde_json::json!(letter_suffix_style),
        )
    }

    /// Gets the letter suffix style.
    fn get_letter_suffix_style(&self) -> i32 {
        match self.get_property("letter-suffix-style") {
            Some(value) => value.as_i64().unwrap() as i32,
            None => util::LetterSuffixStyle::Upper as i32,
        }
    }

    /// Returns an OSM name -> ref name map.
    pub fn get_refstreets(&self) -> HashMap<String, String> {
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
    fn get_filters(&self) -> Option<&serde_json::Value> {
        // The schema doesn't allow this key in parent config, no need to go via the slow
        // get_property().
        self.dict.get("filters")
    }

    /// Returns a street from relation filters.
    fn get_filter_street(&self, street: &str) -> Option<&serde_json::Value> {
        let filters = match self.get_filters() {
            Some(value) => value,
            None => {
                return None;
            }
        };
        let filters_obj = match filters.as_object() {
            Some(value) => value,
            None => {
                return None;
            }
        };

        filters_obj.get(street)
    }

    /// Determines in a relation's street is interpolation=all or not.
    pub fn get_street_is_even_odd(&self, street: &str) -> bool {
        let mut interpolation_all = false;
        if let Some(filter_for_street) = self.get_filter_street(street) {
            let street_props = filter_for_street.as_object().unwrap();
            if let Some(interpolation) = street_props.get("interpolation") {
                if interpolation == "all" {
                    interpolation_all = true;
                }
            }
        }
        !interpolation_all
    }

    /// Decides is a ref street should be shown for an OSM street.
    fn should_show_ref_street(&self, osm_street_name: &str) -> bool {
        let mut show_ref_street = true;
        if let Some(filter_for_street) = self.get_filter_street(osm_street_name) {
            let street_props = filter_for_street.as_object().unwrap();
            if let Some(value) = street_props.get("show-refstreet") {
                show_ref_street = value.as_bool().unwrap();
            }
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
}

/// A relation is a closed polygon on the map.
#[derive(Clone)]
pub struct Relation {
    ctx: context::Context,
    name: String,
    file: area_files::RelationFiles,
    config: RelationConfig,
    osm_housenumbers: HashMap<String, Vec<util::HouseNumber>>,
}

impl Relation {
    fn new(
        ctx: &context::Context,
        name: &str,
        parent_config: &serde_json::Value,
        yaml_cache: &serde_json::Map<String, serde_json::Value>,
    ) -> anyhow::Result<Self> {
        let mut my_config = serde_json::json!({});
        let file = area_files::RelationFiles::new(&ctx.get_ini().get_workdir()?, name);
        let relation_path = format!("relation-{}.yaml", name);
        // Intentionally don't require this cache to be present, it's fine to omit it for simple
        // relations.
        if let Some(value) = yaml_cache.get(&relation_path) {
            my_config = value.clone();
        }
        let config = RelationConfig::new(parent_config, &my_config);
        // osm street name -> house number list map, so we don't have to read the on-disk list of the
        // relation again and again for each street.
        let osm_housenumbers: HashMap<String, Vec<util::HouseNumber>> = HashMap::new();
        Ok(Relation {
            ctx: ctx.clone(),
            name: name.into(),
            file,
            config,
            osm_housenumbers,
        })
    }

    /// Gets the name of the relation.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Gets access to the file interface.
    pub fn get_files(&self) -> &area_files::RelationFiles {
        &self.file
    }

    /// Gets access to the config interface.
    pub fn get_config(&self) -> &RelationConfig {
        &self.config
    }

    /// Sets the config interface.
    pub fn set_config(&mut self, config: &RelationConfig) {
        self.config = config.clone()
    }

    /// Gets a street name -> ranges map, which allows silencing false positives.
    fn get_street_ranges(&self) -> HashMap<String, ranges::Ranges> {
        let mut filter_dict: HashMap<String, ranges::Ranges> = HashMap::new();

        let filters = match self.config.get_filters() {
            Some(value) => value,
            None => {
                return filter_dict;
            }
        };
        let filters_obj = filters.as_object().unwrap();
        for street in filters_obj.keys() {
            let mut interpolation = "";
            let filter = filters_obj.get(street).unwrap().as_object().unwrap();
            if let Some(value) = filter.get("interpolation") {
                interpolation = value.as_str().unwrap();
            }
            let mut i: Vec<ranges::Range> = Vec::new();
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
                    i.push(ranges::Range::new(start, end, interpolation));
                }
                filter_dict.insert(street.into(), ranges::Ranges::new(i));
            }
        }

        filter_dict
    }

    /// Gets a street name -> invalid map, which allows silencing individual false positives.
    fn get_street_invalid(&self) -> HashMap<String, Vec<String>> {
        let mut invalid_dict: HashMap<String, Vec<String>> = HashMap::new();

        let filters = match self.config.get_filters() {
            Some(value) => value,
            None => {
                return invalid_dict;
            }
        };
        let filters_obj = filters.as_object().unwrap();
        for street in filters_obj.keys() {
            let filter = filters_obj.get(street).unwrap().as_object().unwrap();
            if let Some(value) = filter.get("invalid") {
                let values: Vec<String> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|i| i.as_str().unwrap().into())
                    .collect();
                invalid_dict.insert(street.into(), values);
            }
        }

        invalid_dict
    }

    /// Decides is a ref street should be shown for an OSM street.
    pub fn should_show_ref_street(&self, osm_street_name: &str) -> bool {
        self.config.should_show_ref_street(osm_street_name)
    }

    /// Reads list of streets for an area from OSM.
    fn get_osm_streets(&self, sorted_result: bool) -> anyhow::Result<Vec<util::Street>> {
        let mut ret: Vec<util::Street> = Vec::new();
        let stream: Arc<Mutex<dyn Read + Send>> =
            self.file.get_osm_streets_read_stream(&self.ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut read = guard.deref_mut();
        let mut csv_read = util::CsvRead::new(&mut read);
        let mut first = true;
        for result in csv_read.records() {
            if first {
                first = false;
                continue;
            }

            let row = match result {
                Ok(value) => value,
                Err(_) => {
                    continue;
                }
            };
            // 0: @id, 1: name, 6: @type
            if row.get(1).is_none() {
                // data/streets-template.txt requests this, so we got garbage, give up.
                return Err(anyhow::anyhow!("missing name column in CSV"));
            }
            let mut street = util::Street::new(
                /*osm_name=*/ &row[1],
                /*ref_name=*/ "",
                /*show_ref_street=*/ true,
                /*osm_id=*/ row[0].parse::<u64>().unwrap(),
            );
            if let Some(value) = row.get(6) {
                street.set_osm_type(value);
            }
            street.set_source(&tr("street"));
            ret.push(street)
        }
        let path = self.file.get_osm_housenumbers_path()?;
        if std::path::Path::new(&path).exists() {
            let stream: Arc<Mutex<dyn Read + Send>> =
                self.file.get_osm_housenumbers_read_stream(&self.ctx)?;
            let mut guard = stream.lock().unwrap();
            let mut read = guard.deref_mut();
            let mut csv_read = util::CsvRead::new(&mut read);
            ret.append(
                &mut util::get_street_from_housenumber(&mut csv_read)
                    .with_context(|| format!("get_street_from_housenumber() failed on {}", path))?,
            );
        }
        if sorted_result {
            ret.sort();
            ret.dedup();
        }
        Ok(ret)
    }

    /// Produces a query which lists streets in relation.
    pub fn get_osm_streets_query(&self) -> anyhow::Result<String> {
        let contents = std::fs::read_to_string(format!(
            "{}/{}",
            self.ctx.get_abspath("data")?,
            "streets-template.txt"
        ))?;
        Ok(util::process_template(
            &contents,
            self.config.get_osmrelation(),
        ))
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
    fn get_osm_housenumbers(
        &mut self,
        street_name: &str,
    ) -> anyhow::Result<Vec<util::HouseNumber>> {
        if self.osm_housenumbers.is_empty() {
            // This function gets called for each & every street, make sure we read the file only
            // once.
            let street_ranges = self.get_street_ranges();
            let mut house_numbers: HashMap<String, Vec<util::HouseNumber>> = HashMap::new();
            let stream: Arc<Mutex<dyn Read + Send>> =
                self.file.get_osm_housenumbers_read_stream(&self.ctx)?;
            let mut guard = stream.lock().unwrap();
            let mut read = guard.deref_mut();
            let mut csv_read = util::CsvRead::new(&mut read);
            let mut first = true;
            let mut columns: HashMap<String, usize> = HashMap::new();
            for result in csv_read.records() {
                let row = match result {
                    Ok(value) => value,
                    Err(_) => {
                        continue;
                    }
                };
                if first {
                    first = false;
                    for (index, label) in row.iter().enumerate() {
                        columns.insert(label.into(), index);
                    }
                    continue;
                }
                let mut street = &row[*columns.get("addr:street").unwrap()];
                let street_is_even_odd = self.config.get_street_is_even_odd(street);
                if street.is_empty() {
                    if let Some(value) = columns.get("addr:place") {
                        street = &row[*value];
                    }
                }
                for house_number in row[*columns.get("addr:housenumber").unwrap()].split(';') {
                    if !house_numbers.contains_key(street) {
                        house_numbers.insert(street.into(), Vec::new());
                    }
                    house_numbers
                        .get_mut(street)
                        .unwrap()
                        .append(&mut normalize(
                            self,
                            house_number,
                            street,
                            street_is_even_odd,
                            &street_ranges,
                        )?)
                }
            }
            for (key, value) in house_numbers {
                let unique: Vec<_> = value.into_iter().unique().collect();
                self.osm_housenumbers
                    .insert(key, util::sort_numerically(&unique));
            }
        }
        Ok(match self.osm_housenumbers.get(street_name) {
            Some(value) => value.clone(),
            None => {
                self.osm_housenumbers.insert(street_name.into(), vec![]);
                vec![]
            }
        })
    }

    /// Gets known streets (not their coordinates) from a reference site, based on relation names
    /// from OSM.
    pub fn write_ref_streets(&self, reference: &str) -> anyhow::Result<()> {
        let memory_cache = util::build_street_reference_cache(reference)?;

        let mut lst = self.config.build_ref_streets(&memory_cache);

        lst.sort();
        lst.dedup();
        let write = self.file.get_ref_streets_write_stream(&self.ctx)?;
        let mut guard = write.lock().unwrap();
        for line in lst {
            guard.write_all((line + "\n").as_bytes())?;
        }
        Ok(())
    }

    /// Builds a list of housenumbers from a reference cache.
    /// This is serialized to disk by write_ref_housenumbers().
    fn build_ref_housenumbers(
        &self,
        reference: &util::HouseNumberReferenceCache,
        street: &str,
        suffix: &str,
    ) -> Vec<String> {
        let refcounty = self.config.get_refcounty();
        let street = self.config.get_ref_street_from_osm_street(street);
        let mut ret: Vec<String> = Vec::new();
        for refsettlement in self.config.get_street_refsettlement(&street) {
            let refcounty_dict = match reference.get(&refcounty) {
                Some(value) => value,
                None => {
                    continue;
                }
            };

            let refsettlement_dict = match refcounty_dict.get(&refsettlement) {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            if let Some(value) = refsettlement_dict.get(&street) {
                let house_numbers = value;
                // i[0] is number, i[1] is comment
                ret.append(
                    &mut house_numbers
                        .iter()
                        .map(|i| street.clone() + "\t" + &i[0] + suffix + "\t" + &i[1])
                        .collect(),
                );
            }
        }

        ret
    }

    /// Determines what suffix should the Nth reference use for hours numbers.
    fn get_ref_suffix(index: usize) -> &'static str {
        match index {
            0 => "",
            _ => "*",
        }
    }

    /// Writes known house numbers (not their coordinates) from a reference, based on street names
    /// from OSM. Uses build_reference_cache() to build an indexed reference, the result will be
    /// used by get_ref_housenumbers().
    pub fn write_ref_housenumbers(&self, references: &[String]) -> anyhow::Result<()> {
        let memory_caches = util::build_reference_caches(references, &self.config.get_refcounty())?;

        let streets: Vec<String> = self
            .get_osm_streets(/*sorted_results=*/ true)?
            .iter()
            .map(|i| i.get_osm_name().into())
            .collect();

        let mut lst: Vec<String> = Vec::new();
        for street in streets {
            for (index, memory_cache) in memory_caches.iter().enumerate() {
                let suffix = Relation::get_ref_suffix(index);
                lst.append(&mut self.build_ref_housenumbers(memory_cache, &street, suffix));
            }
        }

        lst.sort();
        lst.dedup();
        let stream = self
            .file
            .get_ref_housenumbers_write_stream(&self.ctx)
            .context("get_ref_housenumbers_write_stream() failed")?;
        let mut guard = stream.lock().unwrap();
        let write = guard.deref_mut();
        for line in lst {
            write.write_all((line + "\n").as_bytes())?;
        }

        Ok(())
    }

    /// Normalizes an 'invalid' list.
    fn normalize_invalids(
        &self,
        osm_street_name: &str,
        street_invalid: &[String],
    ) -> anyhow::Result<Vec<String>> {
        if self.config.should_check_housenumber_letters() {
            return Ok(street_invalid.into());
        }

        let mut normalized_invalids: Vec<String> = Vec::new();
        let street_ranges = self.get_street_ranges();
        let street_is_even_odd = self.config.get_street_is_even_odd(osm_street_name);
        for i in street_invalid {
            let normalizeds =
                normalize(self, i, osm_street_name, street_is_even_odd, &street_ranges)?;
            // normalize() may return an empty list if the number is out of range.
            if !normalizeds.is_empty() {
                normalized_invalids.push(normalizeds[0].get_number().into())
            }
        }
        Ok(normalized_invalids)
    }

    /// Gets house numbers from reference, produced by write_ref_housenumbers()."""
    fn get_ref_housenumbers(&self) -> anyhow::Result<HashMap<String, Vec<util::HouseNumber>>> {
        let mut ret: HashMap<String, Vec<util::HouseNumber>> = HashMap::new();
        let mut lines: HashMap<String, Vec<String>> = HashMap::new();
        let read: Arc<Mutex<dyn Read + Send>> =
            self.file.get_ref_housenumbers_read_stream(&self.ctx)?;
        let mut guard = read.lock().unwrap();
        let stream = std::io::BufReader::new(guard.deref_mut());
        for line in stream.lines() {
            let line = line?;
            let tokens: Vec<&str> = line.splitn(2, '\t').collect();
            let mut iter = tokens.iter();
            let mut key: String = "".into();
            if let Some(value) = iter.next() {
                key = (*value).into();
            }
            let mut value = "";
            if let Some(v) = iter.next() {
                value = v;
            }
            if !lines.contains_key(&key) {
                lines.insert(key.clone(), Vec::new());
            }
            lines.get_mut(&key).unwrap().push(value.into());
        }
        let street_ranges = self.get_street_ranges();
        let streets_invalid = self.get_street_invalid();
        for osm_street in self.get_osm_streets(/*sorted_result=*/ true)? {
            let osm_street_name = osm_street.get_osm_name();
            let street_is_even_odd = self.config.get_street_is_even_odd(osm_street_name);
            let mut house_numbers: Vec<util::HouseNumber> = Vec::new();
            let ref_street_name = self.config.get_ref_street_from_osm_street(osm_street_name);
            let mut street_invalid: Vec<String> = Vec::new();
            if let Some(value) = streets_invalid.get(osm_street_name) {
                street_invalid = value.clone();

                // Simplify invalid items by default, so the 42a markup can be used, no matter what
                // is the value of housenumber-letters.
                street_invalid = self.normalize_invalids(osm_street_name, &street_invalid)?;
            }

            if let Some(value) = lines.get(&ref_street_name) {
                for house_number in value {
                    let normalized = normalize(
                        self,
                        house_number,
                        osm_street_name,
                        street_is_even_odd,
                        &street_ranges,
                    )?;
                    house_numbers.append(
                        &mut normalized
                            .iter()
                            .filter(|i| {
                                !util::HouseNumber::is_invalid(i.get_number(), &street_invalid)
                            })
                            .cloned()
                            .collect(),
                    );
                }
            }
            let unique: Vec<_> = house_numbers.into_iter().unique().collect();
            ret.insert(osm_street_name.into(), util::sort_numerically(&unique));
        }
        Ok(ret)
    }

    /// Compares ref and osm house numbers, prints the ones which are in ref, but not in osm.
    /// Return value is a pair of ongoing and done streets.
    /// Each of of these is a pair of a street name and a house number list.
    pub fn get_missing_housenumbers(
        &mut self,
    ) -> anyhow::Result<(util::NumberedStreets, util::NumberedStreets)> {
        let mut ongoing_streets = Vec::new();
        let mut done_streets = Vec::new();

        let osm_street_names = self.get_osm_streets(/*sorted_result=*/ true)?;
        let all_ref_house_numbers = self.get_ref_housenumbers()?;
        for osm_street in osm_street_names {
            let osm_street_name = osm_street.get_osm_name();
            let ref_house_numbers = all_ref_house_numbers.get(osm_street_name).unwrap();
            let osm_house_numbers = self.get_osm_housenumbers(osm_street_name)?;
            let only_in_reference = util::get_only_in_first(ref_house_numbers, &osm_house_numbers);
            let in_both = util::get_in_both(ref_house_numbers, &osm_house_numbers);
            let ref_street_name = self.config.get_ref_street_from_osm_street(osm_street_name);
            let street = util::Street::new(
                osm_street_name,
                &ref_street_name,
                self.should_show_ref_street(osm_street_name),
                /*osm_id=*/ 0,
            );
            if !only_in_reference.is_empty() {
                ongoing_streets.push((street.clone(), only_in_reference))
            }
            if !in_both.is_empty() {
                done_streets.push((street, in_both));
            }
        }
        // Sort by length, reverse.
        ongoing_streets.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        Ok((ongoing_streets, done_streets))
    }

    /// Tries to find missing streets in a relation.
    pub fn get_missing_streets(&self) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let reference_streets: Vec<util::Street> = self
            .get_ref_streets()?
            .iter()
            .map(|i| util::Street::from_string(i))
            .collect();
        let street_blacklist = self.config.get_street_filters();
        let osm_streets: Vec<util::Street> = self
            .get_osm_streets(/*sorted_result=*/ true)?
            .iter()
            .map(|street| {
                util::Street::from_string(
                    &self
                        .config
                        .get_ref_street_from_osm_street(street.get_osm_name()),
                )
            })
            .collect();

        let only_in_reference = util::get_only_in_first(&reference_streets, &osm_streets);
        let only_in_ref_names: Vec<String> = only_in_reference
            .iter()
            .filter(|i| !street_blacklist.contains(i.get_osm_name()))
            .map(|i| i.get_osm_name())
            .cloned()
            .collect();
        let in_both: Vec<String> = util::get_in_both(&reference_streets, &osm_streets)
            .iter()
            .map(|i| i.get_osm_name())
            .cloned()
            .collect();

        Ok((only_in_ref_names, in_both))
    }

    /// Tries to find additional streets in a relation.
    pub fn get_additional_streets(&self, sorted_result: bool) -> anyhow::Result<Vec<util::Street>> {
        let ref_streets: Vec<String> = self
            .get_ref_streets()?
            .iter()
            .map(|street| self.config.get_osm_street_from_ref_street(street))
            .collect();
        let ref_street_objs: Vec<_> = ref_streets
            .iter()
            .map(|i| util::Street::from_string(i))
            .collect();
        let osm_streets = self.get_osm_streets(sorted_result)?;
        let osm_street_blacklist = self.config.get_osm_street_filters();

        let mut only_in_osm = util::get_only_in_first(&osm_streets, &ref_street_objs);
        only_in_osm = only_in_osm
            .iter()
            .filter(|i| !osm_street_blacklist.contains(i.get_osm_name()))
            .cloned()
            .collect();

        Ok(only_in_osm)
    }

    /// Calculate and write stat for the street coverage of a relation.
    pub fn write_missing_streets(&self) -> anyhow::Result<(usize, usize, String, Vec<String>)> {
        let (todo_streets, done_streets) = self.get_missing_streets()?;
        let streets = todo_streets.clone();
        let todo_count = todo_streets.len();
        let done_count = done_streets.len();
        let percent: String;
        if done_count > 0 || todo_count > 0 {
            let float: f64 = done_count as f64 / (done_count as f64 + todo_count as f64) * 100_f64;
            percent = format!("{0:.2}", float);
        } else {
            percent = "100.00".into();
        }

        // Write the bottom line to a file, so the index page show it fast.
        let write = self.file.get_streets_percent_write_stream(&self.ctx)?;
        let mut guard = write.lock().unwrap();
        guard.write_all(percent.as_bytes())?;

        Ok((todo_count, done_count, percent, streets))
    }

    /// Calculate and write stat for the unexpected street coverage of a relation.
    pub fn write_additional_streets(&self) -> anyhow::Result<Vec<util::Street>> {
        let additional_streets = self.get_additional_streets(/*sorted_result=*/ true)?;

        // Write the count to a file, so the index page show it fast.
        let write = self
            .file
            .get_streets_additional_count_write_stream(&self.ctx)?;
        let mut guard = write.lock().unwrap();
        guard.write_all(additional_streets.len().to_string().as_bytes())?;

        Ok(additional_streets)
    }

    /// Gets a street name -> valid map, which allows silencing individual false positives.
    fn get_street_valid(&self) -> HashMap<String, Vec<String>> {
        let mut valid_dict: HashMap<String, Vec<String>> = HashMap::new();

        let filters = match self.config.get_filters() {
            Some(value) => value,
            None => {
                return valid_dict;
            }
        };
        for (street, street_filter) in filters.as_object().unwrap() {
            if let Some(valid) = street_filter.get("valid") {
                let value: Vec<String> = valid
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|i| i.as_str().unwrap().into())
                    .collect();
                valid_dict.insert(street.clone(), value);
            }
        }

        valid_dict
    }

    /// Turns a list of numbered streets into a HTML table.
    fn numbered_streets_to_table(
        &self,
        numbered_streets: &[util::NumberedStreet],
    ) -> (Vec<Vec<yattag::Doc>>, usize) {
        let mut todo_count = 0_usize;
        let mut table = vec![vec![
            yattag::Doc::from_text(&tr("Street name")),
            yattag::Doc::from_text(&tr("Missing count")),
            yattag::Doc::from_text(&tr("House numbers")),
        ]];
        let mut rows: Vec<Vec<yattag::Doc>> = Vec::new();
        for result in numbered_streets {
            // street, only_in_ref
            let mut row: Vec<yattag::Doc> = vec![result.0.to_html()];
            let number_ranges = util::get_housenumber_ranges(&result.1);
            row.push(yattag::Doc::from_text(&number_ranges.len().to_string()));

            let doc = yattag::Doc::new();
            if !self.config.get_street_is_even_odd(result.0.get_osm_name()) {
                let mut sorted = number_ranges.clone();
                sorted.sort_by(|a, b| {
                    util::split_house_number_range(a).cmp(&util::split_house_number_range(b))
                });
                for (index, item) in sorted.iter().enumerate() {
                    if index > 0 {
                        doc.text(", ");
                    }
                    doc.append_value(util::color_house_number(item).get_value());
                }
            } else {
                doc.append_value(util::format_even_odd_html(&number_ranges).get_value());
            }
            row.push(doc);

            todo_count += number_ranges.len();
            rows.push(row);
        }

        // It's possible that get_housenumber_ranges() reduces the # of house numbers, e.g. 2, 4 and
        // 6 may be turned into 2-6, which is just 1 item. Sort by the 2nd col, which is the new
        // number of items.
        rows.sort_by(|cells_a, cells_b| {
            // Reverse.
            cells_b[1]
                .get_value()
                .parse::<usize>()
                .unwrap()
                .cmp(&cells_a[1].get_value().parse::<usize>().unwrap())
        });
        table.append(&mut rows);
        (table, todo_count)
    }

    /// Calculate a write stat for the house number coverage of a relation.
    /// Returns a tuple of: todo street count, todo count, done count, percent and table.
    pub fn write_missing_housenumbers(
        &mut self,
    ) -> anyhow::Result<(usize, usize, usize, String, yattag::HtmlTable)> {
        let (ongoing_streets, done_streets) = self.get_missing_housenumbers()?;

        let (table, todo_count) = self.numbered_streets_to_table(&ongoing_streets);

        let mut done_count = 0;
        for result in done_streets {
            let number_ranges = util::get_housenumber_ranges(&result.1);
            done_count += number_ranges.len();
        }
        let percent: String;
        if done_count > 0 || todo_count > 0 {
            let float: f64 = done_count as f64 / (done_count as f64 + todo_count as f64) * 100_f64;
            percent = format!("{0:.2}", float);
        } else {
            percent = "100.00".into();
        }

        // Write the bottom line to a file, so the index page show it fast.
        let write = self
            .file
            .get_housenumbers_percent_write_stream(&self.ctx)
            .context("get_housenumbers_percent_write_stream() failed")?;
        let mut guard = write.lock().unwrap();
        guard.write_all(percent.as_bytes())?;

        Ok((
            ongoing_streets.len(),
            todo_count,
            done_count,
            percent,
            table,
        ))
    }

    /// Compares ref and osm house numbers, prints the ones which are in osm, but not in ref.
    /// Return value is a list of streets.
    /// Each of of these is a pair of a street name and a house number list.
    fn get_additional_housenumbers(&mut self) -> anyhow::Result<util::NumberedStreets> {
        let mut additional = Vec::new();

        let osm_street_names = self.get_osm_streets(/*sorted_result=*/ true)?;
        let all_ref_house_numbers = self.get_ref_housenumbers()?;
        let streets_valid = self.get_street_valid();
        for osm_street in osm_street_names {
            let osm_street_name = osm_street.get_osm_name();
            let ref_house_numbers = all_ref_house_numbers.get(osm_street_name).unwrap();
            let mut osm_house_numbers = self.get_osm_housenumbers(osm_street_name)?;

            if let Some(street_valid) = streets_valid.get(osm_street_name) {
                let filtered: Vec<_> = osm_house_numbers
                    .iter()
                    .filter(|i| !util::HouseNumber::is_invalid(i.get_number(), street_valid))
                    .cloned()
                    .collect();
                osm_house_numbers = filtered;
            }

            let only_in_osm = util::get_only_in_first(&osm_house_numbers, ref_house_numbers);
            let ref_street_name = self.config.get_ref_street_from_osm_street(osm_street_name);
            let street = util::Street::new(
                osm_street_name,
                &ref_street_name,
                self.should_show_ref_street(osm_street_name),
                /*osm_id=*/ 0,
            );
            if !only_in_osm.is_empty() {
                additional.push((street, only_in_osm))
            }
        }
        // Sort by length, reverse.
        additional.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        Ok(additional)
    }

    /// Calculate and write stat for the unexpected house number coverage of a relation.
    /// Returns a tuple of: todo street count, todo count and table.
    pub fn write_additional_housenumbers(
        &mut self,
    ) -> anyhow::Result<(usize, usize, yattag::HtmlTable)> {
        let ongoing_streets = self.get_additional_housenumbers()?;

        let (table, todo_count) = self.numbered_streets_to_table(&ongoing_streets);

        // Write the street count to a file, so the index page show it fast.
        let write = self
            .file
            .get_housenumbers_additional_count_write_stream(&self.ctx)?;
        let mut guard = write.lock().unwrap();
        guard.write_all(todo_count.to_string().as_bytes())?;

        Ok((ongoing_streets.len(), todo_count, table))
    }

    /// Produces a query which lists house numbers in relation.
    pub fn get_osm_housenumbers_query(&self) -> anyhow::Result<String> {
        let contents = std::fs::read_to_string(format!(
            "{}/{}",
            self.ctx.get_abspath("data")?,
            "street-housenumbers-template.txt"
        ))?;
        Ok(util::process_template(
            &contents,
            self.config.get_osmrelation(),
        ))
    }

    /// Returns invalid osm names and ref names.
    pub fn get_invalid_refstreets(&self) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let mut osm_invalids: Vec<String> = Vec::new();
        let mut ref_invalids: Vec<String> = Vec::new();
        let refstreets = self.config.get_refstreets();
        let osm_streets: Vec<String> = self
            .get_osm_streets(/*sorted_result=*/ true)
            .context("get_osm_streets() failed")?
            .iter()
            .map(|i| i.get_osm_name())
            .cloned()
            .collect();
        for (osm_name, ref_name) in refstreets {
            if !osm_streets.contains(&osm_name) {
                osm_invalids.push(osm_name);
            }
            if osm_streets.contains(&ref_name) {
                ref_invalids.push(ref_name);
            }
        }
        Ok((osm_invalids, ref_invalids))
    }

    /// Returns invalid filter key names (street not in OSM).
    pub fn get_invalid_filter_keys(&self) -> anyhow::Result<Vec<String>> {
        let filters = match self.config.get_filters() {
            Some(value) => value,
            None => {
                return Ok(Vec::new());
            }
        }
        .as_object()
        .unwrap();
        let keys: Vec<String> = filters.iter().map(|(key, _value)| key.clone()).collect();
        let osm_streets: Vec<String> = self
            .get_osm_streets(/*sorted_result=*/ true)?
            .iter()
            .map(|i| i.get_osm_name())
            .cloned()
            .collect();
        Ok(keys
            .iter()
            .filter(|key| !osm_streets.contains(key))
            .cloned()
            .collect())
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyRelation {
    pub relation: Relation,
}

#[pymethods]
impl PyRelation {
    fn get_name(&self) -> String {
        self.relation.get_name()
    }

    fn get_files(&self) -> area_files::PyRelationFiles {
        let relation_files = self.relation.get_files().clone();
        area_files::PyRelationFiles { relation_files }
    }

    fn get_config(&self) -> PyRelationConfig {
        let relation_config = self.relation.config.clone();
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

    fn get_osm_housenumbers(&mut self, street_name: String) -> PyResult<Vec<util::PyHouseNumber>> {
        let ret = match self.relation.get_osm_housenumbers(&street_name) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_osm_housenumbers() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(ret
            .iter()
            .map(|i| {
                let house_number = i.clone();
                util::PyHouseNumber { house_number }
            })
            .collect())
    }

    fn write_ref_streets(&self, reference: &str) -> PyResult<()> {
        match self.relation.write_ref_streets(reference) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "write_ref_streets() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_street_ranges(&self) -> HashMap<String, ranges::PyRanges> {
        let mut ret: HashMap<String, ranges::PyRanges> = HashMap::new();
        for (key, value) in self.relation.get_street_ranges() {
            ret.insert(key, ranges::PyRanges { ranges: value });
        }
        ret
    }

    fn should_show_ref_street(&self, osm_street_name: String) -> bool {
        self.relation.should_show_ref_street(&osm_street_name)
    }

    fn get_osm_streets(&self, sorted_result: bool) -> PyResult<Vec<util::PyStreet>> {
        let ret = match self.relation.get_osm_streets(sorted_result) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_osm_streets() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(ret
            .iter()
            .map(|i| {
                let street = i.clone();
                util::PyStreet { street }
            })
            .collect())
    }

    fn get_osm_streets_query(&self) -> PyResult<String> {
        match self.relation.get_osm_streets_query() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_osm_streets_query() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn build_ref_housenumbers(
        &self,
        reference: util::HouseNumberReferenceCache,
        street: &str,
        suffix: &str,
    ) -> Vec<String> {
        self.relation
            .build_ref_housenumbers(&reference, street, suffix)
    }

    fn write_ref_housenumbers(&self, references: Vec<String>) -> PyResult<()> {
        match self
            .relation
            .write_ref_housenumbers(&references)
            .context("write_ref_housenumbers() failed")
        {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
        }
    }

    fn get_missing_housenumbers(
        &mut self,
    ) -> PyResult<(util::PyNumberedStreets, util::PyNumberedStreets)> {
        let (ongoing_streets, done_streets) = match self.relation.get_missing_housenumbers() {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_missing_housenumbers() failed: {}",
                    err.to_string()
                )));
            }
        };
        let mut py_ongoing_streets: Vec<(util::PyStreet, Vec<util::PyHouseNumber>)> = Vec::new();
        for street in ongoing_streets {
            let py_street = util::PyStreet { street: street.0 };
            let py_housenumbers: Vec<util::PyHouseNumber> = street
                .1
                .iter()
                .map(|i| util::PyHouseNumber {
                    house_number: i.clone(),
                })
                .collect();
            py_ongoing_streets.push((py_street, py_housenumbers));
        }
        let mut py_done_streets: Vec<(util::PyStreet, Vec<util::PyHouseNumber>)> = Vec::new();
        for street in done_streets {
            let py_street = util::PyStreet { street: street.0 };
            let py_housenumbers: Vec<util::PyHouseNumber> = street
                .1
                .iter()
                .map(|i| util::PyHouseNumber {
                    house_number: i.clone(),
                })
                .collect();
            py_done_streets.push((py_street, py_housenumbers));
        }
        Ok((py_ongoing_streets, py_done_streets))
    }

    fn get_missing_streets(&self) -> PyResult<(Vec<String>, Vec<String>)> {
        match self.relation.get_missing_streets() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_missing_streets() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_additional_streets(&self, sorted_result: bool) -> PyResult<Vec<util::PyStreet>> {
        let ret = match self.relation.get_additional_streets(sorted_result) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_additional_streets() failed: {}",
                    err.to_string()
                )));
            }
        };

        Ok(ret
            .iter()
            .map(|i| util::PyStreet { street: i.clone() })
            .collect())
    }

    fn write_missing_streets(&self) -> PyResult<(usize, usize, String, Vec<String>)> {
        match self.relation.write_missing_streets() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "write_missing_streets() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn write_additional_streets(&self) -> PyResult<Vec<util::PyStreet>> {
        let ret = match self.relation.write_additional_streets() {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "write_additional_streets() failed: {}",
                    err.to_string()
                )));
            }
        };

        Ok(ret
            .iter()
            .map(|i| util::PyStreet { street: i.clone() })
            .collect())
    }

    fn write_missing_housenumbers(
        &mut self,
    ) -> PyResult<(usize, usize, usize, String, yattag::PyHtmlTable)> {
        let (ongoing_len, todo, done, percent, table) = match self
            .relation
            .write_missing_housenumbers()
            .context("write_missing_housenumbers() failed")
        {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err)));
            }
        };
        let py_table: Vec<Vec<yattag::PyDoc>> = table
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| yattag::PyDoc { doc: cell.clone() })
                    .collect()
            })
            .collect();
        Ok((ongoing_len, todo, done, percent, py_table))
    }

    fn get_additional_housenumbers(&mut self) -> PyResult<util::PyNumberedStreets> {
        let ret = match self.relation.get_additional_housenumbers() {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_additional_housenumbers() failed: {}",
                    err.to_string()
                )));
            }
        };
        let mut py_ret: Vec<(util::PyStreet, Vec<util::PyHouseNumber>)> = Vec::new();
        for street in ret {
            let py_street = util::PyStreet { street: street.0 };
            let py_housenumbers: Vec<util::PyHouseNumber> = street
                .1
                .iter()
                .map(|i| util::PyHouseNumber {
                    house_number: i.clone(),
                })
                .collect();
            py_ret.push((py_street, py_housenumbers));
        }
        Ok(py_ret)
    }

    fn get_osm_housenumbers_query(&self) -> PyResult<String> {
        match self.relation.get_osm_housenumbers_query() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_osm_housenumbers_query() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_invalid_refstreets(&self) -> PyResult<(Vec<String>, Vec<String>)> {
        match self.relation.get_invalid_refstreets() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_invalid_refstreets() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_invalid_filter_keys(&self) -> PyResult<Vec<String>> {
        match self.relation.get_invalid_filter_keys() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_invalid_filter_keys() failed: {}",
                err.to_string()
            ))),
        }
    }
}

/// A relations object is a container of named relation objects.
#[derive(Clone)]
pub struct Relations {
    workdir: String,
    ctx: context::Context,
    yaml_cache: serde_json::Map<String, serde_json::Value>,
    dict: serde_json::Map<String, serde_json::Value>,
    relations: HashMap<String, Relation>,
    activate_all: bool,
    refcounty_names: HashMap<String, String>,
    refsettlement_names: HashMap<String, HashMap<String, String>>,
}

impl Relations {
    pub fn new(ctx: &context::Context) -> anyhow::Result<Self> {
        let workdir = ctx.get_ini().get_workdir()?;
        let stream = ctx.get_file_system().open_read(&format!(
            "{}/{}",
            ctx.get_abspath("data")?,
            "yamls.cache"
        ))?;
        let mut guard = stream.lock().unwrap();
        let read = guard.deref_mut();
        let value: serde_json::Value = serde_json::from_reader(read)?;
        let yaml_cache = value.as_object().unwrap();
        let dict = yaml_cache
            .get("relations.yaml")
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        let relations: HashMap<String, Relation> = HashMap::new();
        let activate_all = false;
        let refcounty_names: HashMap<String, String> = yaml_cache
            .get("refcounty-names.yaml")
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.as_str().unwrap().into()))
            .collect();
        let refsettlement_names: HashMap<String, HashMap<String, String>> = yaml_cache
            .get("refsettlement-names.yaml")
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| {
                let value: HashMap<String, String> = value
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(key, value)| (key.clone(), value.as_str().unwrap().into()))
                    .collect();
                (key.clone(), value)
            })
            .collect();
        Ok(Relations {
            workdir,
            ctx: ctx.clone(),
            yaml_cache: yaml_cache.clone(),
            dict,
            relations,
            activate_all,
            refcounty_names,
            refsettlement_names,
        })
    }

    /// Gets the workdir directory path.
    fn get_workdir(&self) -> &String {
        &self.workdir
    }

    /// Gets the relation that has the specified name.
    pub fn get_relation(&mut self, name: &str) -> anyhow::Result<Relation> {
        if !self.relations.contains_key(name) {
            if !self.dict.contains_key(name) {
                self.dict.insert(name.into(), serde_json::json!({}));
            }
            let relation = Relation::new(
                &self.ctx,
                name,
                self.dict.get(name).unwrap(),
                &self.yaml_cache,
            )?;
            self.relations.insert(name.into(), relation);
        }

        Ok(self.relations.get(name).unwrap().clone())
    }

    /// Sets a relation for testing.
    fn set_relation(&mut self, name: &str, relation: &Relation) {
        self.relations.insert(name.into(), relation.clone());
    }

    /// Gets a sorted list of relation names.
    pub fn get_names(&self) -> Vec<String> {
        let mut ret: Vec<String> = self.dict.iter().map(|(key, _value)| key.into()).collect();
        ret.sort();
        ret.dedup();
        ret
    }

    /// Gets a sorted list of active relation names.
    fn get_active_names(&mut self) -> anyhow::Result<Vec<String>> {
        let mut active_relations: Vec<Relation> = Vec::new();
        for relation in self.get_relations()? {
            if self.activate_all || relation.config.is_active() {
                active_relations.push(relation.clone())
            }
        }
        let mut ret: Vec<String> = active_relations
            .iter()
            .map(|relation| relation.get_name())
            .collect();
        ret.sort();
        ret.dedup();
        Ok(ret)
    }

    /// Gets a list of relations.
    pub fn get_relations(&mut self) -> anyhow::Result<Vec<Relation>> {
        let mut ret: Vec<Relation> = Vec::new();
        for name in self.get_names() {
            ret.push(self.get_relation(&name)?)
        }
        Ok(ret)
    }

    /// Produces a UI name for a refcounty.
    pub fn refcounty_get_name(&self, refcounty: &str) -> String {
        match self.refcounty_names.get(refcounty) {
            Some(value) => value.into(),
            None => "".into(),
        }
    }

    /// Produces a UI name for a refsettlement in refcounty.
    pub fn refsettlement_get_name(&self, refcounty_name: &str, refsettlement: &str) -> String {
        let refcounty = match self.refsettlement_names.get(refcounty_name) {
            Some(value) => value,
            None => {
                return "".into();
            }
        };
        match refcounty.get(refsettlement) {
            Some(value) => value.clone(),
            None => "".into(),
        }
    }

    /// Sets if inactive=true is ignored or not.
    fn activate_all(&mut self, activate_all: bool) {
        self.activate_all = activate_all;
    }

    /// If refcounty is not None, forget about all relations outside that refcounty.
    fn limit_to_refcounty(&mut self, refcounty: &Option<String>) -> anyhow::Result<()> {
        let refcounty: String = match refcounty {
            Some(value) => value.clone(),
            None => {
                return Ok(());
            }
        };
        let relation_names: Vec<String> =
            self.dict.iter().map(|(key, _value)| key.clone()).collect();
        for relation_name in relation_names {
            let relation = self.get_relation(&relation_name)?;
            if relation.config.get_refcounty() == refcounty {
                continue;
            }
            self.dict.remove(&relation_name);
        }

        Ok(())
    }

    /// If refsettlement is not None, forget about all relations outside that refsettlement.
    fn limit_to_refsettlement(&mut self, refsettlement: &Option<String>) -> anyhow::Result<()> {
        let refsettlement: String = match refsettlement {
            Some(value) => value.clone(),
            None => {
                return Ok(());
            }
        };
        let relation_names: Vec<String> =
            self.dict.iter().map(|(key, _value)| key.clone()).collect();
        for relation_name in relation_names {
            let relation = self.get_relation(&relation_name)?;
            if relation.config.get_refsettlement() == refsettlement {
                continue;
            }
            self.dict.remove(&relation_name);
        }

        Ok(())
    }

    /// Produces refsettlement IDs of a refcounty.
    pub fn refcounty_get_refsettlement_ids(&self, refcounty_name: &str) -> Vec<String> {
        let refcounty = match self.refsettlement_names.get(refcounty_name) {
            Some(value) => value,
            None => {
                return Vec::new();
            }
        };
        let mut ret: Vec<String> = refcounty.iter().map(|(key, _value)| key.clone()).collect();
        ret.sort();
        ret
    }

    /// Provide an alias -> real name map of relations.
    pub fn get_aliases(&mut self) -> anyhow::Result<HashMap<String, String>> {
        let mut ret: HashMap<String, String> = HashMap::new();
        for relation in self.get_relations()? {
            let aliases = relation.config.get_alias();
            if !aliases.is_empty() {
                let name = relation.get_name();
                for alias in aliases {
                    ret.insert(alias, name.clone());
                }
            }
        }
        Ok(ret)
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyRelations {
    pub relations: Relations,
}

#[pymethods]
impl PyRelations {
    #[new]
    fn new(ctx: PyObject) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let ctx: PyRefMut<'_, context::PyContext> = ctx.extract(gil.python())?;
        let relations = match Relations::new(&ctx.context) {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "Relations::new() failed: {}",
                    err.to_string()
                )));
            }
        };
        Ok(PyRelations { relations })
    }

    fn get_workdir(&self) -> String {
        self.relations.get_workdir().clone()
    }

    fn get_relation(&mut self, name: &str) -> PyResult<PyRelation> {
        match self.relations.get_relation(name) {
            Ok(value) => Ok(PyRelation { relation: value }),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_relation() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn set_relation(&mut self, name: &str, relation: PyRelation) {
        self.relations.set_relation(name, &relation.relation)
    }

    fn get_names(&self) -> Vec<String> {
        self.relations.get_names()
    }

    fn get_active_names(&mut self) -> PyResult<Vec<String>> {
        match self.relations.get_active_names() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_active_names() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn refcounty_get_name(&self, refcounty: &str) -> String {
        self.relations.refcounty_get_name(refcounty)
    }

    fn refsettlement_get_name(&self, refcounty_name: &str, refsettlement: &str) -> String {
        self.relations
            .refsettlement_get_name(refcounty_name, refsettlement)
    }

    fn activate_all(&mut self, activate_all: bool) {
        self.relations.activate_all(activate_all);
    }

    fn limit_to_refcounty(&mut self, refcounty: Option<String>) -> PyResult<()> {
        match self.relations.limit_to_refcounty(&refcounty) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "limit_to_refcounty() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn limit_to_refsettlement(&mut self, refsettlement: Option<String>) -> PyResult<()> {
        match self.relations.limit_to_refsettlement(&refsettlement) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "limit_to_refsettlement() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_relations(&mut self) -> PyResult<Vec<PyRelation>> {
        let ret = match self.relations.get_relations() {
            Ok(value) => value,
            Err(err) => {
                return Err(pyo3::exceptions::PyOSError::new_err(format!(
                    "get_relations() failed: {}",
                    err.to_string()
                )));
            }
        };

        Ok(ret
            .iter()
            .map(|i| PyRelation {
                relation: i.clone(),
            })
            .collect::<Vec<PyRelation>>())
    }

    fn refcounty_get_refsettlement_ids(&self, refcounty_name: &str) -> Vec<String> {
        self.relations
            .refcounty_get_refsettlement_ids(refcounty_name)
    }

    fn get_aliases(&mut self) -> PyResult<HashMap<String, String>> {
        match self.relations.get_aliases() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_aliases() failed: {}",
                err.to_string()
            ))),
        }
    }
}

/// Strips down string input to bare minimum that can be interpreted as an
/// actual number. Think about a/b, a-b, and so on.
fn normalize(
    relation: &Relation,
    house_numbers: &str,
    street_name: &str,
    street_is_even_odd: bool,
    normalizers: &HashMap<String, ranges::Ranges>,
) -> anyhow::Result<Vec<util::HouseNumber>> {
    let mut comment: String = "".into();
    let mut house_numbers: String = house_numbers.into();
    if house_numbers.contains('\t') {
        let tokens = house_numbers.clone();
        let mut iter = tokens.split('\t');
        house_numbers = iter.next().unwrap().into();
        comment = iter.next().unwrap().into();
    }
    let separator: &str;
    if house_numbers.contains(';') {
        separator = ";";
    } else if house_numbers.contains(',') {
        separator = ",";
    } else {
        separator = "-";
    }

    // Determine suffix which is not normalized away.
    let mut suffix: String = "".into();
    if house_numbers.ends_with('*') {
        suffix = house_numbers.chars().last().unwrap().into();
    }

    let normalizer = util::get_normalizer(street_name, normalizers);

    let (mut ret_numbers, ret_numbers_nofilter) =
        util::split_house_number_by_separator(&house_numbers, separator, &normalizer);

    if separator == "-" {
        let (should_expand, new_stop) =
            util::should_expand_range(&ret_numbers_nofilter, street_is_even_odd);
        if should_expand {
            let start = ret_numbers_nofilter[0];
            let stop = new_stop;
            if stop == 0 {
                ret_numbers = vec![start]
                    .iter()
                    .filter(|number| normalizer.contains(**number))
                    .cloned()
                    .collect();
            } else if street_is_even_odd {
                // Assume that e.g. 2-6 actually means 2, 4 and 6, not only 2 and 4.
                // Closed interval, even only or odd only case.
                //ret_numbers = [number for number in range(start, stop + 2, 2) if number in normalizer]
                ret_numbers = (start..stop + 2)
                    .step_by(2)
                    .filter(|number| normalizer.contains(*number))
                    .collect();
            } else {
                // Closed interval, but mixed even and odd.
                ret_numbers = (start..stop + 1)
                    .filter(|number| normalizer.contains(*number))
                    .collect();
            }
        }
    }

    let check_housenumber_letters =
        ret_numbers.len() == 1 && relation.config.should_check_housenumber_letters();
    if check_housenumber_letters && util::HouseNumber::has_letter_suffix(&house_numbers, &suffix) {
        return normalize_housenumber_letters(relation, &house_numbers, &suffix, &comment);
    }
    Ok(ret_numbers
        .iter()
        .map(|number| {
            util::HouseNumber::new(&(number.to_string() + &suffix), &house_numbers, &comment)
        })
        .collect())
}

/// Handles the part of normalize() that deals with housenumber letters.
fn normalize_housenumber_letters(
    relation: &Relation,
    house_numbers: &str,
    suffix: &str,
    comment: &str,
) -> anyhow::Result<Vec<util::HouseNumber>> {
    let style =
        util::LetterSuffixStyle::try_from(relation.config.get_letter_suffix_style()).unwrap();
    let normalized = util::HouseNumber::normalize_letter_suffix(house_numbers, suffix, style)?;
    Ok(vec![util::HouseNumber::new(
        &normalized,
        &normalized,
        comment,
    )])
}

/// Creates an overpass query that shows all streets from a missing housenumbers table.
pub fn make_turbo_query_for_streets(relation: &Relation, streets: &[String]) -> String {
    let header = r#"[out:json][timeout:425];
rel(@RELATION@)->.searchRelation;
area(@AREA@)->.searchArea;
(rel(@RELATION@);
"#;
    let mut query = util::process_template(header, relation.config.get_osmrelation());
    for street in streets {
        query += &format!("way[\"name\"=\"{}\"](r.searchRelation);\n", street);
        query += &format!("way[\"name\"=\"{}\"](area.searchArea);\n", street);
    }
    query += r#");
out body;
>;
out skel qt;
{{style:
relation{width:3}
way{color:blue; width:4;}
}}"#;
    query
}

#[pyfunction]
fn py_make_turbo_query_for_streets(relation: PyRelation, streets: Vec<String>) -> String {
    make_turbo_query_for_streets(&relation.relation, &streets)
}

/// Creates an overpass query that shows all streets from a list.
pub fn make_turbo_query_for_street_objs(relation: &Relation, streets: &[util::Street]) -> String {
    let header = r#"[out:json][timeout:425];
rel(@RELATION@)->.searchRelation;
area(@AREA@)->.searchArea;
("#;
    let mut query = util::process_template(header, relation.config.get_osmrelation());
    let mut ids = Vec::new();
    for street in streets {
        ids.push((street.get_osm_type(), street.get_osm_id().to_string()));
    }
    ids.sort();
    ids.dedup();
    for (osm_type, osm_id) in ids {
        query += &format!("{}({});\n", osm_type, osm_id);
    }
    query += r#");
out body;
>;
out skel qt;"#;
    query
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyRelationConfig>()?;
    module.add_class::<PyRelation>()?;
    module.add_class::<PyRelations>()?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_make_turbo_query_for_streets,
        module
    )?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Seek;
    use std::io::SeekFrom;

    /// Tests normalize().
    #[test]
    fn test_normalize() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "139",
            "Budaörsi út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["139"])
    }

    /// Tests normalize: when the number is not in range.
    #[test]
    fn test_normalize_not_in_range() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "999",
            "Budaörsi út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        assert_eq!(house_numbers.is_empty(), true);
    }

    /// Tests normalize: the case when the house number is not a number.
    #[test]
    fn test_normalize_not_a_number() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "x",
            "Budaörsi út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        assert_eq!(house_numbers.is_empty(), true);
    }

    /// Tests normalize: the case when there is no filter for this street.
    #[test]
    fn test_normalize_nofilter() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "1",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["1"])
    }

    /// Tests normalize: the case when ';' is a separator.
    #[test]
    fn test_normalize_separator_semicolon() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "1;2",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["1", "2"])
    }

    /// Tests normalize: the 2-6 case means implicit 4.
    #[test]
    fn test_normalize_separator_interval() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "2-6",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["2", "4", "6"])
    }

    /// Tests normalize: the 5-8 case: means just 5 and 8 as the parity doesn't match.
    #[test]
    fn test_normalize_separator_interval_parity() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "5-8",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["5", "8"])
    }

    /// Tests normalize: the 2-5 case: means implicit 3 and 4 (interpolation=all).
    #[test]
    fn test_normalize_separator_interval_interp_all() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Hamzsabégi út");
        let house_numbers = normalize(
            &relation,
            "2-5",
            "Hamzsabégi út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["2", "3", "4", "5"])
    }

    /// Tests normalize: the case where x-y is partially filtered out.
    #[test]
    fn test_normalize_separator_interval_filter() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        // filter is 137-165
        let house_numbers = normalize(
            &relation,
            "163-167",
            "Budaörsi út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        // Make sure there is no 167.
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["163", "165"])
    }

    /// Tests normalize: the case where x-y is nonsense: y is too large.
    #[test]
    fn test_normalize_separator_interval_block() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "2-2000",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        // Make sure that we simply ignore 2000: it's larger than the default <998 filter and the
        // 2-2000 range would be too large.
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["2"])
    }

    /// Tests normalize: the case where x-y is nonsense: y-x is too large.
    #[test]
    fn test_normalize_separator_interval_block2() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "2-56",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        // No expansions for 4, 6, etc.
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["2", "56"])
    }

    /// Tests normalize: the case where x-y is nonsense: x is 0.
    #[test]
    fn test_normalize_separator_interval_block3() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "0-42",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        // No expansion like 0, 2, 4, etc.
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["42"])
    }

    /// Tests normalize: the case where x-y is only partially useful: x is OK, but y is a suffix.
    #[test]
    fn test_normalize_separator_interval_block4() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "42-1",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        // No "1", just "42".
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["42"])
    }

    /// Tests normalize: the * suffix is preserved.
    #[test]
    fn test_normalize_keep_suffix() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "1*",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["1*"]);
        let house_numbers = normalize(
            &relation,
            "2",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, vec!["2"]);
    }

    /// Tests normalize: the case when ',' is a separator.
    #[test]
    fn test_normalize_separator_comma() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let normalizers = relation.get_street_ranges();
        let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
        let house_numbers = normalize(
            &relation,
            "2,6",
            "Budaörs út",
            street_is_even_odd,
            &normalizers,
        )
        .unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        // Same as ";", no 4.
        assert_eq!(actual, vec!["2", "6"]);
    }

    /// Tests Relation.get_osm_streets().
    #[test]
    fn test_relation_get_osm_streets() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("test").unwrap();
        let actual: Vec<String> = relation
            .get_osm_streets(/*sorted_result=*/ true)
            .unwrap()
            .iter()
            .map(|i| i.get_osm_name().clone())
            .collect();
        let expected: Vec<String> = vec!["B1".into(), "B2".into(), "HB1".into(), "HB2".into()];
        assert_eq!(actual, expected);
    }

    /// Tests Relation.get_osm_streets(): the case when the street name is coming from a house
    /// number (node).
    #[test]
    fn test_relation_get_osm_streets_street_is_node() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gh830").unwrap();
        let actual = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].get_osm_type(), "node");
    }

    /// Tests Relation.get_osm_streets(): the case when we have streets, but no house numbers.
    #[test]
    fn test_relation_get_osm_streets_no_house_number() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("ujbuda").unwrap();
        let osm_streets = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
        let actual: Vec<_> = osm_streets.iter().map(|i| i.get_osm_name()).collect();
        let expected = vec!["OSM Name 1", "Törökugrató utca", "Tűzkő utca"];
        assert_eq!(actual, expected);
    }

    /// Tests Relation.get_osm_streets(): when there is only an addr:conscriptionnumber.
    #[test]
    fn test_relation_get_osm_streets_conscriptionnumber() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gh754").unwrap();
        let osm_streets = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
        let streets: Vec<_> = osm_streets.iter().map(|i| i.get_osm_name()).collect();
        // This is coming from a house number which has addr:street and addr:conscriptionnumber, but
        // no addr:housenumber.
        let expected: &String = &String::from("Barcfa dűlő");
        assert_eq!(streets.contains(&expected), true);
    }

    /// Tests Relation.get_osm_streets_query().
    #[test]
    fn test_relation_get_osm_streets_query() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        assert_eq!(
            relations.get_workdir(),
            &ctx.get_abspath("workdir").unwrap()
        );
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let ret = relation.get_osm_streets_query().unwrap();
        assert_eq!(ret, "aaa 2713748 bbb 3602713748 ccc\n");
    }

    /// Tests Relation.get_osm_housenumbers_query().
    #[test]
    fn test_relation_get_osm_housenumbers_query() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let ret = relation.get_osm_housenumbers_query().unwrap();
        assert_eq!(ret, "housenr aaa 2713748 bbb 3602713748 ccc\n");
    }

    /// Tests RelationFiles.write_osm_streets().
    #[test]
    fn test_relation_files_write_osm_streets() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let result_from_overpass =
            "@id\tname\n1\tTűzkő utca\n2\tTörökugrató utca\n3\tOSM Name 1\n4\tHamzsabégi út\n";
        let expected = util::get_content("tests/workdir/streets-gazdagret.csv").unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let streets_value: Arc<Mutex<std::io::Cursor<Vec<u8>>>> =
            Arc::new(Mutex::new(std::io::Cursor::new(Vec::new())));
        let mut files: HashMap<String, Arc<Mutex<std::io::Cursor<Vec<u8>>>>> = HashMap::new();
        files.insert(
            ctx.get_abspath("workdir/streets-gazdagret.csv").unwrap(),
            streets_value.clone(),
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        relation
            .get_files()
            .write_osm_streets(&ctx, result_from_overpass)
            .unwrap();
        let mut guard = streets_value.lock().unwrap();
        guard.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        guard.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, expected);
    }

    /// Tests RelationFiles.write_osm_housenumbers().
    #[test]
    fn test_relation_files_write_osm_housenumbers() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let result_from_overpass =
            "@id\taddr:street\taddr:housenumber\taddr:postcode\taddr:housename\t\
addr:conscriptionnumber\taddr:flats\taddr:floor\taddr:door\taddr:unit\tname\t@type\n\n\
1\tTörökugrató utca\t1\t\t\t\t\t\t\t\t\tnode\n\
1\tTörökugrató utca\t2\t\t\t\t\t\t\t\t\tnode\n\
1\tTűzkő utca\t9\t\t\t\t\t\t\t\t\tnode\n\
1\tTűzkő utca\t10\t\t\t\t\t\t\t\t\tnode\n\
1\tOSM Name 1\t1\t\t\t\t\t\t\t\t\tnode\n\
1\tOSM Name 1\t2\t\t\t\t\t\t\t\t\tnode\n\
1\tOnly In OSM utca\t1\t\t\t\t\t\t\t\t\tnode\n\
1\tSecond Only In OSM utca\t1\t\t\t\t\t\t\t\t\tnode\n";
        let expected = String::from_utf8(
            util::get_content("tests/workdir/street-housenumbers-gazdagret.csv").unwrap(),
        )
        .unwrap();
        let relation = relations.get_relation(relation_name).unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let housenumbers_value: Arc<Mutex<std::io::Cursor<Vec<u8>>>> =
            Arc::new(Mutex::new(std::io::Cursor::new(Vec::new())));
        let mut files: HashMap<String, Arc<Mutex<std::io::Cursor<Vec<u8>>>>> = HashMap::new();
        files.insert(
            ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv")
                .unwrap(),
            housenumbers_value.clone(),
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        relation
            .get_files()
            .write_osm_housenumbers(&ctx, result_from_overpass)
            .unwrap();
        let mut guard = housenumbers_value.lock().unwrap();
        guard.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        guard.read_to_end(&mut actual).unwrap();
        assert_eq!(String::from_utf8(actual).unwrap(), expected);
    }
}
