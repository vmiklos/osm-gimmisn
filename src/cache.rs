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

/// Decides if we have an up to date cache entry or not.
fn is_cache_current(
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

/// Decides if we have an up to date HTML cache entry for additional house numbers or not.
fn is_additional_housenumbers_html_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation
        .get_files()
        .get_additional_housenumbers_htmlcache_path();
    let datadir = ctx.get_abspath("data");
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path,
    ];
    is_cache_current(ctx, &cache_path, &dependencies)
}

/// Gets the cached HTML of the additional housenumbers for a relation.
pub fn get_additional_housenumbers_html(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    if is_additional_housenumbers_html_cached(ctx, relation)? {
        let files = relation.get_files();
        let stream = files.get_additional_housenumbers_htmlcache_read_stream(ctx)?;
        let mut guard = stream.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        doc.append_value(String::from_utf8(buffer)?);
        return Ok(doc);
    }

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
                "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
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

    let files = relation.get_files();
    ctx.get_file_system().write_from_string(
        &doc.get_value(),
        &files.get_additional_housenumbers_htmlcache_path(),
    )?;

    Ok(doc)
}

/// Decides if we have an up to date json cache entry or not.
fn is_missing_housenumbers_json_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_jsoncache_path();
    let datadir = ctx.get_abspath("data");
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path,
    ];
    is_cache_current(ctx, &cache_path, &dependencies)
}

/// Gets the cached json of the missing housenumbers for a relation.
pub fn get_missing_housenumbers_json(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<String> {
    let output: String;
    if is_missing_housenumbers_json_cached(ctx, relation)? {
        let files = relation.get_files();
        output = ctx
            .get_file_system()
            .read_to_string(&files.get_housenumbers_jsoncache_path())?;
        return Ok(output);
    }

    let missing_housenumbers = relation.get_missing_housenumbers()?;
    output = serde_json::to_string(&missing_housenumbers)?;

    let files = relation.get_files();
    ctx.get_file_system()
        .write_from_string(&output, &files.get_housenumbers_jsoncache_path())?;
    Ok(output)
}

#[cfg(test)]
mod tests;
