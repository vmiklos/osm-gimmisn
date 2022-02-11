/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The wsgi_additional module contains functionality for additional streets.

use crate::areas;
use crate::cache;
use crate::context;
use crate::i18n::translate as tr;
use crate::util;
use crate::webframe;
use crate::yattag;
use anyhow::Context;

/// Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt.
pub fn additional_streets_view_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
    chkl: bool,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
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
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()).unwrap());
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
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations.get_relation(relation_name)?;

    let doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix()?;
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
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()).unwrap());
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
                        &format!(
                            "{}/additional-streets/{}/view-result.txt",
                            prefix, relation_name
                        ),
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
                        &format!(
                            "{}/additional-streets/{}/view-result.chkl",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Checklist format"));
            }
            p.stag("br", &[]);
            {
                let a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/additional-streets/{}/view-turbo", prefix, relation_name),
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
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let mut relation = relations.get_relation(relation_name)?;

    let doc: yattag::Doc;
    let prefix = ctx.get_ini().get_uri_prefix()?;
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
        doc = cache::get_additional_housenumbers_html(ctx, &mut relation)?;
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo.
pub fn additional_streets_view_turbo(
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations.get_relation(relation_name)?;

    let doc = yattag::Doc::new();
    let streets = relation.get_additional_streets(/*sorted_result=*/ false)?;
    let query = areas::make_turbo_query_for_street_objs(&relation, &streets);

    let pre = doc.tag("pre", &[]);
    pre.text(&query);
    Ok(doc)
}

#[cfg(test)]
mod tests;
