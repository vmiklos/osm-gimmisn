/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The areas module contains the Relations class and associated functionality.

use crate::area_files;
use crate::cache;
use crate::context;
use crate::i18n::translate as tr;
use crate::ranges;
use crate::util;
use crate::yattag;
use anyhow::Context;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::BufRead;
use std::io::Read;
use std::ops::DerefMut;
use std::rc::Rc;

/// The filters -> <street> -> ranges key from data/relation-<name>.yaml.
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationRangesDict {
    pub end: String,
    refsettlement: Option<String>,
    pub start: String,
}

/// The filters key from data/relation-<name>.yaml.
#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct RelationFiltersDict {
    pub interpolation: Option<String>,
    pub invalid: Option<Vec<String>>,
    pub ranges: Option<Vec<RelationRangesDict>>,
    pub valid: Option<Vec<String>>,
    refsettlement: Option<String>,
    show_refstreet: Option<bool>,
}

/// A relation from data/relation-<name>.yaml.
#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct RelationDict {
    additional_housenumbers: Option<bool>,
    pub alias: Option<Vec<String>>,
    pub filters: Option<HashMap<String, RelationFiltersDict>>,
    housenumber_letters: Option<bool>,
    inactive: Option<bool>,
    missing_streets: Option<String>,
    osm_street_filters: Option<Vec<String>>,
    pub osmrelation: Option<u64>,
    pub refcounty: Option<String>,
    pub refsettlement: Option<String>,
    pub refstreets: Option<HashMap<String, String>>,
    pub street_filters: Option<Vec<String>>,
    pub source: Option<String>,
}

impl Default for RelationDict {
    fn default() -> Self {
        let additional_housenumbers = None;
        let alias = None;
        let filters = None;
        let housenumber_letters = None;
        let inactive = None;
        let missing_streets = None;
        let osm_street_filters = None;
        let osmrelation = None;
        let refcounty = None;
        let refsettlement = None;
        let refstreets = None;
        let street_filters = None;
        let source = None;
        RelationDict {
            additional_housenumbers,
            alias,
            filters,
            housenumber_letters,
            inactive,
            missing_streets,
            osm_street_filters,
            osmrelation,
            refcounty,
            refsettlement,
            refstreets,
            street_filters,
            source,
        }
    }
}

/// A relation configuration comes directly from static data, not a result of some external query.
#[derive(Clone)]
pub struct RelationConfig {
    parent: RelationDict,
    dict: RelationDict,
}

impl RelationConfig {
    pub fn new(parent_config: &RelationDict, my_config: &RelationDict) -> Self {
        RelationConfig {
            parent: parent_config.clone(),
            dict: my_config.clone(),
        }
    }

    /// Gets the typed value of a property transparently.
    fn get_property<T: Clone>(parent_value: &Option<T>, my_value: &Option<T>) -> Option<T> {
        if let Some(value) = my_value {
            return Some(value.clone());
        }

        if let Some(value) = parent_value {
            return Some(value.clone());
        }

        None
    }

    /// Sets if the relation is active.
    pub fn set_active(&mut self, active: bool) {
        self.dict.inactive = Some(!active);
    }

    /// Gets if the relation is active.
    pub fn is_active(&self) -> bool {
        match RelationConfig::get_property(&self.parent.inactive, &self.dict.inactive) {
            Some(value) => !value,
            None => true,
        }
    }

    /// Gets the OSM relation object's ID.
    pub fn get_osmrelation(&self) -> u64 {
        self.parent.osmrelation.unwrap()
    }

    /// Gets the relation's refcounty identifier from reference.
    pub fn get_refcounty(&self) -> String {
        match RelationConfig::get_property(&self.parent.refcounty, &self.dict.refcounty) {
            Some(value) => value,
            None => "".into(),
        }
    }

    /// Gets the relation's refsettlement identifier from reference.
    pub fn get_refsettlement(&self) -> String {
        RelationConfig::get_property(&self.parent.refsettlement, &self.dict.refsettlement).unwrap()
    }

    /// Gets the alias(es) of the relation: alternative names which are also accepted.
    fn get_alias(&self) -> Vec<String> {
        match RelationConfig::get_property(&self.parent.alias, &self.dict.alias) {
            Some(value) => value,
            None => Vec::new(),
        }
    }

    /// Return value can be 'yes', 'no' and 'only'.
    pub fn should_check_missing_streets(&self) -> String {
        match RelationConfig::get_property(&self.parent.missing_streets, &self.dict.missing_streets)
        {
            Some(value) => value,
            None => "yes".into(),
        }
    }

    /// Do we care if 42/B is missing when 42/A is provided?
    fn should_check_housenumber_letters(&self) -> bool {
        RelationConfig::get_property(
            &self.parent.housenumber_letters,
            &self.dict.housenumber_letters,
        )
        .unwrap_or(false)
    }

    /// Do we care if 42 is in OSM when it's not in the ref?
    pub fn should_check_additional_housenumbers(&self) -> bool {
        RelationConfig::get_property(
            &self.parent.additional_housenumbers,
            &self.dict.additional_housenumbers,
        )
        .unwrap_or(false)
    }

    /// Returns an OSM name -> ref name map.
    pub fn get_refstreets(&self) -> HashMap<String, String> {
        match self.dict.refstreets {
            Some(ref value) => value.clone(),
            None => HashMap::new(),
        }
    }

    /// Returns a street name -> properties map.
    fn get_filters(&self) -> &Option<HashMap<String, RelationFiltersDict>> {
        // The schema doesn't allow this key in parent config, no need to go via the slow
        // get_property().
        &self.dict.filters
    }

    /// Returns a street from relation filters.
    fn get_filter_street(&self, street: &str) -> Option<&RelationFiltersDict> {
        let filters = match self.get_filters() {
            Some(value) => value,
            None => {
                return None;
            }
        };
        filters.get(street)
    }

    /// Determines in a relation's street is interpolation=all or not.
    pub fn get_street_is_even_odd(&self, street: &str) -> bool {
        let mut interpolation_all = false;
        if let Some(filter_for_street) = self.get_filter_street(street) {
            if let Some(ref interpolation) = filter_for_street.interpolation {
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
            if let Some(value) = filter_for_street.show_refstreet {
                show_ref_street = value;
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

        for (filter_street, value) in filters {
            if filter_street != street {
                continue;
            }

            if let Some(ref refsettlement) = value.refsettlement {
                ret = vec![refsettlement.to_string()];
            }
            if let Some(ref ranges) = value.ranges {
                for street_range in ranges {
                    if let Some(ref refsettlement) = street_range.refsettlement {
                        ret.push(refsettlement.to_string());
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
        match RelationConfig::get_property(&self.parent.street_filters, &self.dict.street_filters) {
            Some(value) => value,
            None => vec![],
        }
    }

    /// Gets list of streets which are only in OSM, but have to be filtered out.
    fn get_osm_street_filters(&self) -> Vec<String> {
        match RelationConfig::get_property(
            &self.parent.osm_street_filters,
            &self.dict.osm_street_filters,
        ) {
            Some(value) => value,
            None => vec![],
        }
    }

    /// Builds a list of streets from a reference cache.
    fn get_ref_streets<'a>(
        &self,
        reference: &'a HashMap<String, HashMap<String, Vec<String>>>,
    ) -> &'a Vec<String> {
        let refcounty = self.get_refcounty();
        let refsettlement = self.get_refsettlement();
        &reference[&refcounty][&refsettlement]
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

/// Return type of Relation::get_missing_housenumbers().
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MissingHousenumbers {
    pub ongoing_streets: util::NumberedStreets,
    pub done_streets: util::NumberedStreets,
}

/// One row in workdir/streets-<relation>.csv. Keep this in sync with data/streets-template.overpassql.
#[derive(serde::Deserialize)]
pub struct OsmStreet {
    /// Object ID.
    #[serde(rename = "@id")]
    pub id: u64,
    /// Street name.
    pub name: String,
    /// Object type.
    #[serde(rename = "@type")]
    pub object_type: Option<String>,
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
        parent_config: &RelationDict,
        yaml_cache: &HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<Self> {
        let mut my_config = RelationDict::default();
        let file = area_files::RelationFiles::new(&ctx.get_ini().get_workdir(), name);
        let relation_path = format!("relation-{name}.yaml");
        // Intentionally don't require this cache to be present, it's fine to omit it for simple
        // relations.
        if let Some(value) = yaml_cache.get(&relation_path) {
            my_config = serde_json::from_value(value.clone())
                .context(format!("failed to parse '{relation_path}'"))?;
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
        self.config = config.clone();
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
        for (street, filter) in filters {
            let mut interpolation = "";
            if let Some(ref value) = filter.interpolation {
                interpolation = value;
            }
            let mut i: Vec<ranges::Range> = Vec::new();
            if let Some(ref value) = filter.ranges {
                for range in value {
                    let start = range
                        .start
                        .trim()
                        .parse::<i64>()
                        .context("failed to parse() 'start'")?;
                    let end = range
                        .end
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

        if let Some(filters) = self.config.get_filters() {
            for (street, filter) in filters {
                if let Some(ref value) = filter.invalid {
                    invalid_dict.insert(street.into(), value.to_vec());
                }
            }
        }

        invalid_dict
    }

    /// Reads list of streets for an area from OSM.
    fn get_osm_streets(&self, sorted_result: bool) -> anyhow::Result<Vec<util::Street>> {
        let mut ret: Vec<util::Street> = Vec::new();
        let stream: Rc<RefCell<dyn Read>> = self.file.get_osm_streets_read_stream(&self.ctx)?;
        let mut guard = stream.borrow_mut();
        let mut read = std::io::BufReader::new(guard.deref_mut());
        let mut csv_reader = util::make_csv_reader(&mut read);
        for result in csv_reader.deserialize() {
            let row: OsmStreet = result?;
            let mut street = util::Street::new(
                &row.name, /*ref_name=*/ "", /*show_ref_street=*/ true,
                /*osm_id=*/ row.id,
            );
            if let Some(value) = row.object_type {
                street.set_osm_type(&value);
            }
            street.set_source(&tr("street"));
            ret.push(street)
        }
        let path = self.file.get_osm_housenumbers_path();
        if self.ctx.get_file_system().path_exists(&path) {
            let stream: Rc<RefCell<dyn Read>> =
                self.file.get_osm_housenumbers_read_stream(&self.ctx)?;
            let mut guard = stream.borrow_mut();
            let mut read = guard.deref_mut();
            let mut csv_reader = util::make_csv_reader(&mut read);
            ret.append(
                &mut util::get_street_from_housenumber(&mut csv_reader)
                    .context("get_street_from_housenumber() failed")?,
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
        let contents = self.ctx.get_file_system().read_to_string(&format!(
            "{}/{}",
            self.ctx.get_abspath("data"),
            "streets-template.overpassql"
        ))?;
        Ok(util::process_template(
            &contents,
            self.config.get_osmrelation(),
        ))
    }

    /// Gets streets from reference.
    fn get_ref_streets(&self) -> anyhow::Result<Vec<String>> {
        let mut streets: Vec<String> = Vec::new();
        let read: Rc<RefCell<dyn Read>> = self.file.get_ref_streets_read_stream(&self.ctx)?;
        let mut guard = read.borrow_mut();
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
            let stream: Rc<RefCell<dyn Read>> =
                self.file.get_osm_housenumbers_read_stream(&self.ctx)?;
            let mut guard = stream.borrow_mut();
            let mut read = guard.deref_mut();
            let mut csv_reader = util::make_csv_reader(&mut read);
            for result in csv_reader.deserialize() {
                let row: util::OsmHouseNumber = result?;
                let mut street = &row.street;
                let street_is_even_odd = self.config.get_street_is_even_odd(street);
                if street.is_empty() {
                    if let Some(ref value) = row.place {
                        street = value;
                    }
                }
                for house_number in row.housenumber.split(';') {
                    house_numbers
                        .entry(street.to_string())
                        .or_insert_with(Vec::new)
                        .append(&mut normalize(
                            self,
                            house_number,
                            street,
                            street_is_even_odd,
                            &street_ranges,
                        )?)
                }
            }
            for (key, mut value) in house_numbers {
                value.sort_unstable();
                value.dedup();
                self.osm_housenumbers
                    .insert(key, util::sort_numerically(&value));
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
        let memory_cache = util::build_street_reference_cache(&self.ctx, reference)
            .context("build_street_reference_cache() failed")?;

        let mut lst = self.config.get_ref_streets(&memory_cache).clone();

        lst.sort();
        lst.dedup();
        let write = self
            .file
            .get_ref_streets_write_stream(&self.ctx)
            .context("get_ref_streets_write_stream() failed")?;
        let mut guard = write.borrow_mut();
        for line in lst {
            guard.write_all((line + "\n").as_bytes())?;
        }
        Ok(())
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
        let mut conn = self.ctx.get_database().create()?;
        util::build_reference_index(&self.ctx, &mut conn, references)?;

        let streets: Vec<String> = self
            .get_osm_streets(/*sorted_results=*/ true)?
            .iter()
            .map(|i| i.get_osm_name().into())
            .collect();

        let mut lst: Vec<String> = Vec::new();
        let mut stmt = conn.prepare(
            "select housenumber, comment from ref_housenumbers where county_code = ?1 and settlement_code = ?2 and street = ?3")?;
        for street in streets {
            let street = self.config.get_ref_street_from_osm_street(&street);
            for refsettlement in self.config.get_street_refsettlement(&street) {
                let mut rows =
                    stmt.query([&self.config.get_refcounty(), &refsettlement, &street])?;
                while let Some(row) = rows.next()? {
                    let housenumber: String = row.get(0).unwrap();
                    let mut comment: String = row.get(1).unwrap();
                    let suffix = Relation::get_ref_suffix(if comment.is_empty() { 0 } else { 1 });
                    if comment == " " {
                        comment = "".into();
                    }
                    lst.push(street.clone() + "\t" + &housenumber + suffix + "\t" + &comment);
                }
            }
        }

        lst.sort();
        lst.dedup();
        let stream = self
            .file
            .get_ref_housenumbers_write_stream(&self.ctx)
            .context("get_ref_housenumbers_write_stream() failed")?;
        let mut guard = stream.borrow_mut();
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
    fn get_ref_housenumbers(
        &self,
        osm_street_names: &[util::Street],
    ) -> anyhow::Result<HashMap<String, Vec<util::HouseNumber>>> {
        let mut ret: HashMap<String, Vec<util::HouseNumber>> = HashMap::new();
        let mut lines: HashMap<String, Vec<String>> = HashMap::new();
        let read: Rc<RefCell<dyn Read>> = self.file.get_ref_housenumbers_read_stream(&self.ctx)?;
        let mut guard = read.borrow_mut();
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
            lines.entry(key).or_insert_with(Vec::new).push(value.into());
        }
        let street_ranges = self
            .get_street_ranges()
            .context("get_street_ranges() failed")?;
        let streets_invalid = self.get_street_invalid();
        for osm_street in osm_street_names {
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
            house_numbers.sort_unstable();
            house_numbers.dedup();
            ret.insert(
                osm_street_name.into(),
                util::sort_numerically(&house_numbers),
            );
        }
        Ok(ret)
    }

    /// Compares ref and osm house numbers, prints the ones which are in ref, but not in osm.
    /// Return value is a pair of ongoing and done streets.
    /// Each of of these is a pair of a street name and a house number list.
    pub fn get_missing_housenumbers(&mut self) -> anyhow::Result<MissingHousenumbers> {
        let mut ongoing_streets = Vec::new();
        let mut done_streets = Vec::new();

        let osm_street_names = self.get_osm_streets(/*sorted_result=*/ true)?;
        let all_ref_house_numbers = self
            .get_ref_housenumbers(&osm_street_names)
            .context("get_ref_housenumbers() failed")?;
        for osm_street in osm_street_names {
            let osm_street_name = osm_street.get_osm_name();
            let ref_house_numbers = &all_ref_house_numbers[osm_street_name];
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
                ongoing_streets.push(util::NumberedStreet {
                    street: street.clone(),
                    house_numbers: only_in_reference,
                })
            }
            if !in_both.is_empty() {
                done_streets.push(util::NumberedStreet {
                    street,
                    house_numbers: in_both,
                });
            }
        }
        // Sort by length, reverse.
        ongoing_streets.sort_by(|a, b| b.house_numbers.len().cmp(&a.house_numbers.len()));

        Ok(MissingHousenumbers {
            ongoing_streets,
            done_streets,
        })
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
        only_in_osm.retain(|i| !osm_street_blacklist.contains(i.get_osm_name()));

        Ok(only_in_osm)
    }

    /// Calculate and write stat for the street coverage of a relation.
    pub fn write_missing_streets(&self) -> anyhow::Result<(usize, usize, f64, Vec<String>)> {
        let (todo_streets, done_streets) = self.get_missing_streets()?;
        let streets = todo_streets.clone();
        let todo_count = todo_streets.len();
        let done_count = done_streets.len();
        let percent: f64 = if done_count > 0 || todo_count > 0 {
            let float: f64 = done_count as f64 / (done_count as f64 + todo_count as f64) * 100_f64;
            float
        } else {
            100_f64
        };

        // Write the bottom line to a file, so the index page show it fast.
        let string = format!("{percent:.2}");
        self.ctx
            .get_file_system()
            .write_from_string(&string, &self.file.get_streets_percent_path())?;

        Ok((todo_count, done_count, percent, streets))
    }

    /// Calculate and write stat for the unexpected street coverage of a relation.
    pub fn write_additional_streets(&self) -> anyhow::Result<Vec<util::Street>> {
        let additional_streets = self.get_additional_streets(/*sorted_result=*/ true)?;

        // Write the count to a file, so the index page show it fast.
        let file = &self.file;
        self.ctx.get_file_system().write_from_string(
            &additional_streets.len().to_string(),
            &file.get_streets_additional_count_path(),
        )?;

        Ok(additional_streets)
    }

    /// Gets a street name -> valid map, which allows silencing individual false positives.
    fn get_street_valid(&self) -> HashMap<String, Vec<String>> {
        let mut valid_dict: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(ref filters) = self.config.get_filters() {
            for (street, street_filter) in filters {
                if let Some(ref valid) = street_filter.valid {
                    valid_dict.insert(street.clone(), valid.to_vec());
                }
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
            let mut row: Vec<yattag::Doc> = vec![result.street.to_html()];
            let number_ranges = util::get_housenumber_ranges(&result.house_numbers);
            row.push(yattag::Doc::from_text(&number_ranges.len().to_string()));

            let doc = yattag::Doc::new();
            if !self
                .config
                .get_street_is_even_odd(result.street.get_osm_name())
            {
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
    ) -> anyhow::Result<(usize, usize, usize, f64, yattag::HtmlTable)> {
        let json = cache::get_missing_housenumbers_json(self)?;
        let missing_housenumbers: MissingHousenumbers = serde_json::from_str(&json)?;

        let (table, todo_count) =
            self.numbered_streets_to_table(&missing_housenumbers.ongoing_streets);

        let mut done_count = 0;
        for result in missing_housenumbers.done_streets {
            let number_ranges = util::get_housenumber_ranges(&result.house_numbers);
            done_count += number_ranges.len();
        }
        let percent: f64 = if done_count > 0 || todo_count > 0 {
            let float: f64 = done_count as f64 / (done_count as f64 + todo_count as f64) * 100_f64;
            float
        } else {
            100_f64
        };

        // Write the bottom line to a file, so the index page show it fast.
        self.ctx.get_file_system().write_from_string(
            &format!("{percent:.2}"),
            &self.file.get_housenumbers_percent_path(),
        )?;

        Ok((
            missing_housenumbers.ongoing_streets.len(),
            todo_count,
            done_count,
            percent,
            table,
        ))
    }

    /// Compares ref and osm house numbers, prints the ones which are in osm, but not in ref.
    /// Return value is a list of streets.
    /// Each of of these is a pair of a street name and a house number list.
    pub fn get_additional_housenumbers(&mut self) -> anyhow::Result<util::NumberedStreets> {
        let mut additional = Vec::new();

        let osm_street_names = self.get_osm_streets(/*sorted_result=*/ true)?;
        let all_ref_house_numbers = self.get_ref_housenumbers(&osm_street_names)?;
        let streets_valid = self.get_street_valid();
        for osm_street in osm_street_names {
            let osm_street_name = osm_street.get_osm_name();
            let ref_house_numbers = &all_ref_house_numbers[osm_street_name];
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
                additional.push(util::NumberedStreet {
                    street,
                    house_numbers: only_in_osm,
                })
            }
        }
        // Sort by length, reverse.
        additional.sort_by(|a, b| b.house_numbers.len().cmp(&a.house_numbers.len()));

        Ok(additional)
    }

    /// Calculate and write stat for the unexpected house number coverage of a relation.
    /// Returns a tuple of: todo street count, todo count and table.
    pub fn write_additional_housenumbers(
        &mut self,
    ) -> anyhow::Result<(usize, usize, yattag::HtmlTable)> {
        let json = cache::get_additional_housenumbers_json(self)?;
        let ongoing_streets: util::NumberedStreets = serde_json::from_str(&json)?;

        let (table, todo_count) = self.numbered_streets_to_table(&ongoing_streets);

        // Write the street count to a file, so the index page show it fast.
        let file = &self.file;
        self.ctx.get_file_system().write_from_string(
            &todo_count.to_string(),
            &file.get_housenumbers_additional_count_path(),
        )?;

        Ok((ongoing_streets.len(), todo_count, table))
    }

    /// Produces a query which lists house numbers in relation.
    pub fn get_osm_housenumbers_query(&self) -> anyhow::Result<String> {
        let contents = self.ctx.get_file_system().read_to_string(&format!(
            "{}/{}",
            self.ctx.get_abspath("data"),
            "street-housenumbers-template.overpassql"
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
        };
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

    pub fn get_ctx(&mut self) -> &mut context::Context {
        &mut self.ctx
    }
}

/// List of relations from data/relations.yaml.
pub type RelationsDict = HashMap<String, RelationDict>;

/// A relations object is a container of named relation objects.
pub struct Relations {
    ctx: context::Context,
    yaml_cache: HashMap<String, serde_json::Value>,
    dict: RelationsDict,
    relations: HashMap<String, Relation>,
    activate_all: bool,
    activate_new: bool,
    refcounty_names: HashMap<String, String>,
    refsettlement_names: HashMap<String, HashMap<String, String>>,
}

impl Relations {
    pub fn new(ctx: &context::Context) -> anyhow::Result<Self> {
        let yamls_cache_path = format!("{}/{}", ctx.get_abspath("data"), "yamls.cache");
        let mut yaml_cache: HashMap<String, serde_json::Value> = HashMap::new();
        if let Ok(stream) = ctx.get_file_system().open_read(&yamls_cache_path) {
            let mut guard = stream.borrow_mut();
            let read = guard.deref_mut();
            yaml_cache = serde_json::from_reader(read)?;
        }
        let mut dict: RelationsDict = HashMap::new();
        if let Some(value) = yaml_cache.get("relations.yaml") {
            dict =
                serde_json::from_value(value.clone()).context("failed to parse relations.yaml")?;
        }
        let relations: HashMap<String, Relation> = HashMap::new();
        let activate_all = false;
        let activate_new = false;
        let refcounty_names: HashMap<String, String> = match yaml_cache.get("refcounty-names.yaml")
        {
            Some(value) => serde_json::from_value(value.clone())
                .context("failed to parse refcounty-names.yaml")?,
            None => HashMap::new(),
        };
        let refsettlement_names: HashMap<String, HashMap<String, String>> =
            match yaml_cache.get("refsettlement-names.yaml") {
                Some(value) => serde_json::from_value(value.clone())
                    .context("failed to parse refsettlement-names.yaml")?,
                None => HashMap::new(),
            };
        Ok(Relations {
            ctx: ctx.clone(),
            yaml_cache,
            dict,
            relations,
            activate_all,
            activate_new,
            refcounty_names,
            refsettlement_names,
        })
    }

    /// Gets the relation that has the specified name.
    pub fn get_relation(&mut self, name: &str) -> anyhow::Result<Relation> {
        if !self.relations.contains_key(name) {
            let relation = Relation::new(
                &self.ctx,
                name,
                self.dict
                    .entry(name.to_string())
                    .or_insert_with(RelationDict::default),
                &self.yaml_cache,
            )?;
            self.relations.insert(name.into(), relation);
        }

        Ok(self.relations[name].clone())
    }

    /// Gets a sorted list of relation names.
    pub fn get_names(&self) -> Vec<String> {
        let mut ret: Vec<String> = self.dict.keys().map(|key| key.into()).collect();
        ret.sort();
        ret.dedup();
        ret
    }

    fn is_new(&self, relation: &Relation) -> bool {
        if !self.activate_new {
            return false;
        }

        let file_system = self.ctx.get_file_system();
        let files = relation.get_files();
        if file_system.path_exists(&files.get_osm_streets_path())
            && file_system.path_exists(&files.get_osm_housenumbers_path())
            && file_system.path_exists(&files.get_ref_streets_path())
            && file_system.path_exists(&files.get_ref_housenumbers_path())
            && file_system.path_exists(&files.get_streets_percent_path())
            && file_system.path_exists(&files.get_housenumbers_percent_path())
        {
            return false;
        }
        true
    }

    /// Gets a sorted list of active relation names.
    pub fn get_active_names(&mut self) -> anyhow::Result<Vec<String>> {
        let mut active_relations: Vec<Relation> = Vec::new();
        for relation in self.get_relations()? {
            if self.activate_all || relation.config.is_active() || self.is_new(&relation) {
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

    /// Activates relations which don't have state in workdir/ yet.
    pub fn activate_new(&mut self) {
        self.activate_new = true;
    }

    /// If refcounty is not None, forget about all relations outside that refcounty.
    pub fn limit_to_refcounty(&mut self, refcounty: &Option<&String>) -> anyhow::Result<()> {
        let refcounty: String = match refcounty {
            Some(value) => value.to_string(),
            None => {
                return Ok(());
            }
        };
        let relation_names: Vec<String> = self.dict.keys().cloned().collect();
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
    pub fn limit_to_refsettlement(
        &mut self,
        refsettlement: &Option<&String>,
    ) -> anyhow::Result<()> {
        let refsettlement: String = match refsettlement {
            Some(value) => value.to_string(),
            None => {
                return Ok(());
            }
        };
        let relation_names: Vec<String> = self.dict.keys().cloned().collect();
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

/// Expands numbers_nofilter into a list of numbers, returns ret_numbers otherwise.
fn normalize_expand(
    street_is_even_odd: bool,
    separator: &str,
    normalizer: &ranges::Ranges,
    numbers_nofilter: &[i64],
    mut ret_numbers: Vec<i64>,
) -> Vec<i64> {
    if separator != "-" {
        return ret_numbers;
    }

    let (should_expand, new_stop) = util::should_expand_range(numbers_nofilter, street_is_even_odd);
    if should_expand {
        let start = numbers_nofilter[0];
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

    ret_numbers
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
    let star = '*';
    if house_numbers.ends_with(star) {
        suffix = star.into();
    }

    let normalizer = util::get_normalizer(street_name, normalizers);

    let (mut ret_numbers, ret_numbers_nofilter) =
        util::split_house_number_by_separator(&house_numbers, separator, &normalizer);

    ret_numbers = normalize_expand(
        street_is_even_odd,
        separator,
        &normalizer,
        &ret_numbers_nofilter,
        ret_numbers,
    );

    let check_housenumber_letters =
        ret_numbers.len() == 1 && relation.config.should_check_housenumber_letters();
    if check_housenumber_letters && util::HouseNumber::has_letter_suffix(&house_numbers, &suffix) {
        return normalize_housenumber_letters(&house_numbers, &suffix, &comment);
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
    house_numbers: &str,
    suffix: &str,
    comment: &str,
) -> anyhow::Result<Vec<util::HouseNumber>> {
    let normalized = util::HouseNumber::normalize_letter_suffix(house_numbers, suffix)?;
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
        let _ = writeln!(query, "way[\"name\"=\"{street}\"](r.searchRelation);");
        let _ = writeln!(query, "way[\"name\"=\"{street}\"](area.searchArea);");
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
        let _ = writeln!(query, "{osm_type}({osm_id});");
    }
    query += r#");
out body;
>;
out skel qt;"#;
    query
}

#[cfg(test)]
mod tests;
