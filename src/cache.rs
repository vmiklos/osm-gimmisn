/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cache module accelerates some functions of the areas module.

use crate::areas;
use crate::context;
use crate::i18n::translate as tr;
use crate::util;
use crate::yattag;
use anyhow::Context;
use pyo3::prelude::*;

/// Decides if we have an up to date cache entry or not.
fn is_cache_outdated(
    ctx: &context::Context,
    cache_path: &str,
    dependencies: &[String],
) -> anyhow::Result<bool> {
    if !ctx.get_file_system().path_exists(cache_path) {
        return Ok(false);
    }

    let cache_mtime = ctx.get_file_system().getmtime(cache_path)?;

    for dependency in dependencies {
        if ctx.get_file_system().path_exists(dependency)
            && ctx.get_file_system().getmtime(dependency)? > cache_mtime
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Decides if we have an up to date HTML cache entry or not.
fn is_missing_housenumbers_html_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path()?;
    let datadir = ctx.get_abspath("data")?;
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path()?,
        relation.get_files().get_osm_housenumbers_path()?,
        relation.get_files().get_ref_housenumbers_path()?,
        relation_path,
    ];
    is_cache_outdated(ctx, &cache_path, &dependencies)
}

#[pyfunction]
fn py_is_missing_housenumbers_html_cached(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<bool> {
    match is_missing_housenumbers_html_cached(&ctx.context, &relation.relation)
        .context("is_missing_housenumbers_html_cached() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Decides if we have an up to date HTML cache entry for additional house numbers or not.
fn is_additional_housenumbers_html_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation
        .get_files()
        .get_additional_housenumbers_htmlcache_path()?;
    let datadir = ctx.get_abspath("data")?;
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path()?,
        relation.get_files().get_osm_housenumbers_path()?,
        relation.get_files().get_ref_housenumbers_path()?,
        relation_path,
    ];
    is_cache_outdated(ctx, &cache_path, &dependencies)
}

/// Gets the cached HTML of the missing housenumbers for a relation.
pub fn get_missing_housenumbers_html(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    if is_missing_housenumbers_html_cached(ctx, relation)? {
        let stream = relation
            .get_files()
            .get_housenumbers_htmlcache_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer = Vec::new();
        guard.read_to_end(&mut buffer)?;
        doc.append_value(String::from_utf8(buffer)?);
        return Ok(doc);
    }

    let (todo_street_count, todo_count, done_count, percent, table) =
        relation.write_missing_housenumbers()?;

    {
        let _p = doc.tag("p", &[]);
        let prefix = ctx.get_ini().get_uri_prefix()?;
        let relation_name = relation.get_name();
        doc.text(
            &tr("OpenStreetMap is possibly missing the below {0} house numbers for {1} streets.")
                .replace("{0}", &todo_count.to_string())
                .replace("{1}", &todo_street_count.to_string()),
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
                    "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
                )],
            );
            doc.text(&tr("Filter incorrect information"));
        }
        doc.text(".");
        doc.stag("br", &[]);
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-turbo",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("Overpass turbo query for the below streets"));
        }
        doc.stag("br", &[]);
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-result.txt",
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
                        "{}/missing-housenumbers/{}/view-result.chkl",
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

    let stream = relation
        .get_files()
        .get_housenumbers_htmlcache_write_stream(ctx)?;
    let mut guard = stream.lock().unwrap();
    guard.write_all(doc.get_value().as_bytes())?;

    Ok(doc)
}

#[pyfunction]
fn py_get_missing_housenumbers_html(
    ctx: context::PyContext,
    mut relation: areas::PyRelation,
) -> PyResult<yattag::PyDoc> {
    match get_missing_housenumbers_html(&ctx.context, &mut relation.relation)
        .context("get_missing_housenumbers_html() failed")
    {
        Ok(value) => Ok(yattag::PyDoc { doc: value }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Gets the cached HTML of the additional housenumbers for a relation.
pub fn get_additional_housenumbers_html(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    if is_additional_housenumbers_html_cached(ctx, relation)? {
        let stream = relation
            .get_files()
            .get_additional_housenumbers_htmlcache_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        doc.append_value(String::from_utf8(buffer)?);
        return Ok(doc);
    }

    let (todo_street_count, todo_count, table) = relation.write_additional_housenumbers()?;

    {
        let _p = doc.tag("p", &[]);
        doc.text(
            &tr("OpenStreetMap additionally has the below {0} house numbers for {1} streets.")
                .replace("{0}", &todo_count.to_string())
                .replace("{1}", &todo_street_count.to_string()),
        );
        doc.stag("br", &[]);
        let _a = doc.tag(
            "a",
            &[(
                "href",
                "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
            )],
        );
        doc.text(&tr("Filter incorrect information"));
    }

    doc.append_value(util::html_table_from_list(&table).get_value());
    let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;
    doc.append_value(util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value());
    doc.append_value(
        util::invalid_filter_keys_to_html(&relation.get_invalid_filter_keys()?).get_value(),
    );

    let stream = relation
        .get_files()
        .get_additional_housenumbers_htmlcache_write_stream(ctx)?;
    let mut guard = stream.lock().unwrap();
    guard.write_all(doc.get_value().as_bytes())?;

    Ok(doc)
}

#[pyfunction]
fn py_get_additional_housenumbers_html(
    ctx: context::PyContext,
    mut relation: areas::PyRelation,
) -> PyResult<yattag::PyDoc> {
    match get_additional_housenumbers_html(&ctx.context, &mut relation.relation)
        .context("get_additional_housenumbers_html() failed")
    {
        Ok(value) => Ok(yattag::PyDoc { doc: value }),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Decides if we have an up to date plain text cache entry or not.
fn is_missing_housenumbers_txt_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_txtcache_path()?;
    let datadir = ctx.get_abspath("data")?;
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path()?,
        relation.get_files().get_osm_housenumbers_path()?,
        relation.get_files().get_ref_housenumbers_path()?,
        relation_path,
    ];
    is_cache_outdated(ctx, &cache_path, &dependencies)
}

#[pyfunction]
fn py_is_missing_housenumbers_txt_cached(
    ctx: context::PyContext,
    relation: areas::PyRelation,
) -> PyResult<bool> {
    match is_missing_housenumbers_txt_cached(&ctx.context, &relation.relation)
        .context("is_missing_housenumbers_txt_cached() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Gets the cached plain text of the missing housenumbers for a relation.
pub fn get_missing_housenumbers_txt(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<String> {
    let output: String;
    if is_missing_housenumbers_txt_cached(ctx, relation)? {
        let stream = relation
            .get_files()
            .get_housenumbers_txtcache_read_stream(ctx)?;
        let mut guard = stream.lock().unwrap();
        let mut buffer = Vec::new();
        guard.read_to_end(&mut buffer)?;
        output = String::from_utf8(buffer)?;
        return Ok(output);
    }

    let (ongoing_streets, _done_streets) = relation.get_missing_housenumbers()?;
    let mut table: Vec<String> = Vec::new();
    for result in ongoing_streets {
        let range_list = util::get_housenumber_ranges(&result.1);
        let mut range_strings: Vec<String> =
            range_list.iter().map(|i| i.get_number()).cloned().collect();
        let row: String;
        // Street name, only_in_reference items.
        if !relation
            .get_config()
            .get_street_is_even_odd(result.0.get_osm_name())
        {
            range_strings.sort_by_key(|i| util::split_house_number(i));
            row = format!(
                "{}\t[{}]",
                result.0.get_osm_name(),
                range_strings.join(", ")
            );
        } else {
            let elements = util::format_even_odd(&range_list);
            row = format!("{}\t[{}]", result.0.get_osm_name(), elements.join("], ["));
        }
        table.push(row);
    }
    table.sort_by_key(|i| util::get_sort_key(i).unwrap());
    output = table.join("\n");

    let stream = relation
        .get_files()
        .get_housenumbers_txtcache_write_stream(ctx)?;
    let mut guard = stream.lock().unwrap();
    guard.write_all(output.as_bytes())?;
    Ok(output)
}

#[pyfunction]
fn py_get_missing_housenumbers_txt(
    ctx: context::PyContext,
    mut relation: areas::PyRelation,
) -> PyResult<String> {
    match get_missing_housenumbers_txt(&ctx.context, &mut relation.relation)
        .context("get_missing_housenumbers_txt() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(
        py_is_missing_housenumbers_html_cached,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_get_missing_housenumbers_html,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_get_additional_housenumbers_html,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_is_missing_housenumbers_txt_cached,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_get_missing_housenumbers_txt,
        module
    )?)?;
    Ok(())
}
