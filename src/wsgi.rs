/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The wsgi module contains functionality specific to the web interface

use crate::areas;
use crate::cache;
use crate::context;
use crate::i18n::translate as tr;
use crate::overpass_query;
use crate::util;
use crate::webframe;
use crate::wsgi_additional;
use crate::yattag;
use anyhow::Context;
use pyo3::prelude::*;
use std::ops::DerefMut;
use std::sync::Arc;

/// Gets the update date string of a file.
fn get_last_modified(path: &str) -> String {
    webframe::format_timestamp(util::get_timestamp(path) as i64)
}

/// Gets the update date of streets for a relation.
fn get_streets_last_modified(relation: &areas::Relation) -> anyhow::Result<String> {
    Ok(get_last_modified(
        &relation.get_files().get_osm_streets_path()?,
    ))
}

/// Expected request_uri: e.g. /osm/streets/ormezo/view-query.
fn handle_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().unwrap();
    let relation_name = tokens.next_back().unwrap();

    let relation = relations.get_relation(relation_name)?;
    let osmrelation = relation.get_config().get_osmrelation();

    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            "streets",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    if action == "view-query" {
        let _pre = doc.tag("pre", &[]);
        doc.text(&relation.get_osm_streets_query()?);
    } else if action == "update-result" {
        let query = relation.get_osm_streets_query()?;
        match overpass_query::overpass_query(ctx, query) {
            Ok(buf) => {
                relation.get_files().write_osm_streets(ctx, &buf)?;
                let streets = relation.get_config().should_check_missing_streets();
                if streets != "only" {
                    doc.text(&tr("Update successful: "));
                    let prefix = ctx.get_ini().get_uri_prefix()?;
                    let link = format!(
                        "{}/missing-housenumbers/{}/view-result",
                        prefix, relation_name
                    );
                    doc.append_value(
                        util::gen_link(&link, &tr("View missing house numbers")).get_value(),
                    );
                } else {
                    doc.text(&tr("Update successful."));
                }
            }
            Err(err) => {
                doc.append_value(util::handle_overpass_error(ctx, &err.to_string()).get_value());
            }
        }
    } else {
        // assume view-result
        let stream = relation.get_files().get_osm_streets_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut read = guard.deref_mut();
        let mut csv_read = util::CsvRead::new(&mut read);
        let table = util::tsv_to_list(&mut csv_read)?;
        doc.append_value(util::html_table_from_list(&table).get_value());
    }

    doc.append_value(webframe::get_footer(&get_streets_last_modified(&relation)?).get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_streets(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match handle_streets(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_streets() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Gets the update date of house numbers for a relation.
fn get_housenumbers_last_modified(relation: &areas::Relation) -> anyhow::Result<String> {
    Ok(get_last_modified(
        &relation.get_files().get_osm_housenumbers_path()?,
    ))
}

/// Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query.
fn handle_street_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().unwrap();
    let relation_name = tokens.next_back().unwrap();

    let relation = relations.get_relation(relation_name)?;
    let osmrelation = relation.get_config().get_osmrelation();

    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            "street-housenumbers",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    let prefix = ctx.get_ini().get_uri_prefix()?;
    if action == "view-query" {
        let _pre = doc.tag("pre", &[]);
        doc.text(&relation.get_osm_housenumbers_query()?);
    } else if action == "update-result" {
        let query = relation.get_osm_housenumbers_query()?;
        match overpass_query::overpass_query(ctx, query) {
            Ok(buf) => {
                relation.get_files().write_osm_housenumbers(ctx, &buf)?;
                doc.text(&tr("Update successful: "));
                let link = format!(
                    "{}/missing-housenumbers/{}/view-result",
                    prefix, relation_name
                );
                doc.append_value(
                    util::gen_link(&link, &tr("View missing house numbers")).get_value(),
                );
            }
            Err(err) => {
                doc.append_value(util::handle_overpass_error(ctx, &err.to_string()).get_value());
            }
        }
    } else {
        // assume view-result
        if !ctx
            .get_file_system()
            .path_exists(&relation.get_files().get_osm_housenumbers_path()?)
        {
            let _div = doc.tag("div", &[("id", "no-osm-housenumbers")]);
            doc.text(&tr("No existing house numbers"));
        } else {
            let stream = relation.get_files().get_osm_housenumbers_read_stream(ctx)?;
            let mut guard = stream.lock().unwrap();
            let mut read = guard.deref_mut();
            let mut csv_read = util::CsvRead::new(&mut read);
            doc.append_value(
                util::html_table_from_list(&util::tsv_to_list(&mut csv_read)?).get_value(),
            );
        }
    }

    let date = get_housenumbers_last_modified(&relation)?;
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_street_housenumbers(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match handle_street_housenumbers(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_street_housenumbers() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-turbo.
fn missing_housenumbers_view_turbo(
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();

    let doc = yattag::Doc::new();
    let mut relation = relations.get_relation(relation_name)?;
    let (ongoing_streets, _done_streets) = relation.get_missing_housenumbers()?;
    let mut streets: Vec<String> = Vec::new();
    for result in ongoing_streets {
        // Street name, # of only_in_reference items.
        streets.push(result.0.get_osm_name().into());
    }
    let query = areas::make_turbo_query_for_streets(&relation, &streets);

    let _pre = doc.tag("pre", &[]);
    doc.text(&query);
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.
fn missing_housenumbers_view_res(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();

    let doc: yattag::Doc;
    let mut relation = relations.get_relation(relation_name)?;
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path()?)
    {
        doc = webframe::handle_no_osm_streets(&prefix, relation_name);
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path()?)
    {
        doc = webframe::handle_no_osm_housenumbers(&prefix, relation_name);
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path()?)
    {
        doc = webframe::handle_no_ref_housenumbers(&prefix, relation_name);
    } else {
        doc = cache::get_missing_housenumbers_html(ctx, &mut relation)?;
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-streets/budapest_11/view-result.
fn missing_streets_view_result(
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
        .path_exists(&relation.get_files().get_osm_streets_path()?)
    {
        doc.append_value(webframe::handle_no_osm_streets(&prefix, relation_name).get_value());
        return Ok(doc);
    }

    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path()?)
    {
        doc.append_value(webframe::handle_no_ref_streets(&prefix, relation_name).get_value());
        return Ok(doc);
    }

    let (todo_count, done_count, percent, mut streets) = relation.write_missing_streets()?;
    streets.sort_by_key(|i| util::get_sort_key(i).unwrap());
    let mut table = vec![vec![yattag::Doc::from_text(&tr("Street name"))]];
    for street in streets {
        table.push(vec![yattag::Doc::from_text(&street)]);
    }

    {
        let _p = doc.tag("p", &[]);
        doc.text(
            &tr("OpenStreetMap is possibly missing the below {0} streets.")
                .replace("{0}", &todo_count.to_string()),
        );
        doc.text(
            &tr(" (existing: {0}, ready: {1}).")
                .replace("{0}", &done_count.to_string())
                .replace("{1}", &util::format_percent(&percent)?),
        );
        doc.stag("br", &[]);
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("{}/missing-streets/{}/view-turbo", prefix, relation_name),
                )],
            );
            doc.text(&tr(
                "Overpass turbo query for streets with questionable names",
            ));
        }
        doc.stag("br", &[]);
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-streets/{}/view-result.txt",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("Plain text format"));
        }
        doc.stag("br", &[]);
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-streets/{}/view-result.chkl",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("Checklist format"));
        }
    }

    doc.append_value(util::html_table_from_list(&table).get_value());
    let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;
    doc.append_value(util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value());
    doc.append_value(
        util::invalid_filter_keys_to_html(&relation.get_invalid_filter_keys()?).get_value(),
    );
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.txt.
fn missing_housenumbers_view_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let mut relation = relations.get_relation(relation_name)?;
    let mut config = relation.get_config().clone();
    config.set_letter_suffix_style(util::LetterSuffixStyle::Lower as i32);
    relation.set_config(&config);

    let output: String;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path()?)
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path()?)
    {
        output = tr("No existing house numbers");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path()?)
    {
        output = tr("No reference house numbers");
    } else {
        output = cache::get_missing_housenumbers_txt(ctx, &mut relation)?;
    }
    Ok(output)
}

#[pyfunction]
fn py_missing_housenumbers_view_txt(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<String> {
    match missing_housenumbers_view_txt(&ctx.context, &mut relations.relations, request_uri)
        .context("missing_housenumbers_view_txt() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.chkl.
fn missing_housenumbers_view_chkl(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let mut relation = relations.get_relation(relation_name)?;
    let mut config = relation.get_config().clone();
    config.set_letter_suffix_style(util::LetterSuffixStyle::Lower as i32);
    relation.set_config(&config);

    let output: String;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path()?)
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path()?)
    {
        output = tr("No existing house numbers");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path()?)
    {
        output = tr("No reference house numbers");
    } else {
        let (ongoing_streets, _) = relation.get_missing_housenumbers()?;

        let mut table: Vec<String> = Vec::new();
        for result in ongoing_streets {
            let range_list = util::get_housenumber_ranges(&result.1);
            // Street name, only_in_reference items.
            if !relation
                .get_config()
                .get_street_is_even_odd(result.0.get_osm_name())
            {
                let mut result_sorted: Vec<String> =
                    range_list.iter().map(|i| i.get_number().into()).collect();
                result_sorted.sort_by_key(|i| util::split_house_number(i));
                let row = format!(
                    "[ ] {} [{}]",
                    result.0.get_osm_name(),
                    result_sorted.join(", ")
                );
                table.push(row);
            } else {
                let elements = util::format_even_odd(&range_list);
                if elements.len() > 1 && range_list.len() > 20 {
                    for element in elements {
                        let row = format!("[ ] {} [{}]", result.0.get_osm_name(), element);
                        table.push(row);
                    }
                } else {
                    let row = format!(
                        "[ ] {} [{}]",
                        result.0.get_osm_name(),
                        elements.join("], [")
                    );
                    table.push(row);
                }
            }
        }
        table.sort_by_key(|i| util::get_sort_key(i).unwrap());
        output = table.join("\n");
    }
    Ok((output, relation_name.into()))
}

#[pyfunction]
fn py_missing_housenumbers_view_chkl(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<(String, String)> {
    match missing_housenumbers_view_chkl(&ctx.context, &mut relations.relations, request_uri)
        .context("missing_housenumbers_view_chkl() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-result.txt.
fn missing_streets_view_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
    chkl: bool,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations.get_relation(relation_name)?;

    let output: String;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path()?)
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path()?)
    {
        output = tr("No reference streets");
    } else {
        let (mut todo_streets, _) = relation.get_missing_streets()?;
        todo_streets.sort_by_key(|i| util::get_sort_key(i).unwrap());
        let mut lines: Vec<String> = Vec::new();
        for street in todo_streets {
            if chkl {
                lines.push(format!("[ ] {}\n", street));
            } else {
                lines.push(format!("{}\n", street));
            }
        }
        output = lines.join("");
    }
    Ok((output, relation_name.into()))
}

#[pyfunction]
fn py_missing_streets_view_txt(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
    chkl: bool,
) -> PyResult<(String, String)> {
    match missing_streets_view_txt(&ctx.context, &mut relations.relations, request_uri, chkl)
        .context("missing_streets_view_txt() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/update-result.
fn missing_housenumbers_update(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    relation_name: &str,
) -> anyhow::Result<yattag::Doc> {
    let references = ctx.get_ini().get_reference_housenumber_paths()?;
    let relation = relations.get_relation(relation_name)?;
    relation.write_ref_housenumbers(&references)?;
    let doc = yattag::Doc::new();
    doc.text(&tr("Update successful: "));
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let link = format!(
        "{}/missing-housenumbers/{}/view-result",
        prefix, relation_name
    );
    doc.append_value(util::gen_link(&link, &tr("View missing house numbers")).get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-streets/ujbuda/update-result.
fn missing_streets_update(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    relation_name: &str,
) -> anyhow::Result<yattag::Doc> {
    let relation = relations.get_relation(relation_name)?;
    relation.write_ref_streets(&ctx.get_ini().get_reference_street_path()?)?;
    let doc = yattag::Doc::new();
    let _div = doc.tag("div", &[("id", "update-success")]);
    doc.text(&tr("Update successful."));
    Ok(doc)
}

/// Gets the update date for missing house numbers.
fn ref_housenumbers_last_modified(
    relations: &mut areas::Relations,
    name: &str,
) -> anyhow::Result<String> {
    let relation = relations.get_relation(name)?;
    let t_ref = util::get_timestamp(&relation.get_files().get_ref_housenumbers_path()?);
    let t_housenumbers = util::get_timestamp(&relation.get_files().get_osm_housenumbers_path()?);
    Ok(webframe::format_timestamp(std::cmp::max(
        t_ref as i64,
        t_housenumbers as i64,
    )))
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-[result|query].
fn handle_missing_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().unwrap();
    let relation_name = tokens.next_back().unwrap();
    let mut date = "".into();

    let relation = relations.get_relation(relation_name)?;
    let osmrelation = relation.get_config().get_osmrelation();
    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            "missing-housenumbers",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    if action == "view-turbo" {
        doc.append_value(missing_housenumbers_view_turbo(relations, request_uri)?.get_value());
    } else if action == "view-query" {
        {
            let _pre = doc.tag("pre", &[]);
            let stream = relation.get_files().get_ref_housenumbers_read_stream(ctx)?;
            let mut guard = stream.lock().unwrap();
            let mut buffer: Vec<u8> = Vec::new();
            guard.read_to_end(&mut buffer)?;
            doc.text(&String::from_utf8(buffer)?);
        }
        date = get_last_modified(&relation.get_files().get_ref_housenumbers_path()?);
    } else if action == "update-result" {
        doc.append_value(missing_housenumbers_update(ctx, relations, relation_name)?.get_value())
    } else {
        // assume view-result
        doc.append_value(missing_housenumbers_view_res(ctx, relations, request_uri)?.get_value())
    }

    if date.is_empty() {
        date = ref_housenumbers_last_modified(relations, relation_name)?;
    }
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_missing_housenumbers(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match handle_missing_housenumbers(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_missing_housenumbers() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/missing-streets/ormezo/view-turbo.
fn missing_streets_view_turbo(
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();

    let doc = yattag::Doc::new();
    let relation = relations.get_relation(relation_name)?;
    let refstreets = relation.get_config().get_refstreets();
    let mut streets: Vec<String> = Vec::new();
    for (key, _value) in refstreets {
        if relation.should_show_ref_street(&key) {
            streets.push(key)
        }
    }
    let query = areas::make_turbo_query_for_streets(&relation, &streets);

    let _pre = doc.tag("pre", &[]);
    doc.text(&query);
    Ok(doc)
}

/// Gets the update date for missing/additional streets.
fn streets_diff_last_modified(relation: &areas::Relation) -> anyhow::Result<String> {
    let t_ref = util::get_timestamp(&relation.get_files().get_ref_streets_path()?) as i64;
    let t_osm = util::get_timestamp(&relation.get_files().get_osm_streets_path()?) as i64;
    Ok(webframe::format_timestamp(std::cmp::max(t_ref, t_osm)))
}

/// Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-[result|query].
fn handle_missing_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().unwrap();
    let relation_name = tokens.next_back().unwrap();

    let relation = relations.get_relation(relation_name)?;
    let osmrelation = relation.get_config().get_osmrelation();

    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            "missing-streets",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    if action == "view-turbo" {
        doc.append_value(missing_streets_view_turbo(relations, request_uri)?.get_value());
    } else if action == "view-query" {
        let _pre = doc.tag("pre", &[]);
        let stream = relation.get_files().get_ref_streets_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        doc.text(&String::from_utf8(buffer)?);
    } else if action == "update-result" {
        doc.append_value(missing_streets_update(ctx, relations, relation_name)?.get_value());
    } else {
        // assume view-result
        doc.append_value(missing_streets_view_result(ctx, relations, request_uri)?.get_value());
    }

    let date = streets_diff_last_modified(&relation)?;
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_missing_streets(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match handle_missing_streets(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_missing_streets() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-[result|query].
fn handle_additional_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().unwrap();
    let relation_name = tokens.next_back().unwrap();

    let relation = relations.get_relation(relation_name)?;
    let osmrelation = relation.get_config().get_osmrelation();

    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            "additional-streets",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    if action == "view-turbo" {
        doc.append_value(
            wsgi_additional::additional_streets_view_turbo(relations, request_uri)?.get_value(),
        )
    } else {
        // assume view-result
        doc.append_value(
            wsgi_additional::additional_streets_view_result(ctx, relations, request_uri)?
                .get_value(),
        )
    }

    let date = streets_diff_last_modified(&relation)?;
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_additional_streets(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match handle_additional_streets(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_additional_streets() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Gets the update date for missing/additional housenumbers.
fn housenumbers_diff_last_modified(relation: &areas::Relation) -> anyhow::Result<String> {
    let t_ref = util::get_timestamp(&relation.get_files().get_ref_housenumbers_path()?) as i64;
    let t_osm = util::get_timestamp(&relation.get_files().get_osm_housenumbers_path()?) as i64;
    Ok(webframe::format_timestamp(std::cmp::max(t_ref, t_osm)))
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/ujbuda/view-[result|query].
fn handle_additional_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let _action = tokens.next_back();
    let relation_name = tokens.next_back().unwrap();

    let relation = relations.get_relation(relation_name)?;
    let osmrelation = relation.get_config().get_osmrelation();

    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            "additional-housenumbers",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    // assume action is view-result
    doc.append_value(
        wsgi_additional::additional_housenumbers_view_result(ctx, relations, request_uri)?
            .get_value(),
    );

    let date = housenumbers_diff_last_modified(&relation)?;
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_additional_housenumbers(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match handle_additional_housenumbers(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_additional_housenumbers() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Handles the house number percent part of the main page.
fn handle_main_housenr_percent(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<(yattag::Doc, String)> {
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let url = format!(
        "{}/missing-housenumbers/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut percent: String = "N/A".into();
    if ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_housenumbers_percent_path()?)
    {
        let stream = relation
            .get_files()
            .get_housenumbers_percent_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        percent = String::from_utf8(buffer)?;
    }

    let doc = yattag::Doc::new();
    if percent != "N/A" {
        let date = get_last_modified(&relation.get_files().get_housenumbers_percent_path()?);
        let _strong = doc.tag("strong", &[]);
        let _a = doc.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        doc.text(&util::format_percent(&percent)?);
        return Ok((doc, percent));
    }

    let _strong = doc.tag("strong", &[]);
    let _a = doc.tag("a", &[("href", &url)]);
    doc.text(&tr("missing house numbers"));
    Ok((doc, "0".into()))
}

/// Handles the street percent part of the main page.
fn handle_main_street_percent(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<(yattag::Doc, String)> {
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let url = format!(
        "{}/missing-streets/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut percent: String = "N/A".into();
    if ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_streets_percent_path()?)
    {
        let stream = relation.get_files().get_streets_percent_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        percent = String::from_utf8(buffer)?;
    }

    let doc = yattag::Doc::new();
    if percent != "N/A" {
        let date = get_last_modified(&relation.get_files().get_streets_percent_path()?);
        let _strong = doc.tag("strong", &[]);
        let _a = doc.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        doc.text(&util::format_percent(&percent)?);
        return Ok((doc, percent));
    }

    let _strong = doc.tag("strong", &[]);
    let _a = doc.tag("a", &[("href", &url)]);
    doc.text(&tr("missing streets"));
    Ok((doc, "0".into()))
}

/// Handles the street additional count part of the main page.
fn handle_main_street_additional_count(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<yattag::Doc> {
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let url = format!(
        "{}/additional-streets/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut additional_count: String = "".into();
    if ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_streets_additional_count_path()?)
    {
        let stream = relation
            .get_files()
            .get_streets_additional_count_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        additional_count = String::from_utf8(buffer)?;
    }

    let doc = yattag::Doc::new();
    if !additional_count.is_empty() {
        let date = get_last_modified(&relation.get_files().get_streets_additional_count_path()?);
        let _strong = doc.tag("strong", &[]);
        let _a = doc.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        doc.text(&tr("{} streets").replace("{}", &additional_count));
        return Ok(doc);
    }

    let _strong = doc.tag("strong", &[]);
    let _a = doc.tag("a", &[("href", &url)]);
    doc.text(&tr("additional streets"));
    Ok(doc)
}

/// Handles the housenumber additional count part of the main page.
fn handle_main_housenr_additional_count(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<yattag::Doc> {
    if !relation.get_config().should_check_additional_housenumbers() {
        return Ok(yattag::Doc::new());
    }

    let prefix = ctx.get_ini().get_uri_prefix()?;
    let url = format!(
        "{}/additional-housenumbers/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut additional_count: String = "".into();
    if ctx.get_file_system().path_exists(
        &relation
            .get_files()
            .get_housenumbers_additional_count_path()?,
    ) {
        let stream = relation
            .get_files()
            .get_housenumbers_additional_count_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        additional_count = String::from_utf8(buffer)?.trim().into();
    }

    let doc = yattag::Doc::new();
    if !additional_count.is_empty() {
        let date = get_last_modified(
            &relation
                .get_files()
                .get_housenumbers_additional_count_path()?,
        );
        let _strong = doc.tag("strong", &[]);
        let _a = doc.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        doc.text(&tr("{} house numbers").replace("{}", &additional_count));
        return Ok(doc);
    }

    let _strong = doc.tag("strong", &[]);
    let _a = doc.tag("a", &[("href", &url)]);
    doc.text(&tr("additional house numbers"));
    Ok(doc)
}

#[pyfunction]
fn py_handle_main_housenr_additional_count(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<yattag::PyDoc> {
    match handle_main_housenr_additional_count(&ctx.context, &relation.relation)
        .context("handle_main_housenr_additional_count() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Does not filter out anything.
fn filter_for_everything(_complete: bool, _relation: &areas::Relation) -> bool {
    true
}

/// Filters out complete items.
fn filter_for_incomplete(complete: bool, _relation: &areas::Relation) -> bool {
    !complete
}

type RelationFilter = dyn Fn(bool, &areas::Relation) -> bool;

/// Creates a function that filters for a single refcounty.
fn create_filter_for_refcounty(refcounty_filter: &str) -> Box<RelationFilter> {
    let refcounty_filter_arc = Arc::new(refcounty_filter.to_string());
    let refcounty_filter = refcounty_filter_arc;
    Box::new(move |_complete, relation| {
        relation.get_config().get_refcounty() == refcounty_filter.as_str()
    })
}

/// Creates a function that filters for the specified relations.
fn create_filter_for_relations(relation_filter: &str) -> Box<RelationFilter> {
    let mut relations: Vec<u64> = Vec::new();
    if !relation_filter.is_empty() {
        relations = relation_filter
            .split(',')
            .map(|i| i.parse().unwrap())
            .collect();
    }
    let relations_arc = Arc::new(relations);
    let relations = relations_arc;
    Box::new(move |_complete, relation| {
        relations.contains(&relation.get_config().get_osmrelation())
    })
}

/// Creates a function that filters for a single refsettlement in a refcounty.
fn create_filter_for_refcounty_refsettlement(
    refcounty_filter: &str,
    refsettlement_filter: &str,
) -> Box<RelationFilter> {
    let refcounty_arc = Arc::new(refcounty_filter.to_string());
    let refcounty_filter = refcounty_arc;
    let refsettlement_arc = Arc::new(refsettlement_filter.parse::<u64>().unwrap());
    let refsettlement_filter = refsettlement_arc;
    Box::new(move |_complete, relation| {
        let config = relation.get_config();
        config.get_refcounty() == refcounty_filter.as_str()
            && config.get_osmrelation().eq(refsettlement_filter.as_ref())
    })
}

/// Sets up a filter-for function from request uri: only certain areas are shown then.
fn setup_main_filter_for(request_uri: &str) -> (Box<RelationFilter>, String) {
    let tokens: Vec<String> = request_uri.split('/').map(|i| i.to_string()).collect();
    let mut filter_for: Box<RelationFilter> = Box::new(filter_for_incomplete);
    let filters = util::parse_filters(&tokens);
    let mut refcounty = "";
    if filters.contains_key("incomplete") {
        // /osm/filter-for/incomplete
        filter_for = Box::new(filter_for_incomplete);
    } else if filters.contains_key("everything") {
        // /osm/filter-for/everything
        filter_for = Box::new(filter_for_everything);
    } else if filters.contains_key("refcounty") && filters.contains_key("refsettlement") {
        // /osm/filter-for/refcounty/<value>/refsettlement/<value>
        refcounty = filters.get("refcounty").unwrap();
        filter_for = create_filter_for_refcounty_refsettlement(
            filters.get("refcounty").unwrap(),
            filters.get("refsettlement").unwrap(),
        );
    } else if filters.contains_key("refcounty") {
        // /osm/filter-for/refcounty/<value>/whole-county
        refcounty = filters.get("refcounty").unwrap();
        filter_for = create_filter_for_refcounty(refcounty);
    } else if filters.contains_key("relations") {
        // /osm/filter-for/relations/<id1>,<id2>
        let relations = filters.get("relations").unwrap();
        filter_for = create_filter_for_relations(relations);
    }
    (filter_for, refcounty.into())
}

/// Handles one refcounty in the filter part of the main wsgi page.
fn handle_main_filters_refcounty(
    ctx: &context::Context,
    relations: &areas::Relations,
    refcounty_id: &str,
    refcounty: &str,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let name = relations.refcounty_get_name(refcounty);
    if name.is_empty() {
        return Ok(doc);
    }

    let prefix = ctx.get_ini().get_uri_prefix()?;
    {
        let _a = doc.tag(
            "a",
            &[(
                "href",
                &format!("{}/filter-for/refcounty/{}/whole-county", prefix, refcounty),
            )],
        );
        doc.text(&name);
    }
    if !refcounty_id.is_empty() && refcounty == refcounty_id {
        let refsettlement_ids = relations.refcounty_get_refsettlement_ids(refcounty_id);
        if !refsettlement_ids.is_empty() {
            let mut names: Vec<yattag::Doc> = Vec::new();
            for refsettlement_id in refsettlement_ids {
                let name = relations.refsettlement_get_name(refcounty_id, &refsettlement_id);
                let name_doc = yattag::Doc::new();
                {
                    let href = format!(
                        "{}/filter-for/refcounty/{}/refsettlement/{}",
                        prefix, refcounty, refsettlement_id
                    );
                    let _a = name_doc.tag("a", &[("href", &href)]);
                    name_doc.text(&name);
                }
                names.push(name_doc);
            }
            doc.text(" (");
            for (index, item) in names.iter().enumerate() {
                if index > 0 {
                    doc.text(", ");
                }
                doc.append_value(item.get_value());
            }
            doc.text(")");
        }
    }
    Ok(doc)
}

/// Handlers the filter part of the main wsgi page.
fn handle_main_filters(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    refcounty_id: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut items: Vec<yattag::Doc> = Vec::new();

    let mut doc = yattag::Doc::new();
    {
        let _span = doc.tag("span", &[("id", "filter-based-on-position")]);
        let _a = doc.tag("a", &[("href", "#")]);
        doc.text(&tr("Based on position"))
    }
    items.push(doc);

    doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    {
        let _a = doc.tag(
            "a",
            &[("href", &format!("{}/filter-for/everything", prefix))],
        );
        doc.text(&tr("Show complete areas"));
    }
    items.push(doc);

    // Sorted set of refcounty values of all relations.
    let mut refcounties: Vec<_> = relations
        .get_relations()?
        .iter()
        .map(|i| i.get_config().get_refcounty())
        .collect();
    refcounties.sort();
    refcounties.dedup();
    for refcounty in refcounties {
        items.push(handle_main_filters_refcounty(
            ctx,
            relations,
            refcounty_id,
            &refcounty,
        )?);
    }
    doc = yattag::Doc::new();
    {
        let _h1 = doc.tag("h1", &[]);
        doc.text(&tr("Where to map?"));
    }
    {
        let _p = doc.tag("p", &[]);
        doc.text(&format!("{} ", tr("Filters:")));
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                doc.text(" ¦ ");
            }
            doc.append_value(item.get_value());
        }
    }

    let string_pairs = &[
        ("str-gps-wait", tr("Waiting for GPS...")),
        ("str-gps-error", tr("Error from GPS: ")),
        ("str-overpass-wait", tr("Waiting for Overpass...")),
        ("str-overpass-error", tr("Error from Overpass: ")),
        ("str-relations-wait", tr("Waiting for relations...")),
        ("str-relations-error", tr("Error from relations: ")),
        ("str-redirect-wait", tr("Waiting for redirect...")),
    ];
    webframe::emit_l10n_strings_for_js(&doc, string_pairs);
    Ok(doc)
}

/// Handles one relation (one table row) on the main page.
fn handle_main_relation(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    filter_for: &RelationFilter,
    relation_name: &str,
) -> anyhow::Result<Vec<yattag::Doc>> {
    let relation = relations.get_relation(relation_name)?;
    // If checking both streets and house numbers, then "is complete" refers to both street and
    // housenr coverage for "hide complete" purposes.
    let mut complete = true;

    let streets = relation.get_config().should_check_missing_streets();

    let mut row = vec![yattag::Doc::from_text(relation_name)];

    if streets != "only" {
        let (cell, percent) = handle_main_housenr_percent(ctx, &relation)?;
        let doc = yattag::Doc::new();
        doc.append_value(cell.get_value());
        row.push(doc);
        complete &= percent.parse::<f64>().unwrap() >= 100_f64;

        row.push(handle_main_housenr_additional_count(ctx, &relation)?);
    } else {
        row.push(yattag::Doc::new());
        row.push(yattag::Doc::new());
    }

    if streets != "no" {
        let (cell, percent) = handle_main_street_percent(ctx, &relation)?;
        row.push(cell);
        complete &= percent.parse::<f64>().unwrap() >= 100_f64;
    } else {
        row.push(yattag::Doc::new());
    }

    if streets != "no" {
        row.push(handle_main_street_additional_count(ctx, &relation)?);
    } else {
        row.push(yattag::Doc::new());
    }

    let doc = yattag::Doc::new();
    {
        let _a = doc.tag(
            "a",
            &[(
                "href",
                &format!(
                    "https://www.openstreetmap.org/relation/{}",
                    relation.get_config().get_osmrelation()
                ),
            )],
        );
        doc.text(&tr("area boundary"));
    }
    row.push(doc);

    if !filter_for(complete, &relation) {
        row.clear();
    }

    Ok(row)
}

/// Handles the main wsgi page.
///
/// Also handles /osm/filter-for/* which filters for a condition.
fn handle_main(
    request_uri: &str,
    ctx: &context::Context,
    relations: &mut areas::Relations,
) -> anyhow::Result<yattag::Doc> {
    let (filter_for, refcounty) = setup_main_filter_for(request_uri);

    let doc = yattag::Doc::new();
    doc.append_value(
        webframe::get_toolbar(
            ctx,
            &Some(relations.clone()),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    doc.append_value(handle_main_filters(ctx, relations, &refcounty)?.get_value());
    let mut table = vec![vec![
        yattag::Doc::from_text(&tr("Area")),
        yattag::Doc::from_text(&tr("House number coverage")),
        yattag::Doc::from_text(&tr("Additional house numbers")),
        yattag::Doc::from_text(&tr("Street coverage")),
        yattag::Doc::from_text(&tr("Additional streets")),
        yattag::Doc::from_text(&tr("Area boundary")),
    ]];
    for relation_name in relations.get_names() {
        let row = handle_main_relation(ctx, relations, &filter_for, &relation_name)?;
        if !row.is_empty() {
            table.push(row);
        }
    }
    doc.append_value(util::html_table_from_list(&table).get_value());
    {
        let _p = doc.tag("p", &[]);
        let _a = doc.tag(
            "a",
            &[(
                "href",
                "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
            )],
        );
        doc.text(&tr("Add new area"));
    }

    doc.append_value(webframe::get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_main(
    request_uri: &str,
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
) -> PyResult<yattag::PyDoc> {
    match handle_main(request_uri, &ctx.context, &mut relations.relations)
        .context("handle_main() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Determines the HTML title for a given function and relation name.
fn get_html_title(request_uri: &str) -> String {
    let tokens: Vec<String> = request_uri.split('/').map(|i| i.to_string()).collect();
    let mut function: String = "".into();
    let mut relation_name: String = "".into();
    if tokens.len() > 3 {
        function = tokens[2].clone();
        relation_name = tokens[3].clone();
    }
    match function.as_str() {
        "missing-housenumbers" => format!(
            " - {}",
            tr("{0} missing house numbers").replace("{0}", &relation_name)
        ),
        "missing-streets" => format!(" - {} {}", relation_name, tr("missing streets")),
        "street-housenumbers" => format!(" - {} {}", relation_name, tr("existing house numbers")),
        "streets" => format!(" - {} {}", relation_name, tr("existing streets")),
        _ => "".into(),
    }
}

#[pyfunction]
fn py_get_html_title(request_uri: &str) -> String {
    get_html_title(request_uri)
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_handle_streets, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_street_housenumbers,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_housenumbers_view_txt,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_housenumbers_view_chkl,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_missing_streets_view_txt, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_missing_housenumbers,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_missing_streets, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_additional_streets,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_additional_housenumbers,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_main_housenr_additional_count,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_main, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_html_title, module)?)?;
    Ok(())
}
