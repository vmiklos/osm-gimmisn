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
    match overpass_query::overpass_query_with_retry(ctx, &query) {
        Ok(buf) => {
            relation
                .get_files()
                .write_osm_json_housenumbers(ctx, &buf)?;
            ret.insert("error".into(), "".into());
        }
        Err(err) => {
            ret.insert("error".into(), err.to_string());
        }
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

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/geojson.json.
fn missing_housenumbers_geojson(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("short tokens")?;
    let mut relation = relations.get_relation(relation_name)?;
    let ongoing_streets = relation.get_missing_housenumbers()?.ongoing_streets;
    let mut streets: Vec<String> = Vec::new();
    for result in ongoing_streets {
        streets.push(result.street.get_osm_name().into());
    }
    let query = areas::make_query_for_streets(&relation, &streets);
    let overpass = overpass_query::overpass_query_with_retry(ctx, &query)?;
    overpass_to_geojson(&overpass)
}

/// Turns one overpass element into a geojson Feature, with the given resolved geometry.
fn make_geojson_feature(
    kind: &str,
    id: i64,
    tags: &serde_json::Value,
    geometry: serde_json::Value,
) -> serde_json::Value {
    let full_id = format!("{kind}/{id}");
    // properties is the tags, prefixed with an "@id" pointing to the source object.
    let mut properties = serde_json::Map::new();
    properties.insert("@id".into(), serde_json::json!(full_id));
    if let Some(object) = tags.as_object() {
        for (key, value) in object {
            properties.insert(key.clone(), value.clone());
        }
    }
    serde_json::json!({
        "type": "Feature",
        "properties": serde_json::Value::Object(properties),
        "geometry": geometry,
        "id": full_id,
    })
}

/// Converts an overpass JSON response into a geojson FeatureCollection.
fn overpass_to_geojson(overpass: &str) -> anyhow::Result<String> {
    let root: serde_json::Value = serde_json::from_str(overpass)?;
    let elements = root
        .get("elements")
        .and_then(|i| i.as_array())
        .context("no elements array in overpass response")?;

    // Collect node id -> [lon, lat], so way geometries can be resolved below.
    let mut nodes: HashMap<i64, serde_json::Value> = HashMap::new();
    for element in elements {
        if element.get("type").and_then(|i| i.as_str()) != Some("node") {
            continue;
        }
        let (Some(id), Some(lat), Some(lon)) = (
            element.get("id").and_then(|i| i.as_i64()),
            element.get("lat").and_then(|i| i.as_f64()),
            element.get("lon").and_then(|i| i.as_f64()),
        ) else {
            continue;
        };
        nodes.insert(id, serde_json::json!([lon, lat]));
    }

    let mut features: Vec<serde_json::Value> = Vec::new();
    for element in elements {
        let element_type = element.get("type").and_then(|i| i.as_str()).unwrap_or("");
        let Some(id) = element.get("id").and_then(|i| i.as_i64()) else {
            continue;
        };
        let tags = element
            .get("tags")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let has_tags = tags.as_object().map(|i| !i.is_empty()).unwrap_or(false);
        match element_type {
            "node" => {
                // Untagged nodes only provide geometry for ways, they are not features.
                if !has_tags {
                    continue;
                }
                let Some(coordinates) = nodes.get(&id) else {
                    continue;
                };
                let geometry = serde_json::json!({
                    "type": "Point",
                    "coordinates": coordinates.clone(),
                });
                features.push(make_geojson_feature("node", id, &tags, geometry));
            }
            "way" => {
                // Untagged ways are just relation members / geometry fragments, skip them.
                if !has_tags {
                    continue;
                }
                let Some(way_nodes) = element.get("nodes").and_then(|i| i.as_array()) else {
                    continue;
                };
                let coordinates: Vec<serde_json::Value> = way_nodes
                    .iter()
                    .filter_map(|node| node.as_i64())
                    .filter_map(|node_id| nodes.get(&node_id).cloned())
                    .collect();
                let geometry = serde_json::json!({
                    "type": "LineString",
                    "coordinates": coordinates,
                });
                features.push(make_geojson_feature("way", id, &tags, geometry));
            }
            // Relations are not converted to geometry.
            _ => {}
        }
    }

    let osm3s = root.get("osm3s");
    let field = |name: &str| -> serde_json::Value {
        osm3s
            .and_then(|i| i.get(name))
            .cloned()
            .unwrap_or(serde_json::Value::Null)
    };
    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "copyright": field("copyright"),
        "timestamp": field("timestamp_osm_base"),
        "features": features,
    });
    Ok(serde_json::to_string(&geojson)?)
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
        if request_uri.ends_with("/geojson.json") {
            output = missing_housenumbers_geojson(ctx, relations, request_uri)?;
            // Allow tools like geojson.io to fetch this from their own origin.
            headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
        } else {
            // Assume request_uri ends with view-result.json.
            output = missing_housenumbers_view_result_json(relations, request_uri)?;
        }
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
