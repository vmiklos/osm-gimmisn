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

#[pyfunction]
fn py_missing_housenumbers_view_turbo(
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match missing_housenumbers_view_turbo(&mut relations.relations, request_uri)
        .context("missing_housenumbers_view_turbo() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

#[pyfunction]
fn py_missing_housenumbers_view_res(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match missing_housenumbers_view_res(&ctx.context, &mut relations.relations, request_uri)
        .context("missing_housenumbers_view_res() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

#[pyfunction]
fn py_missing_streets_view_result(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    match missing_streets_view_result(&ctx.context, &mut relations.relations, request_uri)
        .context("missing_streets_view_result() failed")
    {
        Ok(doc) => Ok(yattag::PyDoc { doc }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
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

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_handle_streets, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_handle_street_housenumbers,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_housenumbers_view_turbo,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_housenumbers_view_res,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_streets_view_result,
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
    Ok(())
}
