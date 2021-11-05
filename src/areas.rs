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
    pub fn set_active(&mut self, active: bool) {
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
    pub fn get_refsettlement(&self) -> String {
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
    pub fn should_show_ref_street(&self, osm_street_name: &str) -> bool {
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
    fn get_street_ranges(&self) -> anyhow::Result<HashMap<String, ranges::Ranges>> {
        let mut filter_dict: HashMap<String, ranges::Ranges> = HashMap::new();

        let filters = match self.config.get_filters() {
            Some(value) => value,
            None => {
                return Ok(filter_dict);
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
                        .trim()
                        .parse::<i64>()
                        .context("failed to parse() 'start'")?;
                    let end = start_end_obj
                        .get("end")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .trim()
                        .parse::<i64>()
                        .context("failed to parse() 'end'")?;
                    i.push(ranges::Range::new(start, end, interpolation));
                }
                filter_dict.insert(street.into(), ranges::Ranges::new(i));
            }
        }

        Ok(filter_dict)
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
            let street_ranges = self.get_street_ranges()?;
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
        let memory_cache = util::build_street_reference_cache(reference)
            .context("build_street_reference_cache() failed")?;

        let mut lst = self.config.build_ref_streets(&memory_cache);

        lst.sort();
        lst.dedup();
        let write = self
            .file
            .get_ref_streets_write_stream(&self.ctx)
            .context("get_ref_streets_write_stream() failed")?;
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
        let street_ranges = self.get_street_ranges()?;
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
        let street_ranges = self
            .get_street_ranges()
            .context("get_street_ranges() failed")?;
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
        let all_ref_house_numbers = self
            .get_ref_housenumbers()
            .context("get_ref_housenumbers() failed")?;
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
                self.config.should_show_ref_street(osm_street_name),
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
        let (ongoing_streets, done_streets) = self
            .get_missing_housenumbers()
            .context("get_missing_housenumbers() failed")?;

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
                self.config.should_show_ref_street(osm_street_name),
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
        let yamls_cache = format!("{}/{}", ctx.get_abspath("data")?, "yamls.cache");
        let stream = ctx
            .get_file_system()
            .open_read(&yamls_cache)
            .context(format!("failed to open {} for reading", yamls_cache))?;
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
    pub fn set_relation(&mut self, name: &str, relation: &Relation) {
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
    pub fn get_active_names(&mut self) -> anyhow::Result<Vec<String>> {
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
    pub fn activate_all(&mut self, activate_all: bool) {
        self.activate_all = activate_all;
    }

    /// If refcounty is not None, forget about all relations outside that refcounty.
    pub fn limit_to_refcounty(&mut self, refcounty: &Option<String>) -> anyhow::Result<()> {
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
    pub fn limit_to_refsettlement(&mut self, refsettlement: &Option<String>) -> anyhow::Result<()> {
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

    fn get_relation(&mut self, name: &str) -> PyResult<PyRelation> {
        match self.relations.get_relation(name) {
            Ok(value) => Ok(PyRelation { relation: value }),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_relation() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_names(&self) -> Vec<String> {
        self.relations.get_names()
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let normalizers = relation.get_street_ranges().unwrap();
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
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/streets-gazdagret.csv", &streets_value)],
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
        let housenumbers_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[(
                "workdir/street-housenumbers-gazdagret.csv",
                &housenumbers_value,
            )],
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

    /// Tests Relation::get_street_ranges().
    #[test]
    fn test_relation_get_street_ranges() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let filters = relation.get_street_ranges().unwrap();
        let mut expected_filters: HashMap<String, ranges::Ranges> = HashMap::new();
        expected_filters.insert(
            "Budaörsi út".into(),
            ranges::Ranges::new(vec![ranges::Range::new(137, 165, "")]),
        );
        expected_filters.insert(
            "Csiki-hegyek utca".into(),
            ranges::Ranges::new(vec![
                ranges::Range::new(1, 15, ""),
                ranges::Range::new(2, 26, ""),
            ]),
        );
        expected_filters.insert(
            "Hamzsabégi út".into(),
            ranges::Ranges::new(vec![ranges::Range::new(1, 12, "all")]),
        );
        assert_eq!(filters, expected_filters);
        let mut expected_streets: HashMap<String, String> = HashMap::new();
        expected_streets.insert("OSM Name 1".into(), "Ref Name 1".into());
        expected_streets.insert("OSM Name 2".into(), "Ref Name 2".into());
        expected_streets.insert("Misspelled OSM Name 1".into(), "OSM Name 1".into());
        assert_eq!(relation.get_config().get_refstreets(), expected_streets);
        let street_blacklist = relation.get_config().get_street_filters();
        assert_eq!(street_blacklist, ["Only In Ref Nonsense utca".to_string()]);
    }

    /// Tests Relation::get_street_ranges(): when the filter file is empty.
    #[test]
    fn test_relation_get_street_ranges_empty() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("empty").unwrap();
        let filters = relation.get_street_ranges().unwrap();
        assert_eq!(filters.is_empty(), true);
    }

    /// Tests Relation::get_ref_street_from_osm_street().
    #[test]
    fn test_relation_get_ref_street_from_osm_street() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let street = "Budaörsi út";
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let refcounty = relation.get_config().get_refcounty();
        let street = relation.get_config().get_ref_street_from_osm_street(street);
        assert_eq!(refcounty, "01");
        assert_eq!(
            relation.get_config().get_street_refsettlement(&street),
            ["011"]
        );
        assert_eq!(street, "Budaörsi út");
    }

    /// Tests Relation::get_ref_street_from_osm_street(): street-specific refsettlement override.
    #[test]
    fn test_relation_get_ref_street_from_osm_street_refsettlement_override() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let street = "Teszt utca";
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let refcounty = relation.get_config().get_refcounty();
        let street = relation.get_config().get_ref_street_from_osm_street(street);
        assert_eq!(refcounty, "01");
        assert_eq!(
            relation.get_config().get_street_refsettlement(&street),
            ["012"]
        );
        assert_eq!(street, "Teszt utca");
    }

    /// Tests Relation.get_ref_street_from_osm_street(): OSM -> ref name mapping.
    #[test]
    fn test_relation_get_ref_street_from_osm_street_refstreets() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let street = "OSM Name 1";
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let refcounty = relation.get_config().get_refcounty();
        let street = relation
            .get_config()
            .get_ref_street_from_osm_street(&street);
        assert_eq!(refcounty, "01");
        assert_eq!(
            relation.get_config().get_street_refsettlement(&street),
            ["011"]
        );
        assert_eq!(street, "Ref Name 1");
    }

    /// Tests Relation.get_ref_street_from_osm_street(): relation without a filter file.
    #[test]
    fn test_relation_get_ref_street_from_osm_street_nosuchrelation() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let street = "OSM Name 1";
        let relation_name = "nosuchrelation";
        let relation = relations.get_relation(relation_name).unwrap();
        let refcounty = relation.get_config().get_refcounty();
        let street = relation.get_config().get_ref_street_from_osm_street(street);
        assert_eq!(refcounty, "01");
        assert_eq!(
            relation.get_config().get_street_refsettlement(&street),
            ["011"]
        );
        assert_eq!(street, "OSM Name 1");
    }

    /// Tests Relation.get_ref_street_from_osm_street(): a relation with an empty filter file.
    #[test]
    fn test_relation_get_ref_street_from_osm_street_emptyrelation() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let street = "OSM Name 1";
        let relation_name = "empty";
        let relation = relations.get_relation(relation_name).unwrap();
        let refcounty = relation.get_config().get_refcounty();
        let street = relation.get_config().get_ref_street_from_osm_street(street);
        assert_eq!(refcounty, "01");
        assert_eq!(
            relation.get_config().get_street_refsettlement(&street),
            ["011"]
        );
        assert_eq!(street, "OSM Name 1");
    }

    /// Tests Relation.get_ref_street_from_osm_street(): the refsettlement range-level override.
    #[test]
    fn test_relation_get_ref_street_from_osm_street_range_level_override() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let street = "Csiki-hegyek utca";
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let refcounty = relation.get_config().get_refcounty();
        let street = relation.get_config().get_ref_street_from_osm_street(street);
        assert_eq!(refcounty, "01");
        assert_eq!(
            relation.get_config().get_street_refsettlement(&street),
            ["011", "013"]
        );
        assert_eq!(street, "Csiki-hegyek utca");
    }

    /// Tests make_turbo_query_for_streets().
    #[test]
    fn test_make_turbo_query_for_streets() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let from = ["A2".to_string()];
        let ret = make_turbo_query_for_streets(&relation, &from);
        let expected = r#"[out:json][timeout:425];
rel(2713748)->.searchRelation;
area(3602713748)->.searchArea;
(rel(2713748);
way["name"="A2"](r.searchRelation);
way["name"="A2"](area.searchArea);
);
out body;
>;
out skel qt;
{{style:
relation{width:3}
way{color:blue; width:4;}
}}"#;
        assert_eq!(ret, expected);
    }

    /// Tests Relation::get_ref_streets().
    #[test]
    fn test_relation_get_ref_streets() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let streets = relation.get_ref_streets().unwrap();
        assert_eq!(
            streets,
            [
                "Hamzsabégi út",
                "Only In Ref Nonsense utca",
                "Only In Ref utca",
                "Ref Name 1",
                "Törökugrató utca",
                "Tűzkő utca"
            ]
        );
    }

    /// Tests Relation::get_osm_housenumbers().
    #[test]
    fn test_relation_get_osm_housenumbers() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let street_name = "Törökugrató utca";
        let mut relation = relations.get_relation(relation_name).unwrap();
        let house_numbers = relation.get_osm_housenumbers(street_name).unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, ["1", "2"]);
    }

    /// Tests Relation::get_osm_housenumbers(): the case when addr:place is used instead of addr:street.
    #[test]
    fn test_relation_get_osm_housenumbers_addr_place() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh964";
        let mut relation = relations.get_relation(relation_name).unwrap();
        let street_name = "Tolvajos tanya";
        let house_numbers = relation.get_osm_housenumbers(street_name).unwrap();
        let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
        assert_eq!(actual, ["52"]);
    }

    /// Tests Relation::get_missing_housenumbers().
    #[test]
    fn test_relation_get_missing_housenumbers() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let mut relation = relations.get_relation(relation_name).unwrap();
        let (ongoing_streets, done_streets) = relation.get_missing_housenumbers().unwrap();
        let ongoing_streets_strs: Vec<_> = ongoing_streets
            .iter()
            .map(|(name, numbers)| {
                let numbers: Vec<_> = numbers.iter().map(|i| i.get_number()).collect();
                (name.get_osm_name().clone(), numbers)
            })
            .collect();
        // Notice how 11 and 12 is filtered out by the 'invalid' mechanism for 'Törökugrató utca'.
        assert_eq!(
            ongoing_streets_strs,
            [
                ("Törökugrató utca".to_string(), vec!["7", "10"]),
                ("Tűzkő utca".to_string(), vec!["1", "2"]),
                ("Hamzsabégi út".to_string(), vec!["1"])
            ]
        );
        let expected = [
            ("OSM Name 1".to_string(), vec!["1", "2"]),
            ("Törökugrató utca".to_string(), vec!["1", "2"]),
            ("Tűzkő utca".to_string(), vec!["9", "10"]),
        ];
        let done_streets_strs: Vec<_> = done_streets
            .iter()
            .map(|(name, numbers)| {
                let numbers: Vec<_> = numbers.iter().map(|i| i.get_number()).collect();
                (name.get_osm_name().clone(), numbers)
            })
            .collect();
        assert_eq!(done_streets_strs, expected);
    }

    /// Sets the housenumber_letters property from code.
    fn set_config_housenumber_letters(config: &mut RelationConfig, housenumber_letters: bool) {
        config.set_property(
            "housenumber-letters",
            &serde_json::json!(housenumber_letters),
        )
    }

    /// Sets the 'filters' key from code.
    fn set_config_filters(config: &mut RelationConfig, filters: &serde_json::Value) {
        config.set_property("filters", filters)
    }

    /// Tests Relation::get_missing_housenumbers(): 7/A is detected when 7/B is already mapped.
    #[test]
    fn test_relation_get_missing_housenumbers_letter_suffix() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh267";
        let mut relation = relations.get_relation(relation_name).unwrap();
        // Opt-in, this is not the default behavior.
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let (ongoing_streets, _done_streets) = relation.get_missing_housenumbers().unwrap();
        let ongoing_street = ongoing_streets[0].clone();
        let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
        let mut housenumber_range_names: Vec<_> =
            housenumber_ranges.iter().map(|i| i.get_number()).collect();
        housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
        // Make sure that 1/1 shows up in the output: it's not the same as '1' or '11'.
        let expected = [
            "1", "1/1", "1/2", "3", "5", "7", "7/A", "7/B", "7/C", "9", "11", "13", "13-15",
        ];
        assert_eq!(housenumber_range_names, expected);
    }

    /// Tests Relation::get_missing_housenumbers(): how 'invalid' interacts with normalization.
    #[test]
    fn test_relation_get_missing_housenumbers_letter_suffix_invalid() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh296";
        let mut relation = relations.get_relation(relation_name).unwrap();
        // Opt-in, this is not the default behavior.
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        // Set custom 'invalid' map.
        let filters = serde_json::json!({
            "Rétköz utca": {
                "invalid": ["9", "47"]
            }
        });
        set_config_filters(&mut config, &filters);
        relation.set_config(&config);
        let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
        let ongoing_street = ongoing_streets[0].clone();
        let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
        let mut housenumber_range_names: Vec<_> =
            housenumber_ranges.iter().map(|i| i.get_number()).collect();
        housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
        // Notice how '9 A 1' is missing here: it's not a simple house number, so it gets normalized
        // to just '9' and the above filter silences it.
        let expected = ["9/A"];
        assert_eq!(housenumber_range_names, expected);
    }

    /// Tests Relation::get_missing_housenumbers(): how 'invalid' interacts with housenumber-letters: true or false.
    #[test]
    fn test_relation_get_missing_housenumbers_invalid_simplify() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh385";
        let mut relation = relations.get_relation(relation_name).unwrap();

        // Default case: housenumber-letters=false.
        {
            let filters = serde_json::json!({
                "Kővirág sor": {
                    "invalid": ["37b"]
                }
            });
            let mut config = relation.get_config().clone();
            set_config_filters(&mut config, &filters);
            relation.set_config(&config);
            let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
            // Note how 37b from invalid is simplified to 37; and how 37/B from ref is simplified to
            // 37 as well, so we find the match.
            assert_eq!(ongoing_streets.is_empty(), true);
        }

        // Opt-in case: housenumber-letters=true.
        {
            let mut config = relation.get_config().clone();
            set_config_housenumber_letters(&mut config, true);
            relation.set_config(&config);
            let filters = serde_json::json!({
                "Kővirág sor": {
                    "invalid": ["37b"]
                }
            });
            set_config_filters(&mut config, &filters);
            relation.set_config(&config);
            let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
            // In this case 37b from invalid matches 37/B from ref.
            assert_eq!(ongoing_streets.is_empty(), true);
        }

        // Make sure out-of-range invalid elements are just ignored and no exception is raised.
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let filters = serde_json::json!({
            "Kővirág sor": {
                "invalid": ["5"],
                "ranges": [{"start": "1", "end": "3"}],
            }
        });
        set_config_filters(&mut config, &filters);
        relation.set_config(&config);
        relation.get_missing_housenumbers().unwrap();
    }

    /// Tests Relation::get_missing_housenumbers(): '42 A' vs '42/A' is recognized as a match.
    #[test]
    fn test_relation_get_missing_housenumbers_letter_suffix_normalize() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh286";
        let mut relation = relations.get_relation(relation_name).unwrap();
        // Opt-in, this is not the default behavior.
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
        let ongoing_street = ongoing_streets[0].clone();
        let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
        let mut housenumber_range_names: Vec<_> =
            housenumber_ranges.iter().map(|i| i.get_number()).collect();
        housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
        // Note how 10/B is not in this list.
        let expected = ["10/A"];
        assert_eq!(housenumber_range_names, expected);
    }

    /// Tests Relation::get_missing_housenumbers(): '42/A*' and '42/a' matches.
    #[test]
    fn test_relation_get_missing_housenumbers_letter_suffix_source_suffix() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh299";
        let mut relation = relations.get_relation(relation_name).unwrap();
        // Opt-in, this is not the default behavior.
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
        // Note how '52/B*' is not in this list.
        assert_eq!(ongoing_streets, []);
    }

    /// Tests Relation::get_missing_housenumbers(): 'a' is not stripped from '1;3a'.
    #[test]
    fn test_relation_get_missing_housenumbers_letter_suffix_normalize_semicolon() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh303";
        let mut relation = relations.get_relation(relation_name).unwrap();
        // Opt-in, this is not the default behavior.
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
        let ongoing_street = ongoing_streets[0].clone();
        let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
        let mut housenumber_range_names: Vec<_> =
            housenumber_ranges.iter().map(|i| i.get_number()).collect();
        housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
        // Note how 43/B and 43/C is not here.
        let expected = ["43/A", "43/D"];
        assert_eq!(housenumber_range_names, expected);
    }

    /// Tests Relation::get_missing_streets().
    #[test]
    fn test_relation_get_missing_streets() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let (only_in_reference, in_both) = relation.get_missing_streets().unwrap();

        // Note that 'Only In Ref Nonsense utca' is missing from this list.
        assert_eq!(only_in_reference, ["Only In Ref utca"]);

        assert_eq!(
            in_both,
            [
                "Hamzsabégi út",
                "Ref Name 1",
                "Törökugrató utca",
                "Tűzkő utca"
            ]
        );
    }

    /// Tests Relation::get_additional_streets().
    #[test]
    fn test_relation_get_additional_streets() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let only_in_osm = relation
            .get_additional_streets(/*sorted_result=*/ true)
            .unwrap();

        assert_eq!(only_in_osm, [util::Street::from_string("Only In OSM utca")]);

        // These is filtered out, even if it's OSM-only.
        let osm_street_blacklist = relation.get_config().get_osm_street_filters();
        assert_eq!(osm_street_blacklist, ["Second Only In OSM utca"]);
    }

    /// Tests Relation::get_additional_streets(): when the osm-street-filters key is missing.
    #[test]
    fn test_relation_get_additional_streets_no_osm_street_filters() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh385";
        let relation = relations.get_relation(relation_name).unwrap();
        assert_eq!(
            relation.get_config().get_osm_street_filters().is_empty(),
            true
        );
    }

    /// Relation::get_additional_housenumbers().
    #[test]
    fn test_relation_get_additional_housenumbers() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let mut relation = relations.get_relation(relation_name).unwrap();
        let only_in_osm = relation.get_additional_housenumbers().unwrap();
        let only_in_osm_strs: Vec<_> = only_in_osm
            .iter()
            .map(|(name, numbers)| {
                let numbers: Vec<_> = numbers.iter().map(|i| i.get_number()).collect();
                (name.get_osm_name(), numbers)
            })
            .collect();
        // Note how Second Only In OSM utca 1 is filtered out explicitly.
        assert_eq!(
            only_in_osm_strs,
            [(&"Only In OSM utca".to_string(), vec!["1"])]
        );
    }

    /// Wrapper around get_config.get_filters() that doesn't return an Optional.
    fn get_filters(relation: &Relation) -> serde_json::Map<String, serde_json::Value> {
        let mut filters = serde_json::json!({});
        if let Some(value) = relation.config.get_filters() {
            filters = value.clone();
        }
        filters.as_object().unwrap().clone()
    }

    /// Unwraps an escaped matrix of rust.PyDocs into a string matrix.
    fn table_doc_to_string(table: &[Vec<yattag::Doc>]) -> Vec<Vec<String>> {
        let mut table_content = Vec::new();
        for row in table {
            let mut row_content = Vec::new();
            for cell in row {
                row_content.push(cell.get_value());
            }
            table_content.push(row_content);
        }
        table_content
    }

    /// Tests Relation::write_missing_housenumbers().
    #[test]
    fn test_relation_write_missing_housenumbers() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let percent_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/gazdagret.percent", &percent_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let mut relation = relations.get_relation(relation_name).unwrap();
        let expected = String::from_utf8(
            util::get_content(&ctx.get_abspath("workdir/gazdagret.percent").unwrap()).unwrap(),
        )
        .unwrap();

        let ret = relation.write_missing_housenumbers().unwrap();

        let (todo_street_count, todo_count, done_count, percent, table) = ret;
        assert_eq!(todo_street_count, 3);
        assert_eq!(todo_count, 5);
        assert_eq!(done_count, 6);
        assert_eq!(percent, "54.55");
        let string_table = table_doc_to_string(&table);
        assert_eq!(
            string_table,
            [
                ["Street name", "Missing count", "House numbers"],
                ["Törökugrató utca", "2", "7<br />10"],
                ["Tűzkő utca", "2", "1<br />2"],
                ["Hamzsabégi út", "1", "1"]
            ]
        );
        let mut guard = percent_value.lock().unwrap();
        guard.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        guard.read_to_end(&mut actual).unwrap();
        assert_eq!(String::from_utf8(actual).unwrap(), expected);
    }

    /// Tests Relation::write_missing_housenumbers(): the case when percent can't be determined.
    #[test]
    fn test_relation_write_missing_housenumbers_empty() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let percent_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/empty.percent", &percent_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "empty";
        let mut relation = relations.get_relation(relation_name).unwrap();

        let ret = relation.write_missing_housenumbers().unwrap();

        let (_todo_street_count, _todo_count, _done_count, percent, _table) = ret;
        assert_eq!(percent, "100.00");
        assert_eq!(get_filters(&relation).is_empty(), true);
    }

    /// Tests Relation::write_missing_housenumbers(): the case when the street is interpolation=all and coloring is wanted.
    #[test]
    fn test_relation_write_missing_housenumbers_interpolation_all() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let percent_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/budafok.percent", &percent_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "budafok";
        let mut relation = relations.get_relation(relation_name).unwrap();

        let ret = relation.write_missing_housenumbers().unwrap();

        let (_todo_street_count, _todo_count, _done_count, _percent, table) = ret;
        let string_table = table_doc_to_string(&table);
        assert_eq!(
            string_table,
            [
                ["Street name", "Missing count", "House numbers"],
                [
                    "Vöröskúti határsor",
                    "4",
                    "2, 12, 34, <span style=\"color: blue;\">36</span>"
                ]
            ]
        );
        let mut guard = percent_value.lock().unwrap();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }

    /// Tests Relation::write_missing_housenumbers(): sorting is performed after range reduction.
    #[test]
    fn test_relation_write_missing_housenumbers_sorting() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let percent_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/gh414.percent", &percent_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gh414";
        let mut relation = relations.get_relation(relation_name).unwrap();

        let ret = relation.write_missing_housenumbers().unwrap();

        let (_todo_street_count, _todo_count, _done_count, _percent, table) = ret;
        let string_table = table_doc_to_string(&table);
        // Note how 'A utca' is logically 5 house numbers, but it's a single range, so it's
        // ordered after 'B utca'.
        assert_eq!(
            string_table,
            [
                ["Street name", "Missing count", "House numbers"],
                ["B utca", "2", "1, 3"],
                ["A utca", "1", "2-10"]
            ]
        );
        let mut guard = percent_value.lock().unwrap();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }

    /// Tests Relation::write_missing_streets().
    #[test]
    fn test_write_missing_streets() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let percent_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/gazdagret-streets.percent", &percent_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let expected = String::from_utf8(
            util::get_content(
                &ctx.get_abspath("workdir/gazdagret-streets.percent")
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();

        let ret = relation.write_missing_streets().unwrap();

        let (todo_count, done_count, percent, streets) = ret;

        assert_eq!(todo_count, 1);
        assert_eq!(done_count, 4);
        assert_eq!(percent, "80.00");
        assert_eq!(streets, ["Only In Ref utca"]);
        let mut guard = percent_value.lock().unwrap();
        guard.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        guard.read_to_end(&mut actual).unwrap();
        assert_eq!(String::from_utf8(actual).unwrap(), expected);
    }

    /// Tests Relation::write_missing_streets(): the case when percent can't be determined.
    #[test]
    fn test_write_missing_streets_empty() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let percent_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/empty-streets.percent", &percent_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "empty";
        let relation = relations.get_relation(relation_name).unwrap();

        let ret = relation.write_missing_streets().unwrap();

        let mut guard = percent_value.lock().unwrap();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let (_todo_count, _done_count, percent, _streets) = ret;
        assert_eq!(percent, "100.00");
    }

    /// Tests Relation::build_ref_housenumbers().
    #[test]
    fn test_relation_build_ref_housenumbers() {
        let ctx = context::tests::make_test_context().unwrap();
        let refdir = ctx.get_abspath("refdir").unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
        let memory_cache = util::build_reference_cache(&refpath, "01").unwrap();
        let relation_name = "gazdagret";
        let street = "Törökugrató utca";
        let relation = relations.get_relation(relation_name).unwrap();
        let ret = relation.build_ref_housenumbers(&memory_cache, street, "");
        let expected = [
            "Törökugrató utca\t1\tcomment",
            "Törökugrató utca\t10\t",
            "Törökugrató utca\t11\t",
            "Törökugrató utca\t12\t",
            "Törökugrató utca\t2\t",
            "Törökugrató utca\t7\t",
        ];
        assert_eq!(ret, expected);
    }

    /// Tests Relation::build_ref_housenumbers(): the case when the street is not in the reference.
    #[test]
    fn test_relation_build_ref_housenumbers_missing() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let refdir = ctx.get_abspath("refdir").unwrap();
        let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
        let memory_cache = util::build_reference_cache(&refpath, "01").unwrap();
        let relation_name = "gazdagret";
        let street = "No such utca";
        let relation = relations.get_relation(relation_name).unwrap();

        let ret = relation.build_ref_housenumbers(&memory_cache, street, "");

        assert_eq!(ret.is_empty(), true);
    }

    /// Tests Relation::build_ref_streets().
    #[test]
    fn test_relation_build_ref_streets() {
        let ctx = context::tests::make_test_context().unwrap();
        let refdir = ctx.get_abspath("refdir").unwrap();
        let refpath = format!("{}/utcak_20190514.tsv", refdir);
        let memory_cache = util::build_street_reference_cache(&refpath).unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();

        let ret = relation.config.build_ref_streets(&memory_cache);

        assert_eq!(
            ret,
            [
                "Törökugrató utca",
                "Tűzkő utca",
                "Ref Name 1",
                "Only In Ref utca",
                "Only In Ref Nonsense utca",
                "Hamzsabégi út"
            ]
        );
    }

    /// Tests Relation::write_ref_housenumbers().
    #[test]
    fn test_relation_writer_ref_housenumbers() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let refdir = ctx.get_abspath("refdir").unwrap();
        let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
        let refpath2 = format!("{}/hazszamok_kieg_20190808.tsv", refdir);
        let mut file_system = context::tests::TestFileSystem::new();
        let ref_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[(
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_value,
            )],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let expected = String::from_utf8(
            util::get_content(
                &ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst")
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let relation = relations.get_relation(relation_name).unwrap();

        relation
            .write_ref_housenumbers(&[refpath, refpath2])
            .unwrap();

        let mut guard = ref_value.lock().unwrap();
        guard.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        guard.read_to_end(&mut actual).unwrap();
        assert_eq!(String::from_utf8(actual).unwrap(), expected);
    }

    /// Tests Relation::write_ref_housenumbers(): the case when the refcounty code is missing in the reference.
    #[test]
    fn test_relation_writer_ref_housenumbers_nosuchrefcounty() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let refdir = ctx.get_abspath("refdir").unwrap();
        let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
        let mut file_system = context::tests::TestFileSystem::new();
        let ref_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[(
                "workdir/street-housenumbers-reference-nosuchrefcounty.lst",
                &ref_value,
            )],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "nosuchrefcounty";
        let relation = relations.get_relation(relation_name).unwrap();

        relation.write_ref_housenumbers(&[refpath]).unwrap();
    }

    /// Tests Relation::write_ref_housenumbers(): the case when the refsettlement code is missing in the reference.
    #[test]
    fn test_relation_writer_ref_housenumbers_nosuchrefsettlement() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let refdir = ctx.get_abspath("refdir").unwrap();
        let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
        let mut file_system = context::tests::TestFileSystem::new();
        let ref_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[(
                "workdir/street-housenumbers-reference-nosuchrefsettlement.lst",
                &ref_value,
            )],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "nosuchrefsettlement";
        let relation = relations.get_relation(relation_name).unwrap();

        relation.write_ref_housenumbers(&[refpath]).unwrap();
    }

    /// Tests Relation::write_ref_streets().
    #[test]
    fn test_relation_write_ref_streets() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut file_system = context::tests::TestFileSystem::new();
        let ref_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &ctx,
            &[("workdir/streets-reference-gazdagret.lst", &ref_value)],
        );
        file_system.set_files(&files);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);
        let refdir = ctx.get_abspath("refdir").unwrap();
        let refpath = format!("{}/utcak_20190514.tsv", refdir);
        let mut relations = Relations::new(&ctx).unwrap();
        let relation_name = "gazdagret";
        let relation = relations.get_relation(relation_name).unwrap();
        let expected = String::from_utf8(
            util::get_content(
                &ctx.get_abspath("workdir/streets-reference-gazdagret.lst")
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();

        relation.write_ref_streets(&refpath).unwrap();

        let mut guard = ref_value.lock().unwrap();
        guard.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        guard.read_to_end(&mut actual).unwrap();
        assert_eq!(String::from_utf8(actual).unwrap(), expected);
    }

    /// Tests the Relations struct.
    #[test]
    fn test_relations() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let expected_relation_names = [
            "budafok",
            "empty",
            "gazdagret",
            "gellerthegy",
            "inactiverelation",
            "nosuchrefcounty",
            "nosuchrefsettlement",
            "nosuchrelation",
            "test",
            "ujbuda",
        ];
        assert_eq!(relations.get_names(), expected_relation_names);
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"inactiverelation".to_string()),
            false
        );
        let mut osmids: Vec<_> = relations
            .get_relations()
            .unwrap()
            .iter()
            .map(|relation| relation.get_config().get_osmrelation())
            .collect();
        osmids.sort();
        assert_eq!(
            osmids,
            [13, 42, 42, 43, 44, 45, 66, 221998, 2702687, 2713748]
        );
        assert_eq!(
            relations
                .get_relation("ujbuda")
                .unwrap()
                .get_config()
                .should_check_missing_streets(),
            "only"
        );

        relations.activate_all(true);
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"inactiverelation".to_string()),
            true
        );

        // Allow seeing data of a relation even if it's not in relations.yaml.
        relations.get_relation("gh195").unwrap();

        // Test limit_to_refcounty().
        // 01
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"gazdagret".to_string()),
            true
        );
        // 43
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"budafok".to_string()),
            true
        );
        relations
            .limit_to_refcounty(&Some("01".to_string()))
            .unwrap();
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"gazdagret".to_string()),
            true
        );
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"budafok".to_string()),
            false
        );

        // Test limit_to_refsettlement().
        // 011
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"gazdagret".to_string()),
            true
        );
        // 99
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"nosuchrefsettlement".to_string()),
            true
        );
        relations
            .limit_to_refsettlement(&Some("99".to_string()))
            .unwrap();
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"gazdagret".to_string()),
            false
        );
        assert_eq!(
            relations
                .get_active_names()
                .unwrap()
                .contains(&"nosuchrefsettlement".to_string()),
            true
        );
    }

    /// Tests RelationConfig::should_check_missing_streets().
    #[test]
    fn test_relation_config_should_check_missing_streets() {
        let relation_name = "ujbuda";
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation(relation_name).unwrap();
        let ret = relation.get_config().should_check_missing_streets();
        assert_eq!(ret, "only");
    }

    /// Tests RelationConfig::should_check_missing_streets(): the default value.
    #[test]
    fn test_relation_config_should_check_missing_streets_empty() {
        let relation_name = "empty";
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation(relation_name).unwrap();
        assert_eq!(relation.get_name(), "empty");
        let ret = relation.get_config().should_check_missing_streets();
        assert_eq!(ret, "yes");
    }

    /// Tests RelationConfig::should_check_missing_streets(): a relation without a filter file.
    #[test]
    fn test_relation_config_should_check_missing_streets_nosuchrelation() {
        let relation_name = "nosuchrelation";
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation(relation_name).unwrap();
        let ret = relation.get_config().should_check_missing_streets();
        assert_eq!(ret, "yes");
    }

    /// Tests RelationConfig::get_letter_suffix_style().
    #[test]
    fn test_relation_config_get_letter_suffix_style() {
        let relation_name = "empty";
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let mut relation = relations.get_relation(relation_name).unwrap();
        assert_eq!(
            relation.config.get_letter_suffix_style(),
            util::LetterSuffixStyle::Upper as i32
        );
        let mut config = relation.config.clone();
        config.set_letter_suffix_style(util::LetterSuffixStyle::Lower as i32);
        relation.set_config(&config);
        assert_eq!(
            relation.config.get_letter_suffix_style(),
            util::LetterSuffixStyle::Lower as i32
        );
    }

    /// Tests refcounty_get_name().
    #[test]
    fn test_refcounty_get_name() {
        let ctx = context::tests::make_test_context().unwrap();
        let relations = Relations::new(&ctx).unwrap();
        assert_eq!(relations.refcounty_get_name("01"), "Budapest");
        assert_eq!(relations.refcounty_get_name("99"), "");
    }

    /// Tests refcounty_get_refsettlement_ids().
    #[test]
    fn test_refcounty_get_refsettlement_ids() {
        let ctx = context::tests::make_test_context().unwrap();
        let relations = Relations::new(&ctx).unwrap();
        assert_eq!(
            relations.refcounty_get_refsettlement_ids("01"),
            ["011".to_string(), "012".to_string()]
        );
        assert_eq!(
            relations.refcounty_get_refsettlement_ids("99").is_empty(),
            true
        );
    }

    /// Tests refsettlement_get_name().
    #[test]
    fn test_refsettlement_get_name() {
        let ctx = context::tests::make_test_context().unwrap();
        let relations = Relations::new(&ctx).unwrap();
        assert_eq!(relations.refsettlement_get_name("01", "011"), "Újbuda");
        assert_eq!(relations.refsettlement_get_name("99", ""), "");
        assert_eq!(relations.refsettlement_get_name("01", "99"), "");
    }

    /// Tests Relalations::get_aliases().
    #[test]
    fn test_relations_get_aliases() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        // Expect an alias -> canonicalname map.
        let mut expected = HashMap::new();
        expected.insert("budapest_22".to_string(), "budafok".to_string());
        assert_eq!(relations.get_aliases().unwrap(), expected);
    }

    /// Tests RelationConfig::get_street_is_even_odd().
    #[test]
    fn test_relation_config_get_street_is_even_odd() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        assert_eq!(
            relation.config.get_street_is_even_odd("Hamzsabégi út"),
            false
        );

        assert_eq!(relation.config.get_street_is_even_odd("Teszt utca"), true);
    }

    /// Tests RelationConfig::should_show_ref_street().
    #[test]
    fn test_relation_config_should_show_ref_street() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        assert_eq!(
            relation.config.should_show_ref_street("Törökugrató utca"),
            false
        );
        assert_eq!(
            relation.config.should_show_ref_street("Hamzsabégi út"),
            true
        );
    }

    /// Tests RelationConfig::is_active().
    #[test]
    fn test_relation_config_is_active() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        assert_eq!(relation.get_config().is_active(), true);
    }
}
