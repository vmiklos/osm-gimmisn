/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The wsgi_additional module contains functionality for additional streets.

use crate::areas;
use crate::context;
use crate::i18n::translate as tr;
use crate::overpass_query;
use crate::util;
use crate::webframe;
use crate::yattag;
use anyhow::Context;

/// OverpassMember represents one member of a relation's members.
#[derive(serde::Deserialize)]
struct OverpassMember {
    #[serde(rename = "ref")]
    ref_id: u64,
}

/// OverpassElement represents one result from Overpass.
#[derive(serde::Deserialize)]
struct OverpassElement {
    id: u64,
    nodes: Option<Vec<u64>>,
    members: Option<Vec<OverpassMember>>,
    lat: Option<f64>,
    lon: Option<f64>,
}

/// OverpassResult is the result from Overpass.
#[derive(serde::Deserialize)]
struct OverpassResult {
    elements: Vec<OverpassElement>,
}

fn get_gpx_street_lat_lon(
    overpass: &OverpassResult,
    element: &OverpassElement,
) -> anyhow::Result<(String, String)> {
    if let Some(ref members) = element.members {
        let member = &members[0];
        let way = overpass
            .elements
            .iter()
            .find(|i| i.id == member.ref_id)
            .unwrap();
        get_gpx_street_lat_lon(overpass, way)
    } else if let Some(ref nodes) = element.nodes {
        let node_id = nodes.clone()[0];
        let node = overpass.elements.iter().find(|i| i.id == node_id).unwrap();
        let lat = node.lat.unwrap().to_string();
        let lon = node.lon.unwrap().to_string();
        Ok((lat, lon))
    } else {
        let lat = element.lat.context("missing lat")?.to_string();
        let lon = element.lon.context("missing lon")?.to_string();
        Ok((lat, lon))
    }
}

/// Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.gpx.
pub fn additional_streets_view_gpx(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("next_back() failed")?;
    let relation = relations
        .get_relation(relation_name)
        .context("get_relation() failed")?;
    let mut streets = relation.get_additional_streets(/*sorted_result=*/ true)?;
    let query = areas::make_turbo_query_for_street_objs(&relation, &streets);
    let buf = overpass_query::overpass_query(ctx, &query)?;
    let overpass: OverpassResult =
        serde_json::from_str(&buf).context(format!("failed to parse '{buf}' as json"))?;

    let doc = yattag::Doc::new();
    doc.append_value("<?xml version='1.0' encoding='UTF-8'?>".into());
    {
        let gpx = doc.tag(
            "gpx",
            &[
                ("version", "1.1"),
                ("creator", "osm-gimmisn"),
                ("xmlns", "http://www.topografix.com/GPX/1/1"),
                ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
                (
                    "xsi:schemaLocation",
                    "http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd",
                ),
            ],
        );
        {
            let metadata = gpx.tag("metadata", &[]);
            {
                let desc = metadata.tag("desc", &[]);
                desc.text(relation_name);
            }
            {
                let time = metadata.tag("time", &[]);
                let now = ctx.get_time().now();
                time.text(&now.format(&time::format_description::well_known::Rfc3339)?);
            }
        }
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()));
        for street in streets {
            let overpass_element = overpass
                .elements
                .iter()
                .find(|i| i.id == street.get_osm_id())
                .unwrap();
            let (lat, lon) = get_gpx_street_lat_lon(&overpass, overpass_element)
                .context("get_gpx_street_lat_lon() failed")?;
            let wpt = gpx.tag("wpt", &[("lat", &lat), ("lon", &lon)]);
            let name = wpt.tag("name", &[]);
            name.text(street.get_osm_name());
        }
    }
    let output = doc.get_value();
    Ok((output, relation_name.into()))
}
/// Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt.
pub fn additional_streets_view_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
    chkl: bool,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("next_back() failed")?;
    let relation = relations
        .get_relation(relation_name)
        .context("get_relation() failed")?;

    let output: String;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path())
    {
        output = tr("No reference streets");
    } else {
        let mut streets = relation.get_additional_streets(/*sorted_result=*/ true)?;
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()));
        let mut lines: Vec<String> = Vec::new();
        for street in streets {
            if chkl {
                lines.push(format!("[ ] {}\n", street.get_osm_name()));
            } else {
                lines.push(format!("{}\n", street.get_osm_name()));
            }
        }
        output = lines.join("");
    }
    Ok((output, relation_name.into()))
}

/// Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result.
pub fn additional_streets_view_result(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("next_back() failed")?;
    let relation = relations.get_relation(relation_name)?;

    let doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix();
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        doc.append_value(webframe::handle_no_osm_streets(&prefix, relation_name).get_value());
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path())
    {
        doc.append_value(webframe::handle_no_ref_streets(&prefix, relation_name).get_value());
    } else {
        // Get "only in OSM" streets.
        let mut streets = relation.write_additional_streets()?;
        let count = streets.len();
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()));
        let mut table = vec![vec![
            yattag::Doc::from_text(&tr("Identifier")),
            yattag::Doc::from_text(&tr("Type")),
            yattag::Doc::from_text(&tr("Source")),
            yattag::Doc::from_text(&tr("Street name")),
        ]];
        for street in streets {
            let cell = yattag::Doc::new();
            let href = format!(
                "https://www.openstreetmap.org/{}/{}",
                street.get_osm_type(),
                street.get_osm_id()
            );
            {
                let a = cell.tag("a", &[("href", &href), ("target", "_blank")]);
                a.text(&street.get_osm_id().to_string());
            }
            let cells = vec![
                cell,
                yattag::Doc::from_text(street.get_osm_type()),
                yattag::Doc::from_text(street.get_source()),
                yattag::Doc::from_text(street.get_osm_name()),
            ];
            table.push(cells);
        }

        {
            let p = doc.tag("p", &[]);
            p.text(
                &tr("OpenStreetMap additionally has the below {0} streets.")
                    .replace("{0}", &count.to_string()),
            );
            p.stag("br", &[]);
            {
                let a = p.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{prefix}/additional-streets/{relation_name}/view-result.txt"),
                    )],
                );
                a.text(&tr("Plain text format"));
            }
            p.stag("br", &[]);
            {
                let a = p.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{prefix}/additional-streets/{relation_name}/view-result.chkl"),
                    )],
                );
                a.text(&tr("Checklist format"));
            }
            p.stag("br", &[]);
            {
                let a = p.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{prefix}/additional-streets/{relation_name}/view-result.gpx"),
                    )],
                );
                a.text(&tr("GPX format"));
            }
            p.stag("br", &[]);
            {
                let a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{prefix}/additional-streets/{relation_name}/view-turbo"),
                    )],
                );
                a.text(&tr("Overpass turbo query for the below streets"));
            }
        }

        doc.append_value(util::html_table_from_list(&table).get_value());
        let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;
        doc.append_value(
            util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value(),
        );
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/budapest_11/view-result.
pub fn additional_housenumbers_view_result(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("next_back() failed")?;
    let mut relation = relations.get_relation(relation_name)?;

    let doc: yattag::Doc;
    let prefix = ctx.get_ini().get_uri_prefix();
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        doc = webframe::handle_no_osm_streets(&prefix, relation_name);
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path())
    {
        doc = webframe::handle_no_osm_housenumbers(&prefix, relation_name);
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path())
    {
        doc = webframe::handle_no_ref_housenumbers(&prefix, relation_name);
    } else {
        doc = additional_housenumbers_view_result_html(&mut relation)?;
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo.
pub fn additional_streets_view_turbo(
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("next_back() failed")?;
    let relation = relations.get_relation(relation_name)?;

    let doc = yattag::Doc::new();
    let streets = relation.get_additional_streets(/*sorted_result=*/ false)?;
    let query = areas::make_turbo_query_for_street_objs(&relation, &streets);

    let pre = doc.tag("pre", &[]);
    pre.text(&query);
    Ok(doc)
}

/// The actual HTML part of additional_housenumbers_view_result().
fn additional_housenumbers_view_result_html(
    relation: &mut areas::Relation<'_>,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let (todo_street_count, todo_count, table) = relation.write_additional_housenumbers()?;

    {
        let p = doc.tag("p", &[]);
        p.text(
            &tr("OpenStreetMap additionally has the below {0} house numbers for {1} streets.")
                .replace("{0}", &todo_count.to_string())
                .replace("{1}", &todo_street_count.to_string()),
        );
        doc.stag("br");
        let a = doc.tag(
            "a",
            &[(
                "href",
                &tr(
                    "https://vmiklos.hu/osm-gimmisn/usage.html#filtering-out-incorrect-information",
                ),
            )],
        );
        a.text(&tr("Filter incorrect information"));
    }

    doc.append_value(util::html_table_from_list(&table).get_value());
    let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;
    doc.append_value(util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value());
    doc.append_value(
        util::invalid_filter_keys_to_html(&relation.get_invalid_filter_keys()?).get_value(),
    );

    Ok(doc)
}

#[cfg(test)]
mod tests;
