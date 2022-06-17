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

/// Decides if we have an up to date HTML cache entry or not.
fn is_missing_housenumbers_html_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_htmlcache_path();
    let datadir = ctx.get_abspath("data");
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path,
    ];
    is_cache_current(ctx, &cache_path, &dependencies).context("is_cache_current() failed")
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

/// Gets the cached HTML of the missing housenumbers for a relation.
pub fn get_missing_housenumbers_html(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    if is_missing_housenumbers_html_cached(ctx, relation)
        .context("is_missing_housenumbers_html_cached() failed")?
    {
        let files = relation.get_files();
        let stream = files
            .get_housenumbers_htmlcache_read_stream(ctx)
            .context("get_housenumbers_htmlcache_read_stream() failed")?;
        let mut guard = stream.borrow_mut();
        let mut buffer = Vec::new();
        guard
            .read_to_end(&mut buffer)
            .context("read_to_end() failed")?;
        doc.append_value(String::from_utf8(buffer).context("from_utf8() failed")?);
        return Ok(doc);
    }

    let (todo_street_count, todo_count, done_count, percent, table) = relation
        .write_missing_housenumbers()
        .context("write_missing_housenumbers() failed")?;

    {
        let p = doc.tag("p", &[]);
        let prefix = ctx
            .get_ini()
            .get_uri_prefix()
            .context("get_uri_prefix() failed")?;
        let relation_name = relation.get_name();
        p.text(
            &tr("OpenStreetMap is possibly missing the below {0} house numbers for {1} streets.")
                .replace("{0}", &todo_count.to_string())
                .replace("{1}", &todo_street_count.to_string()),
        );
        let percent = util::format_percent(percent).context("format_percent() failed")?;
        p.text(
            &tr(" (existing: {0}, ready: {1}).")
                .replace("{0}", &done_count.to_string())
                .replace("{1}", &percent),
        );
        doc.stag("br");
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
                )],
            );
            a.text(&tr("Filter incorrect information"));
        }
        doc.text(".");
        doc.stag("br");
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-turbo",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Overpass turbo query for the below streets"));
        }
        doc.stag("br");
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-result.txt",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Plain text format"));
        }
        doc.stag("br");
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-result.chkl",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Checklist format"));
        }
    }

    doc.append_value(util::html_table_from_list(&table).get_value());
    let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;
    doc.append_value(util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value());
    doc.append_value(
        util::invalid_filter_keys_to_html(&relation.get_invalid_filter_keys()?).get_value(),
    );

    let files = relation.get_files();
    ctx.get_file_system()
        .write_from_string(&doc.get_value(), &files.get_housenumbers_htmlcache_path())?;

    Ok(doc)
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

/// Decides if we have an up to date plain text cache entry or not.
fn is_missing_housenumbers_txt_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_txtcache_path();
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

/// Gets the cached plain text of the missing housenumbers for a relation.
pub fn get_missing_housenumbers_txt(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<String> {
    let output: String;
    if is_missing_housenumbers_txt_cached(ctx, relation)? {
        let files = relation.get_files();
        output = ctx
            .get_file_system()
            .read_to_string(&files.get_housenumbers_txtcache_path())?;
        return Ok(output);
    }

    let (ongoing_streets, _done_streets) = relation.get_missing_housenumbers()?;
    let mut table: Vec<String> = Vec::new();
    for result in ongoing_streets {
        let range_list = util::get_housenumber_ranges(&result.1);
        let mut range_strings: Vec<String> = range_list
            .iter()
            .map(|i| i.get_lowercase_number())
            .collect();
        // Street name, only_in_reference items.
        let row: String = if !relation
            .get_config()
            .get_street_is_even_odd(result.0.get_osm_name())
        {
            range_strings.sort_by_key(|i| util::split_house_number(i));
            format!(
                "{}\t[{}]",
                result.0.get_osm_name(),
                range_strings.join(", ")
            )
        } else {
            let elements = util::format_even_odd(&range_list);
            format!("{}\t[{}]", result.0.get_osm_name(), elements.join("], ["))
        };
        table.push(row);
    }
    table.sort_by_key(|i| util::get_sort_key(i).unwrap());
    output = table.join("\n");

    let files = relation.get_files();
    ctx.get_file_system()
        .write_from_string(&output, &files.get_housenumbers_txtcache_path())?;
    Ok(output)
}

#[cfg(test)]
mod tests;
