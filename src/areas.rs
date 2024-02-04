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
use crate::stats;
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

impl RelationFiltersDict {
    /// Determines if at least one Option is Some.
    pub fn is_some(&self) -> bool {
        self.interpolation.is_some()
            || self.invalid.is_some()
            || self.ranges.is_some()
            || self.valid.is_some()
            || self.refsettlement.is_some()
            || self.show_refstreet.is_some()
    }
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

#[derive(Clone, Ord, PartialOrd, derivative::Derivative)]
#[derivative(Eq, PartialEq)]
pub struct RelationLint {
    pub relation_name: String,
    pub street_name: String,
    /// Type, e.g. invalid or range.
    pub source: RelationLintSource,
    pub housenumber: String,
    /// E.g. missing from reference or present in OSM
    pub reason: RelationLintReason,
    #[derivative(PartialEq = "ignore")]
    pub id: u64,
    #[derivative(PartialEq = "ignore")]
    pub object_type: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum RelationLintSource {
    Range,
    Invalid,
}

impl TryFrom<&str> for RelationLintSource {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "range" => Ok(RelationLintSource::Range),
            "invalid" => Ok(RelationLintSource::Invalid),
            _ => Err(anyhow::anyhow!("invalid value: {value}")),
        }
    }
}

impl rusqlite::types::FromSql for RelationLintSource {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let i = String::column_result(value)?;
        i.as_str()
            .try_into()
            .map_err(|_| rusqlite::types::FromSqlError::InvalidType)
    }
}

impl std::fmt::Display for RelationLintSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationLintSource::Range => write!(f, "range"),
            RelationLintSource::Invalid => write!(f, "invalid"),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum RelationLintReason {
    CreatedInOsm,
    DeletedFromRef,
    OutOfRange,
}

impl TryFrom<&str> for RelationLintReason {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "created-in-osm" => Ok(RelationLintReason::CreatedInOsm),
            "deleted-from-ref" => Ok(RelationLintReason::DeletedFromRef),
            "out-of-range" => Ok(RelationLintReason::OutOfRange),
            _ => Err(anyhow::anyhow!("invalid value: {value}")),
        }
    }
}

impl rusqlite::types::FromSql for RelationLintReason {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let i = String::column_result(value)?;
        i.as_str()
            .try_into()
            .map_err(|_| rusqlite::types::FromSqlError::InvalidType)
    }
}

impl std::fmt::Display for RelationLintReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationLintReason::CreatedInOsm => write!(f, "created-in-osm"),
            RelationLintReason::DeletedFromRef => write!(f, "deleted-from-ref"),
            RelationLintReason::OutOfRange => write!(f, "out-of-range"),
        }
    }
}

/// A relation is a closed polygon on the map.
#[derive(Clone)]
pub struct Relation<'a> {
    ctx: &'a context::Context,
    name: String,
    file: area_files::RelationFiles,
    config: RelationConfig,
    osm_housenumbers: HashMap<String, Vec<util::HouseNumber>>,
    lints: Vec<RelationLint>,
}

impl<'a> Relation<'a> {
    fn new(
        ctx: &'a context::Context,
        name: &str,
        parent_config: &RelationDict,
        yaml_cache: &HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<Relation<'a>> {
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
        let lints: Vec<RelationLint> = Vec::new();
        Ok(Relation {
            ctx,
            name: name.into(),
            file,
            config,
            osm_housenumbers,
            lints,
        })
    }

    pub fn get_lints(&self) -> &Vec<RelationLint> {
        &self.lints
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
        for row in self.file.get_osm_json_streets(self.ctx)? {
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
        if stats::has_sql_mtime(self.ctx, &format!("housenumbers/{}", self.name))? {
            ret.append(
                &mut util::get_street_from_housenumber(
                    &self.file.get_osm_json_housenumbers(self.ctx)?,
                )
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

    /// Produces a query which lists streets in relation, in JSON format.
    pub fn get_osm_streets_json_query(&self) -> anyhow::Result<String> {
        let query = self.get_osm_streets_query()?;
        let mut i = 0;
        let mut lines = Vec::new();
        for line in query.lines() {
            i += 1;
            if i == 1 {
                lines.push("[out:json];".to_string());
                continue;
            }

            lines.push(line.to_string());
        }
        Ok(lines.join("\n"))
    }

    /// Gets streets from reference.
    fn get_ref_streets(&self) -> anyhow::Result<Vec<String>> {
        let mut streets: Vec<String> = Vec::new();
        let read: Rc<RefCell<dyn Read>> = self.file.get_ref_streets_read_stream(self.ctx)?;
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
            let osm_housenumbers = self.file.get_osm_json_housenumbers(self.ctx)?;
            let mut lints: Vec<RelationLint> = Vec::new();
            for row in osm_housenumbers {
                let mut street = &row.street;
                if street.is_empty() {
                    if let Some(ref value) = row.place {
                        street = value;
                    }
                }
                for house_number in row.housenumber.split(&[';', ',']) {
                    house_numbers
                        .entry(street.to_string())
                        .or_default()
                        .append(&mut normalize(
                            self,
                            house_number,
                            street,
                            &street_ranges,
                            &mut Some(&mut lints),
                            Some(&row),
                        )?)
                }
            }
            self.lints.append(&mut lints);
            for (key, mut value) in house_numbers {
                value.sort_unstable();
                value.dedup();
                self.osm_housenumbers
                    .insert(key, util::sort_numerically(&value));
            }

            let streets_invalids = self.get_street_invalid();
            for (street_name, housenumbers) in &self.osm_housenumbers {
                let mut invalids: Vec<String> = Vec::new();
                if let Some(value) = streets_invalids.get(street_name) {
                    invalids = value.clone();
                    invalids = self.normalize_invalids(street_name, &invalids)?;

                    // housenumber letters: OSM data is already in the 42/A, do the same for the
                    // invalid items as well, so contains() makes sense:
                    invalids = invalids
                        .iter()
                        .map(
                            |i| match util::HouseNumber::normalize_letter_suffix(i, "") {
                                Ok(value) => value,
                                Err(_) => i.to_string(),
                            },
                        )
                        .collect();
                }
                for housenumber in housenumbers {
                    if invalids.contains(&housenumber.get_number().to_string()) {
                        let relation_name = self.get_name();
                        let street_name = street_name.to_string();
                        let source = RelationLintSource::Invalid;
                        let reason = RelationLintReason::CreatedInOsm;
                        let id: u64 = housenumber.get_id().context("no osm id")?;
                        let object_type = housenumber.get_object_type().context("no osm type")?;
                        let housenumber = housenumber.get_number().to_string();
                        let lint = RelationLint {
                            relation_name,
                            street_name,
                            source,
                            housenumber,
                            reason,
                            id,
                            object_type,
                        };
                        self.lints.push(lint);
                    }
                }
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
        let mut conn = self.ctx.get_database_connection()?;
        util::build_street_reference_index(self.ctx, &mut conn, reference)?;

        let mut lst: Vec<String> = Vec::new();
        let mut stmt = conn.prepare(
            "select street from ref_streets where county_code = ?1 and settlement_code = ?2",
        )?;
        let mut rows = stmt.query([
            &self.config.get_refcounty(),
            &self.config.get_refsettlement(),
        ])?;
        while let Some(row) = rows.next()? {
            let street: String = row.get(0).unwrap();
            lst.push(street);
        }

        lst.sort();
        lst.dedup();
        let write = self
            .file
            .get_ref_streets_write_stream(self.ctx)
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
        {
            let mut conn = self.ctx.get_database_connection()?;
            util::build_reference_index(self.ctx, &mut conn, references)?;
        }

        let streets: Vec<String> = self
            .get_osm_streets(/*sorted_results=*/ true)?
            .iter()
            .map(|i| i.get_osm_name().into())
            .collect();

        let conn = self.ctx.get_database_connection()?;
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
            .get_ref_housenumbers_write_stream(self.ctx)
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
        for i in street_invalid {
            let normalizeds = normalize(self, i, osm_street_name, &street_ranges, &mut None, None)?;
            // normalize() may return an empty list if the number is out of range.
            if !normalizeds.is_empty() {
                normalized_invalids.push(normalizeds[0].get_number().into())
            }
        }
        Ok(normalized_invalids)
    }

    /// Gets house numbers from reference, produced by write_ref_housenumbers()."""
    fn get_ref_housenumbers(
        &mut self,
        osm_street_names: &[util::Street],
    ) -> anyhow::Result<HashMap<String, Vec<util::HouseNumber>>> {
        let mut ret: HashMap<String, Vec<util::HouseNumber>> = HashMap::new();
        let mut lines: HashMap<String, Vec<String>> = HashMap::new();
        let read: Rc<RefCell<dyn Read>> = self.file.get_ref_housenumbers_read_stream(self.ctx)?;
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
            lines.entry(key).or_default().push(value.into());
        }
        let street_ranges = self
            .get_street_ranges()
            .context("get_street_ranges() failed")?;
        let streets_invalid = self.get_street_invalid();
        for osm_street in osm_street_names {
            let osm_street_name = osm_street.get_osm_name();
            let mut house_numbers: Vec<util::HouseNumber> = Vec::new();
            let ref_street_name = self.config.get_ref_street_from_osm_street(osm_street_name);
            let mut street_invalid: Vec<String> = Vec::new();
            if let Some(value) = streets_invalid.get(osm_street_name) {
                street_invalid = value.clone();

                // Simplify invalid items by default, so the 42a markup can be used, no matter what
                // is the value of housenumber-letters.
                street_invalid = self.normalize_invalids(osm_street_name, &street_invalid)?;
            }

            let mut used_invalids: Vec<String> = Vec::new();
            if let Some(value) = lines.get(&ref_street_name) {
                for house_number in value {
                    let normalized = normalize(
                        self,
                        house_number,
                        osm_street_name,
                        &street_ranges,
                        &mut None,
                        None,
                    )?;
                    house_numbers.append(
                        &mut normalized
                            .iter()
                            .filter(|i| {
                                let is_invalid = util::HouseNumber::is_invalid(
                                    i.get_number(),
                                    &street_invalid,
                                    &mut used_invalids,
                                );
                                !is_invalid
                            })
                            .cloned()
                            .collect(),
                    );
                }
            }

            if let Some(street_invalid) = streets_invalid.get(osm_street_name) {
                // This is the full list of invalid items, before removing the out of range ones.
                for invalid in street_invalid {
                    if !used_invalids.contains(invalid) {
                        let relation_name = self.get_name();
                        let street_name = osm_street.get_osm_name().to_string();
                        let source = RelationLintSource::Invalid;
                        let housenumber = invalid.to_string();
                        let mut reason = RelationLintReason::DeletedFromRef;

                        if let Some(value) = lines.get(&ref_street_name) {
                            // See if this is indeed deleted from the reference or it's just out of
                            // range, so the returned reference doesn't contain it as the range already
                            // filters it out.
                            let housenumbers: Vec<_> = value
                                .iter()
                                .map(|i| i.split('\t').next().unwrap().to_string())
                                .collect();
                            if housenumbers.contains(&housenumber) {
                                // Out of range, not really deleted from reference.
                                reason = RelationLintReason::OutOfRange;
                            }
                        }

                        let id: u64 = 0;
                        let object_type = "".to_string();
                        let lint = RelationLint {
                            relation_name,
                            street_name,
                            source,
                            housenumber,
                            reason,
                            id,
                            object_type,
                        };
                        self.lints.push(lint);
                    }
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
        self.set_osm_street_coverage(&format!("{percent:.2}"))?;

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
        let json = cache::get_missing_housenumbers_json(self)
            .context("get_missing_housenumbers_json() failed")?;
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
        self.set_osm_housenumber_coverage(&format!("{percent:.2}"))?;

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

            let mut used_valids: Vec<String> = Vec::new();
            if let Some(street_valid) = streets_valid.get(osm_street_name) {
                let filtered: Vec<_> = osm_house_numbers
                    .iter()
                    .filter(|i| {
                        !util::HouseNumber::is_invalid(
                            i.get_number(),
                            street_valid,
                            &mut used_valids,
                        )
                    })
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

    /// Produces a query which lists housenumbers in relation, in JSON format.
    pub fn get_osm_housenumbers_json_query(&self) -> anyhow::Result<String> {
        let query = self.get_osm_housenumbers_query()?;
        let mut i = 0;
        let mut lines = Vec::new();
        for line in query.lines() {
            i += 1;
            if i == 1 {
                lines.push("[out:json];".to_string());
                continue;
            }

            lines.push(line.to_string());
        }
        Ok(lines.join("\n"))
    }

    /// Returns invalid osm names and ref names.
    pub fn get_invalid_refstreets(&self) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let mut osm_invalids: Vec<String> = Vec::new();
        let mut ref_invalids: Vec<String> = Vec::new();
        let refstreets = self.config.get_refstreets();
        if refstreets.is_empty() {
            return Ok((osm_invalids, ref_invalids));
        }
        let osm_streets: Vec<String> = self
            .get_osm_streets(/*sorted_result=*/ true)
            .context("get_osm_streets() failed")?
            .iter()
            .map(|i| i.get_osm_name())
            .cloned()
            .collect();
        let ref_streets: Vec<String> = self.get_ref_streets()?;
        for (osm_name, ref_name) in refstreets {
            if !osm_streets.contains(&osm_name) {
                osm_invalids.push(osm_name);
            }
            if !ref_streets.contains(&ref_name) {
                ref_invalids.push(ref_name.to_string());
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

    pub fn get_ctx(&self) -> &context::Context {
        self.ctx
    }

    pub fn has_osm_housenumber_coverage(&self) -> anyhow::Result<bool> {
        let conn = self.ctx.get_database_connection()?;
        let mut stmt = conn
            .prepare("select coverage from osm_housenumber_coverages where relation_name = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        let row = rows.next()?;
        Ok(row.is_some())
    }

    pub fn set_osm_housenumber_coverage(&self, coverage: &str) -> anyhow::Result<()> {
        let conn = self.ctx.get_database_connection()?;
        conn.execute(
            r#"insert into osm_housenumber_coverages (relation_name, coverage, last_modified) values (?1, ?2, ?3)
                 on conflict(relation_name) do update set coverage = excluded.coverage, last_modified = excluded.last_modified"#,
            [&self.name, coverage, &self.ctx.get_time().now().unix_timestamp_nanos().to_string()],
        )?;
        Ok(())
    }

    pub fn write_lints(&mut self) -> anyhow::Result<()> {
        let conn = self.ctx.get_database_connection()?;
        conn.execute(
            "delete from relation_lints where relation_name = ?1",
            [&self.name],
        )?;
        self.lints.sort();
        self.lints.dedup();
        for lint in self.lints.iter() {
            conn.execute(
                r#"insert into relation_lints (relation_name, street_name, source, housenumber, reason, object_id, object_type) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                 [&lint.relation_name, &lint.street_name, &lint.source.to_string(), &lint.housenumber, &lint.reason.to_string(), &lint.id.to_string(), &lint.object_type],
                 )?;
        }
        Ok(())
    }

    pub fn get_osm_housenumber_coverage(&self) -> anyhow::Result<String> {
        let conn = self.ctx.get_database_connection()?;
        let mut stmt = conn
            .prepare("select coverage from osm_housenumber_coverages where relation_name = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        let row = rows.next()?.context("no next row")?;
        let percent: String = row.get(0)?;
        Ok(percent)
    }

    pub fn get_osm_housenumber_coverage_mtime(&self) -> anyhow::Result<time::OffsetDateTime> {
        let conn = self.ctx.get_database_connection()?;
        let mut stmt = conn.prepare(
            "select last_modified from osm_housenumber_coverages where relation_name = ?1",
        )?;
        let mut rows = stmt.query([&self.name])?;
        let row = rows.next()?.context("no next row")?;
        let last_modified: String = row.get(0)?;
        let nanos: i128 = last_modified.parse()?;
        let modified = time::OffsetDateTime::from_unix_timestamp_nanos(nanos)?;
        let now = self.ctx.get_time().now();
        Ok(modified.to_offset(now.offset()))
    }

    pub fn has_osm_street_coverage(&self) -> anyhow::Result<bool> {
        let conn = self.ctx.get_database_connection()?;
        let mut stmt =
            conn.prepare("select coverage from osm_street_coverages where relation_name = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        let row = rows.next()?;
        Ok(row.is_some())
    }

    pub fn set_osm_street_coverage(&self, coverage: &str) -> anyhow::Result<()> {
        let conn = self.ctx.get_database_connection()?;
        conn.execute(
            r#"insert into osm_street_coverages (relation_name, coverage, last_modified) values (?1, ?2, ?3)
                 on conflict(relation_name) do update set coverage = excluded.coverage, last_modified = excluded.last_modified"#,
            [&self.name, coverage, &self.ctx.get_time().now().unix_timestamp_nanos().to_string()],
        )?;
        Ok(())
    }

    pub fn get_osm_street_coverage(&self) -> anyhow::Result<String> {
        let conn = self.ctx.get_database_connection()?;
        let mut stmt =
            conn.prepare("select coverage from osm_street_coverages where relation_name = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        let row = rows.next()?.context("no next row")?;
        let percent: String = row.get(0)?;
        Ok(percent)
    }

    pub fn get_osm_street_coverage_mtime(&self) -> anyhow::Result<time::OffsetDateTime> {
        let conn = self.ctx.get_database_connection()?;
        let mut stmt = conn
            .prepare("select last_modified from osm_street_coverages where relation_name = ?1")?;
        let mut rows = stmt.query([&self.name])?;
        let row = rows.next()?.context("no next row")?;
        let last_modified: String = row.get(0)?;
        let nanos: i128 = last_modified.parse()?;
        let modified = time::OffsetDateTime::from_unix_timestamp_nanos(nanos)?;
        let now = self.ctx.get_time().now();
        Ok(modified.to_offset(now.offset()))
    }
}

/// List of relations from data/relations.yaml.
pub type RelationsDict = HashMap<String, RelationDict>;

/// A relations object is a container of named relation objects.
pub struct Relations<'a> {
    ctx: &'a context::Context,
    yaml_cache: HashMap<String, serde_json::Value>,
    dict: RelationsDict,
    relations: HashMap<String, Relation<'a>>,
    activate_all: bool,
    activate_new: bool,
    activate_invalid: bool,
    refcounty_names: HashMap<String, String>,
    refsettlement_names: HashMap<String, HashMap<String, String>>,
}

impl<'a> Relations<'a> {
    pub fn new(ctx: &'a context::Context) -> anyhow::Result<Relations<'a>> {
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
        let relations: HashMap<String, Relation<'a>> = HashMap::new();
        let activate_all = false;
        let activate_new = false;
        let activate_invalid = false;
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
            ctx,
            yaml_cache,
            dict,
            relations,
            activate_all,
            activate_new,
            activate_invalid,
            refcounty_names,
            refsettlement_names,
        })
    }

    /// Gets the relation that has the specified name.
    pub fn get_relation(&mut self, name: &str) -> anyhow::Result<Relation<'a>> {
        if !self.relations.contains_key(name) {
            let relation = Relation::new(
                self.ctx,
                name,
                self.dict.entry(name.to_string()).or_default(),
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

    fn is_new(&self, relation: &Relation<'a>) -> bool {
        if !self.activate_new {
            return false;
        }

        let file_system = self.ctx.get_file_system();
        let files = relation.get_files();
        let osm_housenumber_coverage_exists = relation.has_osm_housenumber_coverage().unwrap();
        let osm_street_coverage_exists = relation.has_osm_street_coverage().unwrap();
        if stats::has_sql_mtime(self.ctx, &format!("streets/{}", relation.get_name())).unwrap()
            && stats::has_sql_mtime(self.ctx, &format!("housenumbers/{}", relation.get_name()))
                .unwrap()
            && file_system.path_exists(&files.get_ref_streets_path())
            && file_system.path_exists(&files.get_ref_housenumbers_path())
            && osm_street_coverage_exists
            && osm_housenumber_coverage_exists
        {
            return false;
        }
        true
    }

    fn is_invalid(&self, relation: &Relation<'a>) -> anyhow::Result<bool> {
        if !self.activate_invalid {
            return Ok(false);
        }

        let file_system = self.ctx.get_file_system();
        if !stats::has_sql_mtime(self.ctx, &format!("streets/{}", relation.get_name()))? {
            return Ok(false);
        }

        if !file_system.path_exists(&relation.get_files().get_ref_streets_path()) {
            return Ok(false);
        }

        let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;

        let key_invalids = relation.get_invalid_filter_keys()?;

        Ok(!osm_invalids.is_empty() || !ref_invalids.is_empty() || !key_invalids.is_empty())
    }

    /// Gets a sorted list of active relation names.
    pub fn get_active_names(&mut self) -> anyhow::Result<Vec<String>> {
        let mut active_relations: Vec<Relation<'a>> = Vec::new();
        for relation in self.get_relations()? {
            if self.activate_all
                || relation.config.is_active()
                || self.is_new(&relation)
                || self.is_invalid(&relation)?
            {
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
    pub fn get_relations(&mut self) -> anyhow::Result<Vec<Relation<'a>>> {
        let mut ret: Vec<Relation<'a>> = Vec::new();
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

    /// Activates relations with invalid refstreets / filter keys.
    pub fn activate_invalid(&mut self) {
        self.activate_invalid = true;
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

pub fn normalizer_contains(
    number: i64,
    normalizer: &ranges::Ranges,
    relation_name: &str,
    street_name: &str,
    lints: &mut Option<&mut Vec<RelationLint>>,
    osm_housenumber: Option<&util::OsmHouseNumber>,
) -> bool {
    let ret = normalizer.contains(number);
    // number not in the ranges: raise a lint in case the problem is actionable (has street name,
    // has an actual number).
    if !ret && !street_name.is_empty() && number != 0 {
        if let Some(ref mut lints) = lints {
            let relation_name = relation_name.to_string();
            let street_name = street_name.to_string();
            let source = RelationLintSource::Range;
            let housenumber = number.to_string();
            let reason = RelationLintReason::CreatedInOsm;
            let id: u64 = match osm_housenumber {
                Some(value) => value.id,
                None => 0,
            };
            let object_type = match osm_housenumber {
                Some(value) => value.object_type.to_string(),
                None => "".to_string(),
            };
            let lint = RelationLint {
                relation_name,
                street_name,
                source,
                housenumber,
                reason,
                id,
                object_type,
            };
            lints.push(lint);
        }
    }
    ret
}

struct LintedHouseNumber<'a> {
    lints: &'a mut Option<&'a mut Vec<RelationLint>>,
    osm_housenumber: Option<&'a util::OsmHouseNumber>,
}

/// Expands numbers_nofilter into a list of numbers, returns ret_numbers otherwise.
fn normalize_expand(
    relation: &Relation<'_>,
    separator: &str,
    normalizer: &ranges::Ranges,
    numbers_nofilter: &[i64],
    mut ret_numbers: Vec<i64>,
    street_name: &str,
    lhn: LintedHouseNumber<'_>,
) -> Vec<i64> {
    if separator != "-" {
        return ret_numbers;
    }

    let street_is_even_odd = relation.get_config().get_street_is_even_odd(street_name);
    let (should_expand, new_stop) = util::should_expand_range(numbers_nofilter, street_is_even_odd);
    if should_expand {
        let relation_name = &relation.get_name();
        let start = numbers_nofilter[0];
        let stop = new_stop;
        if stop == 0 {
            ret_numbers = [start]
                .iter()
                .filter(|number| {
                    normalizer_contains(
                        **number,
                        normalizer,
                        relation_name,
                        street_name,
                        lhn.lints,
                        lhn.osm_housenumber,
                    )
                })
                .cloned()
                .collect();
        } else if street_is_even_odd {
            // Assume that e.g. 2-6 actually means 2, 4 and 6, not only 2 and 4.
            // Closed interval, even only or odd only case.
            ret_numbers = (start..stop + 2)
                .step_by(2)
                .filter(|number| {
                    normalizer_contains(
                        *number,
                        normalizer,
                        relation_name,
                        street_name,
                        lhn.lints,
                        lhn.osm_housenumber,
                    )
                })
                .collect();
        } else {
            // Closed interval, but mixed even and odd.
            ret_numbers = (start..stop + 1)
                .filter(|number| {
                    normalizer_contains(
                        *number,
                        normalizer,
                        relation_name,
                        street_name,
                        lhn.lints,
                        lhn.osm_housenumber,
                    )
                })
                .collect();
        }
    }

    ret_numbers
}

/// Strips down string input to bare minimum that can be interpreted as an
/// actual number. Think about a/b, a-b, and so on.
fn normalize<'a>(
    relation: &Relation<'_>,
    house_numbers: &str,
    street_name: &str,
    normalizers: &HashMap<String, ranges::Ranges>,
    lints: &'a mut Option<&'a mut Vec<RelationLint>>,
    osm_housenumber: Option<&'a util::OsmHouseNumber>,
) -> anyhow::Result<Vec<util::HouseNumber>> {
    let mut comment: String = "".into();
    let mut house_numbers: String = house_numbers.into();
    if house_numbers.contains('\t') {
        let tokens = house_numbers;
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

    let (mut ret_numbers, ret_numbers_nofilter) = util::split_house_number_by_separator(
        &house_numbers,
        separator,
        &normalizer,
        &relation.get_name(),
        street_name,
        lints,
        osm_housenumber,
    );

    {
        let lhn = LintedHouseNumber {
            lints,
            osm_housenumber,
        };
        ret_numbers = normalize_expand(
            relation,
            separator,
            &normalizer,
            &ret_numbers_nofilter,
            ret_numbers,
            street_name,
            lhn,
        );
    }

    let check_housenumber_letters =
        ret_numbers.len() == 1 && relation.config.should_check_housenumber_letters();
    let ret: Vec<util::HouseNumber> = if check_housenumber_letters
        && util::HouseNumber::has_letter_suffix(&house_numbers, &suffix)
    {
        normalize_housenumber_letters(&house_numbers, &suffix, &comment)?
    } else {
        ret_numbers
            .iter()
            .map(|number| {
                util::HouseNumber::new(&(number.to_string() + &suffix), &house_numbers, &comment)
            })
            .collect()
    };
    // Finally annotate the result in case we got OSM info, regardless of the value of
    // check_housenumber_letters.
    Ok(ret
        .iter()
        .map(|number| {
            let mut housenumber = number.clone();
            if let Some(osm_housenumber) = osm_housenumber {
                housenumber.set_id(osm_housenumber.id);
                housenumber.set_object_type(&osm_housenumber.object_type);
            }
            housenumber
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
pub fn make_turbo_query_for_streets(relation: &Relation<'_>, streets: &[String]) -> String {
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
pub fn make_turbo_query_for_street_objs(
    relation: &Relation<'_>,
    streets: &[util::Street],
) -> String {
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
