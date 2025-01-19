/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Contains functionality specific to the json part of the web interface.

use crate::areas;
use crate::cache;
use crate::context;
use crate::overpass_query;
use crate::webframe;
use anyhow::Context;
use std::collections::HashMap;

/// Expected request_uri: e.g. /osm/streets/ormezo/update-result.json.
fn streets_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let relation = relations
        .get_relation(relation_name)
        .context("get_relation() failed")?;
    let mut ret: HashMap<String, String> = HashMap::new();
    let query = relation.get_osm_streets_json_query()?;
    match overpass_query::overpass_query(ctx, &query) {
        Ok(buf) => {
            relation.get_files().write_osm_json_streets(ctx, &buf)?;
            ret.insert("error".into(), "".into())
        }
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

/// Expected request_uri: e.g. /osm/street-housenumbers/ormezo/update-result.json.
fn street_housenumbers_update_result_json(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let relation = relations.get_relation(relation_name)?;
    let mut ret: HashMap<String, String> = HashMap::new();
    let query = relation.get_osm_housenumbers_json_query()?;
    match overpass_query::overpass_query(ctx, &query) {
        Ok(buf) => {
            relation
                .get_files()
                .write_osm_json_housenumbers(ctx, &buf)?;
            ret.insert("error".into(), "".into())
        }
        Err(err) => ret.insert("error".into(), err.to_string()),
    };
    Ok(serde_json::to_string(&ret)?)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.json.
fn missing_housenumbers_view_result_json(
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let mut relation = relations.get_relation(relation_name)?;
    cache::get_missing_housenumbers_json(&mut relation)
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-result.json.
fn additional_housenumbers_view_result_json(
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let mut relation = relations.get_relation(relation_name)?;
    cache::get_additional_housenumbers_json(&mut relation)
}

/// Dispatches json requests based on their URIs.
pub fn our_application_json(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<rouille::Response> {
    let mut headers: webframe::Headers = Vec::new();
    let prefix = ctx.get_ini().get_uri_prefix();
    let output: String;
    if request_uri.starts_with(&format!("{prefix}/streets/")) {
        output = streets_update_result_json(ctx, relations, request_uri)?;
    } else if request_uri.starts_with(&format!("{prefix}/street-housenumbers/")) {
        output = street_housenumbers_update_result_json(ctx, relations, request_uri)?;
    } else if request_uri.starts_with(&format!("{prefix}/missing-housenumbers/")) {
        // Assume request_uri ends with view-result.json.
        output = missing_housenumbers_view_result_json(relations, request_uri)?;
    } else if request_uri
        == format!("{prefix}/lints/whole-country/invalid-addr-cities/update-result.json")
    {
        output = webframe::handle_invalid_addr_cities_update_json(ctx)?;
    } else if request_uri == format!("{prefix}/api/relations.json") {
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select json from stats_jsons where category = 'relations'")?;
        let mut rows = stmt.query([])?;
        output = match rows.next()? {
            Some(row) => row.get(0)?,
            None => String::from("[]"),
        };
    } else {
        // Assume /additional-housenumbers/<relation>/view-result.json.
        output = additional_housenumbers_view_result_json(relations, request_uri)?;
    }
    let output_bytes = output.as_bytes().to_vec();
    headers.push((
        "Content-type".into(),
        "application/json; charset=utf-8".into(),
    ));
    Ok(webframe::make_response(200_u16, headers, output_bytes))
}

#[cfg(test)]
mod tests;
