/*
 * Copyright 2025 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The serde module contains structs used while parsing data using the serde crate.

/// OverpassTags contains various tags about one Overpass element.
#[derive(serde::Deserialize)]
pub struct OverpassTags {
    pub name: Option<String>,
    pub highway: Option<String>,
    pub service: Option<String>,
    pub surface: Option<String>,
    pub leisure: Option<String>,

    // region: housenumbers
    #[serde(rename(deserialize = "addr:street"))]
    pub street: Option<String>,
    #[serde(rename(deserialize = "addr:housenumber"))]
    pub housenumber: Option<String>,
    #[serde(rename(deserialize = "addr:postcode"))]
    pub postcode: Option<String>,
    #[serde(rename(deserialize = "addr:place"))]
    pub place: Option<String>,
    #[serde(rename(deserialize = "addr:housename"))]
    pub housename: Option<String>,
    #[serde(rename(deserialize = "addr:conscriptionnumber"))]
    pub conscriptionnumber: Option<String>,
    #[serde(rename(deserialize = "addr:flats"))]
    pub flats: Option<String>,
    #[serde(rename(deserialize = "addr:floor"))]
    pub floor: Option<String>,
    #[serde(rename(deserialize = "addr:door"))]
    pub door: Option<String>,
    #[serde(rename(deserialize = "addr:unit"))]
    pub unit: Option<String>,
    #[serde(rename(deserialize = "addr:city"))]
    pub city: Option<String>,
    // endregion housenumbers
    pub fixme: Option<String>,
}

/// OverpassElement represents one result from Overpass.
#[derive(serde::Deserialize)]
pub struct OverpassElement {
    pub id: u64,
    #[serde(rename(deserialize = "type"))]
    pub osm_type: String,
    pub user: Option<String>,
    pub timestamp: Option<String>,
    pub tags: OverpassTags,
}

#[derive(serde::Deserialize)]
pub struct OverpassTimes {
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp_osm_base: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp_areas_base: time::OffsetDateTime,
}

/// OverpassResult is the result from Overpass.
#[derive(serde::Deserialize)]
pub struct OverpassResult {
    pub osm3s: OverpassTimes,
    pub elements: Vec<OverpassElement>,
}
