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

#[pyfunction]
fn py_handle_main_housenr_percent(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<(yattag::PyDoc, String)> {
    match handle_main_housenr_percent(&ctx.context, &relation.relation)
        .context("handle_main_housenr_percent() failed")
    {
        Ok((doc, percent)) => Ok((yattag::PyDoc { doc }, percent)),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

#[pyfunction]
fn py_handle_main_street_percent(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<(yattag::PyDoc, String)> {
    match handle_main_street_percent(&ctx.context, &relation.relation)
        .context("handle_main_street_percent() failed")
    {
        Ok((doc, percent)) => Ok((yattag::PyDoc { doc }, percent)),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

#[pyfunction]
fn py_handle_main_street_additional_count(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<yattag::PyDoc> {
    match handle_main_street_additional_count(&ctx.context, &relation.relation)
        .context("handle_main_street_additional_count() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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
        py_handle_main_housenr_percent,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_main_street_percent,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_main_street_additional_count,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_main_housenr_additional_count,
        module
    )?)?;
    Ok(())
}
