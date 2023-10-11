/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The util module contains functionality shared between other modules.

use crate::areas;
use crate::context;
use crate::i18n;
use crate::i18n::translate as tr;
use crate::overpass_query;
use crate::ranges;
use crate::yattag;
use anyhow::anyhow;
use anyhow::Context;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::ops::DerefMut;

lazy_static! {
    static ref NUMBER_PER_LETTER: regex::Regex =
        regex::Regex::new(r"^([0-9]+)( |/)?([A-Za-z])$").unwrap();
    static ref NUMBER_PER_NUMBER: regex::Regex =
        regex::Regex::new(r"^([0-9]+)(/)([0-9])$").unwrap();
    static ref NUMBER_WITH_JUNK: regex::Regex = regex::Regex::new(r"([0-9]+).*").unwrap();
    static ref NUMBER_WITH_REMAINDER: regex::Regex =
        regex::Regex::new(r"^([0-9]*)([^0-9].*|)$").unwrap();
    static ref LETTER_SUFFIX: regex::Regex = regex::Regex::new(r".*([A-Za-z]+)\*?").unwrap();
    static ref NUMBER_SUFFIX: regex::Regex = regex::Regex::new(r"^.*/([0-9])\*?$").unwrap();
    static ref NULL_END: regex::Regex = regex::Regex::new(r" null$").unwrap();
    static ref GIT_HASH: regex::Regex = regex::Regex::new(r".*-g([0-9a-f]+)(-modified)?").unwrap();
}

/// A house number range is a string that may expand to one or more HouseNumber instances in the
/// future. It can also have a comment.
#[derive(Clone, Debug)]
pub struct HouseNumberRange {
    number: String,
    comment: String,
}

impl HouseNumberRange {
    fn new(number: &str, comment: &str) -> Self {
        HouseNumberRange {
            number: number.into(),
            comment: comment.into(),
        }
    }

    /// Returns the house number (range) string.
    pub fn get_number(&self) -> &String {
        &self.number
    }

    /// Returns the house number string in '42a' form (as opposed to '42/A').
    pub fn get_lowercase_number(&self) -> String {
        let re = regex::Regex::new(r"^(.*[0-9]+)( |/)([A-Za-z])(.*)$").unwrap();
        if let Some(cap) = re.captures_iter(&self.number).next() {
            let prefix = cap[1].to_string();
            let letter = cap[3].to_string();
            let suffix = cap[4].to_string();
            return format!("{}{}{}", prefix, letter.to_lowercase(), suffix);
        }
        self.number.to_string()
    }

    /// Returns the comment.
    fn get_comment(&self) -> &String {
        &self.comment
    }
}

impl Ord for HouseNumberRange {
    /// Comment is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

impl PartialOrd for HouseNumberRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HouseNumberRange {
    /// Comment is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Eq for HouseNumberRange {}

/// Used to diff two lists of elements.
pub trait Diff {
    /// Gets a string that is used while diffing.
    fn get_diff_key(&self) -> String;
}

/// A street has an OSM and a reference name. Ideally the two are the same. Sometimes the reference
/// name differs.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Street {
    osm_name: String,
    ref_name: String,
    show_ref_street: bool,
    osm_id: u64,
    osm_type: String,
    source: String,
}

impl Street {
    /// Constructor that requires both an OSM and a ref name.
    pub fn new(osm_name: &str, ref_name: &str, show_ref_street: bool, osm_id: u64) -> Street {
        Street {
            osm_name: osm_name.into(),
            ref_name: ref_name.into(),
            show_ref_street,
            osm_id,
            osm_type: "way".into(),
            source: "".into(),
        }
    }

    /// Constructor that only requires an OSM name.
    pub fn from_string(osm_name: &str) -> Street {
        Street::new(osm_name, "", true, 0)
    }

    /// Returns the OSM name.
    pub fn get_osm_name(&self) -> &String {
        &self.osm_name
    }

    /// Returns the OSM (way) id.
    pub fn get_osm_id(&self) -> u64 {
        self.osm_id
    }

    /// Sets the OSM type, e.g. 'way'.
    pub fn set_osm_type(&mut self, osm_type: &str) {
        self.osm_type = osm_type.into()
    }

    /// Returns the OSM type, e.g. 'way'.
    pub fn get_osm_type(&self) -> &String {
        &self.osm_type
    }

    /// Sets the source of this street.
    pub fn set_source(&mut self, source: &str) {
        self.source = source.into()
    }

    /// Gets the source of this street.
    pub fn get_source(&self) -> &str {
        &self.source
    }

    /// Writes the street as a HTML string.
    pub fn to_html(&self) -> yattag::Doc {
        let doc = yattag::Doc::new();
        doc.text(&self.osm_name);
        if self.osm_name != self.ref_name && self.show_ref_street {
            doc.stag("br");
            doc.text("(");
            doc.text(&self.ref_name);
            doc.text(")");
        }
        doc
    }
}

impl Ord for Street {
    /// OSM id is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.osm_name.cmp(&other.osm_name)
    }
}

impl PartialOrd for Street {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Diff for Street {
    fn get_diff_key(&self) -> String {
        let re = regex::Regex::new(r"\*$").unwrap();
        re.replace(&self.osm_name, "").to_string()
    }
}

impl PartialEq for Street {
    /// OSM id is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.osm_name == other.osm_name
    }
}

impl Eq for Street {}

/// A house number is a string which remembers what was its provider range.  E.g. the "1-3" string
/// can generate 3 house numbers, all of them with the same range.
/// The comment is similar to source, it's ignored during eq() and hash().
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HouseNumber {
    number: String,
    source: String,
    comment: String,
    /// Only used for osm housenumbers, not ref ones.
    id: Option<u64>,
    /// Only used for osm housenumbers, not ref ones.
    object_type: Option<String>,
}

/// A list of HouseNumber values.
pub type HouseNumbers = Vec<HouseNumber>;

/// A numbered street is a street with associated house numbers.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NumberedStreet {
    /// The common part for all housenumbers.
    pub street: Street,
    /// Housenumbers of this street.
    pub house_numbers: HouseNumbers,
}

/// A list of NumberedStreet values.
pub type NumberedStreets = Vec<NumberedStreet>;

impl HouseNumber {
    /// Creates a new HouseNumber.
    pub fn new(number: &str, source: &str, comment: &str) -> Self {
        let id: Option<u64> = None;
        let object_type = None;
        HouseNumber {
            number: number.into(),
            source: source.into(),
            comment: comment.into(),
            id,
            object_type,
        }
    }

    /// Returns the house number string.
    pub fn get_number(&self) -> &str {
        &self.number
    }

    /// Returns the source range.
    fn get_source(&self) -> &str {
        &self.source
    }

    /// Returns the comment.
    fn get_comment(&self) -> &str {
        &self.comment
    }

    /// Decides if house_number is invalid according to invalids.
    pub fn is_invalid(
        house_number: &str,
        invalids: &[String],
        used_invalids: &mut Vec<String>,
    ) -> bool {
        if invalids.contains(&house_number.to_string()) {
            used_invalids.push(house_number.to_string());
            return true;
        }

        let mut number: String = "".into();
        if let Some(cap) = NUMBER_WITH_JUNK.captures_iter(house_number).next() {
            number = cap[1].into();
        }
        let mut suffix: String = "".into();
        // Check for letter suffix.
        if let Some(cap) = LETTER_SUFFIX.captures_iter(house_number).next() {
            suffix = cap[1].to_string().to_lowercase();
        }
        // If not, then try digit suggfix, but then only '/' is OK as a separator.
        if suffix.is_empty() {
            let mut iter = NUMBER_SUFFIX.captures_iter(house_number);
            if let Some(cap) = iter.next() {
                suffix = "/".into();
                suffix += &cap[1];
            }
        }

        let house_number = number + &suffix;
        let ret = invalids.contains(&house_number);
        if ret {
            used_invalids.push(house_number);
        }
        ret
    }

    /// Determines if the input is a house number, allowing letter suffixes. This means not only
    /// '42' is allowed, but also '42a', '42/a' and '42 a'. Everything else is still considered just
    /// junk after the numbers.
    pub fn has_letter_suffix(house_number: &str, source_suffix: &str) -> bool {
        let mut house_number: String = house_number.into();
        if !source_suffix.is_empty() {
            house_number = house_number[..house_number.len() - source_suffix.len()].into();
        }
        // Check for letter suffix.
        if NUMBER_PER_LETTER.is_match(&house_number) {
            return true;
        }
        // If not, then try digit suggfix, but then only '/' is OK as a separator.
        NUMBER_PER_NUMBER.is_match(&house_number)
    }

    /// Turn '42A' and '42 A' (and their lowercase versions) into '42/A'.
    pub fn normalize_letter_suffix(
        house_number: &str,
        source_suffix: &str,
    ) -> anyhow::Result<String> {
        let mut house_number: String = house_number.into();
        if !source_suffix.is_empty() {
            house_number = house_number[..house_number.len() - source_suffix.len()].into();
        }
        // Check for letter suffix.
        let mut groups: Vec<String> = Vec::new();
        if let Some(cap) = NUMBER_PER_LETTER.captures_iter(&house_number).next() {
            for index in 1..=3 {
                match cap.get(index) {
                    Some(_) => groups.push(cap[index].to_string()),
                    None => groups.push(String::from("")),
                }
            }
        } else {
            // If not, then try digit suggfix, but then only '/' is OK as a separator.
            if let Some(cap) = NUMBER_PER_NUMBER.captures_iter(&house_number).next() {
                for index in 1..=3 {
                    groups.push(cap[index].to_string());
                }
            } else {
                return Err(anyhow!("ValueError"));
            }
        }

        let mut ret: String = groups[0].clone();
        ret += "/";
        ret += &groups[2].to_uppercase();
        ret += source_suffix;
        Ok(ret)
    }

    /// Sets the housenumber's OSM object ID, if this comes from OSM.
    pub fn set_id(&mut self, id: u64) {
        self.id = Some(id);
    }

    /// Gets the housenumber's OSM object ID, if this comes from OSM.
    pub fn get_id(&self) -> Option<u64> {
        self.id
    }

    /// Sets the housenumber's OSM type ID, if this comes from OSM.
    pub fn set_object_type(&mut self, object_type: &str) {
        self.object_type = Some(object_type.to_string());
    }

    /// Gets the housenumber's OSM type ID, if this comes from OSM.
    pub fn get_object_type(&self) -> Option<String> {
        self.object_type.clone()
    }
}

impl Diff for HouseNumber {
    fn get_diff_key(&self) -> String {
        if self.number.ends_with('*') {
            let mut chars = self.number.chars();
            chars.next_back();
            return chars.as_str().into();
        }

        self.number.clone()
    }
}

impl PartialEq for HouseNumber {
    /// Source is explicitly non-interesting.
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl Eq for HouseNumber {}

impl Ord for HouseNumber {
    /// Source is explicitly non-interesting.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

impl PartialOrd for HouseNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// One row in workdir/refs/city_count_<DATE>.tsv.
#[derive(serde::Deserialize)]
pub struct CityCount {
    /// City name.
    #[serde(rename = "CITY")]
    pub city: String,
    /// Reference count of all housenumbers.
    #[serde(rename = "CNT")]
    pub count: u64,
}

/// One row in workdir/refs/zip_count_<DATE>.tsv.
#[derive(serde::Deserialize)]
pub struct ZipCount {
    /// Zip code.
    #[serde(rename = "ZIP")]
    pub zip: String,
    /// Reference count of all housenumbers.
    #[serde(rename = "CNT")]
    pub count: u64,
}

/// Creates a new typed CSV reader.
pub fn make_csv_reader(read: &mut dyn Read) -> csv::Reader<&mut dyn Read> {
    let reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .double_quote(true)
        .from_reader(read);
    reader
}

/// Splits house_number into a numerical and a remainder part.
pub fn split_house_number(house_number: &str) -> (i32, String) {
    let mut number = 0;
    // There will be always a capture, but it may be an empty string.
    let cap = NUMBER_WITH_REMAINDER
        .captures_iter(house_number)
        .next()
        .unwrap();
    if let Ok(value) = cap[1].parse::<i32>() {
        number = value;
    }
    let remainder = cap[2].to_string();
    (number, remainder)
}

/// Wrapper around split_house_number() for HouseNumberRange objects.
pub fn split_house_number_range(house_number: &HouseNumberRange) -> (i32, String) {
    split_house_number(house_number.get_number())
}

/// Separates even and odd numbers.
fn separate_even_odd(
    only_in_ref: &[HouseNumberRange],
    even: &mut Vec<HouseNumberRange>,
    odd: &mut Vec<HouseNumberRange>,
) {
    let mut even_unsorted: Vec<HouseNumberRange> = only_in_ref
        .iter()
        .filter(|i| split_house_number(i.get_number()).0 % 2 == 0)
        .cloned()
        .collect();
    even_unsorted.sort_by(|a, b| {
        split_house_number_range(a)
            .0
            .cmp(&split_house_number_range(b).0)
    });
    *even = even_unsorted;

    let mut odd_unsorted: Vec<HouseNumberRange> = only_in_ref
        .iter()
        .filter(|i| split_house_number(i.get_number()).0 % 2 == 1)
        .cloned()
        .collect();
    odd_unsorted.sort_by(|a, b| {
        split_house_number_range(a)
            .0
            .cmp(&split_house_number_range(b).0)
    });
    *odd = odd_unsorted;
}

/// Formats even and odd numbers.
pub fn format_even_odd(only_in_ref: &[HouseNumberRange]) -> Vec<String> {
    let mut even: Vec<HouseNumberRange> = Vec::new();
    let mut odd: Vec<HouseNumberRange> = Vec::new();
    separate_even_odd(only_in_ref, &mut even, &mut odd);
    let even_numbers: Vec<String> = even.iter().map(|i| i.get_lowercase_number()).collect();
    let even_string = even_numbers.join(", ");
    let odd_numbers: Vec<String> = odd.iter().map(|i| i.get_lowercase_number()).collect();
    let mut elements: Vec<String> = Vec::new();
    let odd_string = odd_numbers.join(", ");
    if !odd_string.is_empty() {
        elements.push(odd_string);
    }
    if !even_string.is_empty() {
        elements.push(even_string);
    }
    elements
}

/// Formats even and odd numbers, HTML version.
pub fn format_even_odd_html(only_in_ref: &[HouseNumberRange]) -> yattag::Doc {
    let mut even: Vec<HouseNumberRange> = Vec::new();
    let mut odd: Vec<HouseNumberRange> = Vec::new();
    separate_even_odd(only_in_ref, &mut even, &mut odd);
    let doc = yattag::Doc::new();
    for (index, elem) in odd.iter().enumerate() {
        if index > 0 {
            doc.text(", ");
        }
        doc.append_value(color_house_number(elem).get_value());
    }
    if !even.is_empty() && !odd.is_empty() {
        doc.stag("br");
    }
    for (index, elem) in even.iter().enumerate() {
        if index > 0 {
            doc.text(", ");
        }
        doc.append_value(color_house_number(elem).get_value());
    }
    doc
}

/// Colors a house number according to its suffix.
pub fn color_house_number(house_number: &HouseNumberRange) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let number = house_number.get_number();
    if !number.ends_with('*') {
        doc.text(number);
        return doc;
    }
    let mut chars = number.chars();
    chars.next_back();
    let number = chars.as_str();
    let title = house_number.get_comment().replace("&#013;", "\n");
    let span = doc.tag("span", &[("style", "color: blue;")]);
    if !title.is_empty() {
        {
            let abbr = doc.tag("abbr", &[("title", title.as_str()), ("tabindex", "0")]);
            abbr.text(number);
        }
    } else {
        span.text(number);
    }
    doc
}

/// One row in workdir/refs/streets_<DATE>.tsv.
#[derive(serde::Deserialize)]
pub struct RefStreet {
    /// County code.
    #[serde(rename = "COUNTY_CODE")]
    pub county: String,
    /// Settlement code.
    #[serde(rename = "SETTLEMENT_CODE")]
    pub settlement: String,
    /// Street name.
    #[serde(rename = "STREET")]
    pub street: String,
}

/// One row in workdir/refs/housenumbers_<DATE>.tsv.
#[derive(serde::Deserialize)]
pub struct RefHouseNumber {
    /// County code.
    #[serde(rename = "COUNTY_CODE")]
    pub county: String,
    /// Settlement code.
    #[serde(rename = "SETTLEMENT_CODE")]
    pub settlement: String,
    /// Street name.
    #[serde(rename = "STREET")]
    pub street: String,
    /// House number.
    #[serde(rename = "HOUSENUMBER")]
    pub housenumber: String,
    /// Comment.
    #[serde(rename = "COMMENT")]
    pub comment: Option<String>,
}

/// Builds an in-database index from the reference TSV (house number version).
pub fn build_reference_index(
    ctx: &context::Context,
    conn: &mut rusqlite::Connection,
    paths: &[String],
) -> anyhow::Result<()> {
    {
        // Check if the TSV is imported already.
        let mut stmt =
            conn.prepare("select count(*) from (select 0 from ref_housenumbers limit 1)")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let count: i64 = row.get(0).unwrap();
            if count > 0 {
                return Ok(());
            }
        }
    }

    for abspath in paths {
        let mut tokens = abspath.split('/');
        let path = tokens.next_back().unwrap();
        if path.starts_with("hazszamok_kieg") {
            let stream = ctx.get_file_system().open_read(abspath)?;
            let mut guard = stream.borrow_mut();
            let read = std::io::BufReader::new(guard.deref_mut());
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .double_quote(true)
                .from_reader(read);
            let tx = conn.transaction()?;
            for result in reader.deserialize() {
                let row: RefHouseNumber = result?;
                tx.execute(
                        "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values (?1, ?2, ?3, ?4, ?5)",
                        [row.county, row.settlement, row.street, row.housenumber, row.comment.unwrap_or(" ".into())],
                    )?;
            }
            tx.commit()?;
        } else {
            // Assume "hazszamok_".
            let stream = ctx.get_file_system().open_read(abspath)?;
            let mut guard = stream.borrow_mut();
            let read = std::io::BufReader::new(guard.deref_mut());
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .double_quote(true)
                .from_reader(read);
            let tx = conn.transaction()?;
            for result in reader.deserialize() {
                let row: RefHouseNumber = result?;
                tx.execute(
                          "insert into ref_housenumbers (county_code, settlement_code, street, housenumber, comment) values (?1, ?2, ?3, ?4, '')",
                          [row.county, row.settlement, row.street, row.housenumber],
                      )?;
            }
            tx.commit()?;
        }
    }

    Ok(())
}

/// Builds an in-database index from the reference TSV (street version).
pub fn build_street_reference_index(
    ctx: &context::Context,
    conn: &mut rusqlite::Connection,
    path: &str,
) -> anyhow::Result<()> {
    {
        // Check if the TSV is imported already.
        let mut stmt = conn.prepare("select count(*) from (select 0 from ref_streets limit 1)")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let count: i64 = row.get(0).unwrap();
            if count > 0 {
                return Ok(());
            }
        }
    }

    let stream = ctx.get_file_system().open_read(path)?;
    let mut guard = stream.borrow_mut();
    let read = std::io::BufReader::new(guard.deref_mut());
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .double_quote(true)
        .from_reader(read);
    let tx = conn.transaction()?;
    for result in reader.deserialize() {
        let row: RefStreet = result?;
        let street = NULL_END.replace(&row.street, "").to_string();
        tx.execute(
            "insert into ref_streets (county_code, settlement_code, street) values (?1, ?2, ?3)",
            [row.county, row.settlement, street],
        )?;
    }
    tx.commit()?;

    Ok(())
}

/// Parses a filter description, like 'filter-for', 'refcounty', '42'.
pub fn parse_filters(tokens: &[String]) -> HashMap<String, String> {
    let mut ret: HashMap<String, String> = HashMap::new();
    let mut filter_for = false;
    for (index, value) in tokens.iter().enumerate() {
        if value == "filter-for" {
            filter_for = true;
            continue;
        }

        if !filter_for {
            continue;
        }

        if value == "incomplete" || value == "everything" {
            ret.insert(value.clone(), "".into());
        }

        if index + 1 >= tokens.len() {
            continue;
        }

        if ["refcounty", "refsettlement", "relations"].contains(&value.as_str()) {
            ret.insert(value.clone(), tokens[index + 1].clone());
        }
    }
    ret
}

/// Handles a HTTP error from Overpass.
pub fn handle_overpass_error(ctx: &context::Context, http_error: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let div = doc.tag("div", &[("id", "overpass-error")]);
    div.text(&tr("Overpass error: {0}").replace("{0}", http_error));
    let sleep = overpass_query::overpass_query_need_sleep(ctx);
    if sleep > 0 {
        doc.stag("br");
        doc.text(&tr("Note: wait for {} seconds").replace("{}", &sleep.to_string()));
    }
    doc
}

/// Provides localized strings for this thread.
pub fn setup_localization(ctx: &context::Context, headers: rouille::HeadersIter<'_>) -> String {
    let mut languages: String = "".into();
    for (key, value) in headers {
        if key == "Accept-Language" {
            languages = value.into();
        }
    }
    if !languages.is_empty() {
        let parsed = accept_language::parse(&languages);
        if !parsed.is_empty() {
            let language = parsed[0].clone();
            i18n::set_language(ctx, &language);
            return language;
        }
    }
    "".into()
}

/// Generates a link to a URL with a given label.
pub fn gen_link(url: &str, label: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let a = doc.tag("a", &[("href", url)]);
    a.text(&(label.to_string() + "..."));
    doc
}

/// Produces the verify first line of a HTML output.
pub fn write_html_header(doc: &yattag::Doc) {
    doc.append_value("<!DOCTYPE html>\n".into())
}

/// Turns an overpass query template to an actual query.
pub fn process_template(buf: &str, osm_relation: u64) -> String {
    let mut buf = buf.replace("@RELATION@", &osm_relation.to_string());
    // area is relation + 3600000000 (3600000000 == relation), see js/ide.js
    // in https://github.com/tyrasd/overpass-turbo
    buf = buf.replace("@AREA@", &(3600000000 + osm_relation).to_string());
    buf
}

/// Decides if an x-y range should be expanded. Returns a sanitized end value as well.
pub fn should_expand_range(numbers: &[i64], street_is_even_odd: bool) -> (bool, i64) {
    if numbers.len() != 2 {
        return (false, 0);
    }

    if numbers[1] < numbers[0] {
        // E.g. 42-1, -1 is just a suffix to be ignored.
        return (true, 0);
    }

    // If there is a parity mismatch, ignore.
    if street_is_even_odd && numbers[0] % 2 != numbers[1] % 2 {
        return (false, 0);
    }

    // Assume that 0 is just noise.
    if numbers[0] == 0 {
        return (false, 0);
    }

    // Ranges larger than this are typically just noise in the input data.
    if numbers[1] > 1000 || numbers[1] - numbers[0] > 24 {
        return (false, 0);
    }

    (true, numbers[1])
}

/// Produces a HTML table from a list of lists.
pub fn html_table_from_list(table: &[Vec<yattag::Doc>]) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let table_tag = doc.tag("table", &[("class", "sortable")]);
    for (row_index, row_content) in table.iter().enumerate() {
        let tr = table_tag.tag("tr", &[]);
        for cell in row_content {
            if row_index == 0 {
                let th = tr.tag("th", &[]);
                let a = th.tag("a", &[("href", "#")]);
                a.text(&cell.get_value());
            } else {
                let td = tr.tag("td", &[]);
                td.append_value(cell.get_value())
            }
        }
    }
    doc
}

/// Produces HTML enumerations for 2 string lists.
pub fn invalid_refstreets_to_html(osm_invalids: &[String], ref_invalids: &[String]) -> yattag::Doc {
    let doc = yattag::Doc::new();
    if !osm_invalids.is_empty() {
        doc.stag("br");
        let div = doc.tag("div", &[("id", "osm-invalids-container")]);
        div.text(&tr(
            "Warning: broken OSM <-> reference mapping, the following OSM names are invalid:",
        ));
        let ul = doc.tag("ul", &[]);
        for osm_invalid in osm_invalids {
            let li = ul.tag("li", &[]);
            li.text(osm_invalid);
        }
    }
    if !ref_invalids.is_empty() {
        doc.stag("br");
        let div = doc.tag("div", &[("id", "ref-invalids-container")]);
        div.text(&tr(
            "Warning: broken OSM <-> reference mapping, the following reference names are invalid:",
        ));
        let ul = doc.tag("ul", &[]);
        for ref_invalid in ref_invalids {
            let li = ul.tag("li", &[]);
            li.text(ref_invalid);
        }
    }
    if !osm_invalids.is_empty() || !ref_invalids.is_empty() {
        doc.stag("br");
        doc.text(&tr(
            "Note: an OSM name is invalid if it's not in the OSM database. ",
        ));
        doc.text(&tr(
            "A reference name is invalid if it's in the OSM database or it's not in the reference.",
        ));
    }
    doc
}

/// Produces HTML enumerations for a string list.
pub fn invalid_filter_keys_to_html(invalids: &[String]) -> yattag::Doc {
    let doc = yattag::Doc::new();
    if !invalids.is_empty() {
        doc.stag("br");
        let div = doc.tag("div", &[("id", "osm-filter-key-invalids-container")]);
        div.text(&tr(
            "Warning: broken filter key name, the following key names are not OSM names:",
        ));
        let ul = doc.tag("ul", &[]);
        for invalid in invalids {
            let li = ul.tag("li", &[]);
            li.text(invalid);
        }
    }
    doc
}

/// Gets the nth column of row.
fn get_column(row: &[yattag::Doc], column_index: usize) -> String {
    let ret: String = if column_index >= row.len() {
        row[0].get_value()
    } else {
        row[column_index].get_value()
    };
    ret
}

/// Interpret the content as an integer.
fn natnum(column: &str) -> u64 {
    let mut number: String = "".into();
    if let Some(cap) = NUMBER_WITH_JUNK.captures_iter(column).next() {
        number = cap[1].into();
    }
    number.parse::<u64>().unwrap_or(0)
}

/// Turns a tab-separated table into a list of lists.
pub fn tsv_to_list(read: &mut dyn Read) -> anyhow::Result<Vec<Vec<yattag::Doc>>> {
    let mut csv_read = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .double_quote(true)
        .from_reader(read);
    let mut table: Vec<Vec<yattag::Doc>> = Vec::new();

    let mut first = true;
    let mut columns: HashMap<String, usize> = HashMap::new();
    for result in csv_read.records() {
        let row = result?;
        if first {
            first = false;
            for (index, label) in row.iter().enumerate() {
                columns.insert(label.into(), index);
            }
        }
        let mut cells: Vec<yattag::Doc> = row.iter().map(yattag::Doc::from_text).collect();
        if !cells.is_empty() && columns.contains_key("@type") {
            // We know the first column is an OSM ID.
            if let Ok(osm_id) = cells[0].get_value().parse::<u64>() {
                let osm_type = cells[columns["@type"]].get_value();
                let doc = yattag::Doc::new();
                let href = format!("https://www.openstreetmap.org/{osm_type}/{osm_id}");
                {
                    let a = doc.tag("a", &[("href", href.as_str()), ("target", "_blank")]);
                    a.text(&osm_id.to_string());
                }
                cells[0] = doc;
            }
        }
        table.push(cells);
    }

    if columns.contains_key("addr:street") && columns.contains_key("addr:housenumber") {
        let header = table[0].clone();
        table.remove(0);
        table.sort_by(|a, b| {
            let a_key = natnum(&get_column(a, columns["addr:housenumber"]));
            let b_key = natnum(&get_column(b, columns["addr:housenumber"]));
            a_key.cmp(&b_key)
        });
        table.sort_by(|a, b| {
            let a_key = natnum(&get_column(a, columns["addr:street"]));
            let b_key = natnum(&get_column(b, columns["addr:street"]));
            a_key.cmp(&b_key)
        });
        let mut merged = vec![header];
        merged.append(&mut table);
        table = merged;
    }

    Ok(table)
}

/// One row in workdir/street-housenumbers-<relation>.csv. Keep this in sync with
/// data/street-housenumbers-template.overpassql.
#[derive(serde::Deserialize)]
pub struct OsmHouseNumber {
    /// Object ID.
    #[serde(rename = "@id")]
    pub id: u64,
    /// House number.
    #[serde(rename = "addr:housenumber")]
    pub housenumber: String,
    /// Conscription number.
    #[serde(rename = "addr:conscriptionnumber")]
    pub conscriptionnumber: String,
    /// Street name.
    #[serde(rename = "addr:street")]
    pub street: String,
    /// Place name.
    #[serde(rename = "addr:place")]
    pub place: Option<String>,
    /// Object type.
    #[serde(rename = "@type")]
    pub object_type: String,
}

/// One row in workdir/stats/<date>.csv. Keep this in sync with
/// data/data/street-housenumbers-hungary.overpassql.
#[derive(serde::Deserialize)]
pub struct OsmLightHouseNumber {
    /// Postal code.
    #[serde(rename = "addr:postcode")]
    pub postcode: String,
    /// City name.
    #[serde(rename = "addr:city")]
    pub city: String,
    /// Street name.
    #[serde(rename = "addr:street")]
    pub street: String,
    /// House number.
    #[serde(rename = "addr:housenumber")]
    pub housenumber: String,
    /// Last editor.
    #[serde(rename = "@user")]
    pub user: String,
    /// OSM object ID.
    #[serde(rename = "@id")]
    pub osm_id: String,
    /// OSM object type.
    #[serde(rename = "@type")]
    pub osm_type: String,
    /// OSM object timestamp.
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Need further attention.
    #[serde(rename = "fixme")]
    pub fixme: String,
}

/// Reads a house number CSV and extracts streets from rows.
/// Returns a list of street objects, with their name, ID and type set.
pub fn get_street_from_housenumber(
    csv_read: &mut csv::Reader<&mut dyn Read>,
) -> anyhow::Result<Vec<Street>> {
    let mut ret: Vec<Street> = Vec::new();

    for result in csv_read.deserialize() {
        let row: OsmHouseNumber = result?;

        if row.housenumber.is_empty() && row.conscriptionnumber.is_empty() {
            continue;
        }

        let mut street_name = &row.street;

        if street_name.is_empty() {
            if let Some(ref value) = row.place {
                street_name = value;
            }
        }

        if street_name.is_empty() {
            continue;
        }

        let mut street = Street::new(street_name, "", true, row.id);
        street.set_osm_type(&row.object_type);
        street.set_source(&tr("housenumber"));
        ret.push(street);
    }

    Ok(ret)
}

/// Gets a reference range list for a house number list by looking at what range provided a given
/// house number.
pub fn get_housenumber_ranges(house_numbers: &[HouseNumber]) -> Vec<HouseNumberRange> {
    let mut ret: Vec<HouseNumberRange> = Vec::new();
    for house_number in house_numbers {
        ret.push(HouseNumberRange::new(
            house_number.get_source(),
            house_number.get_comment(),
        ));
    }
    ret.sort();
    ret.dedup();
    ret
}

/// Generates a HTML link based on a website prefix and a git-describe version.
pub fn git_link(version: &str, prefix: &str) -> yattag::Doc {
    let mut commit_hash: String = "".into();
    if let Some(cap) = GIT_HASH.captures_iter(version).next() {
        commit_hash = cap[1].into();
    }
    let doc = yattag::Doc::new();
    let a = doc.tag(
        "a",
        &[("href", (prefix.to_string() + &commit_hash).as_str())],
    );
    a.text(version);
    doc
}

/// Sorts strings according to their numerical value, not alphabetically.
pub fn sort_numerically(strings: &[HouseNumber]) -> Vec<HouseNumber> {
    let mut ret: Vec<HouseNumber> = strings.to_owned();
    ret.sort_by_cached_key(|i| split_house_number(i.get_number()));
    ret
}

/// Returns items which are in first, but not in second.
pub fn get_only_in_first<T: Clone + Diff>(first: &[T], second: &[T]) -> Vec<T> {
    if first.is_empty() {
        return Vec::new();
    }

    // Strip suffix that is ignored.
    let second: Vec<String> = second.iter().map(|i| i.get_diff_key()).collect();

    first
        .iter()
        .filter(|i| !second.contains(&i.get_diff_key()))
        .cloned()
        .collect()
}

/// Returns items which are in both first and second.
pub fn get_in_both<T: Clone + Diff>(first: &[T], second: &[T]) -> Vec<T> {
    if first.is_empty() {
        return Vec::new();
    }

    // Strip suffix that is ignored.
    let second: Vec<String> = second.iter().map(|i| i.get_diff_key()).collect();

    first
        .iter()
        .filter(|i| second.contains(&i.get_diff_key()))
        .cloned()
        .collect()
}

/// Determines the normalizer for a given street.
pub fn get_normalizer(
    street_name: &str,
    normalizers: &HashMap<String, ranges::Ranges>,
) -> ranges::Ranges {
    let normalizer: ranges::Ranges = if let Some(value) = normalizers.get(street_name) {
        // Have a custom filter.
        value.clone()
    } else {
        // Default sanity checks.
        let default = vec![
            ranges::Range::new(1, 999, ""),
            ranges::Range::new(2, 998, ""),
        ];
        ranges::Ranges::new(default)
    };
    normalizer
}

/// Splits a house number string (possibly a range) by a given separator.
/// Returns a filtered and a not filtered list of ints.
pub fn split_house_number_by_separator(
    house_numbers: &str,
    separator: &str,
    normalizer: &ranges::Ranges,
    relation_name: &str,
    street_name: &str,
    lints: &mut Option<&mut Vec<areas::RelationLint>>,
    osm_housenumber: Option<&OsmHouseNumber>,
) -> (Vec<i64>, Vec<i64>) {
    let mut ret_numbers: Vec<i64> = Vec::new();
    // Same as ret_numbers, but if the range is 2-6 and we filter for 2-4, then 6 would be lost, so
    // in-range 4 would not be detected, so this one does not drop 6.
    let mut ret_numbers_nofilter: Vec<i64> = Vec::new();

    for house_number in house_numbers.split(separator) {
        let mut number: i64 = 0;
        if let Some(cap) = NUMBER_WITH_JUNK.captures_iter(house_number).next() {
            // If parse() fails, then NUMBER_WITH_JUNK is broken.
            number = cap[1].parse::<i64>().unwrap();
        }

        ret_numbers_nofilter.push(number);

        if !areas::normalizer_contains(
            number,
            normalizer,
            relation_name,
            street_name,
            lints,
            osm_housenumber,
        ) {
            continue;
        }

        ret_numbers.push(number);
    }

    (ret_numbers, ret_numbers_nofilter)
}

/// Constructs a city name based on postcode the nominal city.
pub fn get_city_key(
    postcode: &str,
    city: &str,
    valid_settlements: &HashSet<String>,
) -> anyhow::Result<String> {
    let city = city.to_lowercase();

    if !city.is_empty() && postcode.starts_with('1') {
        let mut chars = postcode.chars();
        chars.next();
        chars.next_back();
        let district = match chars.as_str().parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                return Ok("_Invalid".into());
            }
        };
        if (1..=23).contains(&district) {
            return Ok(city + "_" + chars.as_str());
        }
        return Ok(city);
    }

    if valid_settlements.contains(&city) || city == "budapest" {
        return Ok(city);
    }
    if !city.is_empty() {
        return Ok("_Invalid".into());
    }
    Ok("_Empty".into())
}

/// Returns a string comparator which allows locale-aware lexical sorting.
#[cfg(feature = "icu")]
pub fn get_sort_key(bytes: &str) -> Vec<u8> {
    use rust_icu_ucol as ucol;
    use rust_icu_ustring as ustring;

    // This is good enough for now, English and Hungarian is all we support and this handles both.
    let collator = ucol::UCollator::try_from("hu").expect("UCollator::try_from() failed");
    let string = ustring::UChar::try_from(bytes).expect("UChar::try_from() failed");
    collator.get_sort_key(&string)
}

/// Returns the input as-is to avoid depending on ICU.
#[cfg(not(feature = "icu"))]
pub fn get_sort_key(bytes: &str) -> Vec<u8> {
    bytes.as_bytes().to_vec()
}

/// Builds a set of valid settlement names.
pub fn get_valid_settlements(ctx: &context::Context) -> anyhow::Result<HashSet<String>> {
    let mut settlements: HashSet<String> = HashSet::new();

    let path = ctx.get_ini().get_reference_citycounts_path()?;
    let stream = ctx
        .get_file_system()
        .open_read(&path)
        .context("open_read() failed")?;
    let mut guard = stream.borrow_mut();
    let mut read = guard.deref_mut();
    let mut csv_reader = make_csv_reader(&mut read);
    for result in csv_reader.deserialize() {
        let row: CityCount = match result {
            Ok(value) => value,
            Err(_) => {
                continue;
            }
        };
        settlements.insert(row.city);
    }

    Ok(settlements)
}

/// Formats a percentage, taking locale into account.
#[cfg(feature = "icu")]
pub fn format_percent(parsed: f64) -> anyhow::Result<String> {
    use rust_icu_unumberformatter as unumberformatter;

    let language: &str = &i18n::get_language();
    let formatter = unumberformatter::UNumberFormatter::try_new("% .00", language)?;
    let formatted = formatter.format_double(parsed)?;
    Ok(formatted.try_into()?)
}

/// Formats the input without ICU.
#[cfg(not(feature = "icu"))]
pub fn format_percent(parsed: f64) -> anyhow::Result<String> {
    let _language: &str = &i18n::get_language();
    Ok(format!("{0:.2}%", parsed))
}

/// Gets the mtime of a file if it exists, 0 otherwise.
pub fn get_mtime(ctx: &context::Context, path: &str) -> time::OffsetDateTime {
    let mtime = match ctx.get_file_system().getmtime(path) {
        Ok(value) => value,
        Err(_) => time::OffsetDateTime::UNIX_EPOCH,
    };

    mtime
}

#[cfg(test)]
mod tests;
