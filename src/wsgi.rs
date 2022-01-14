/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The wsgi module contains functionality specific to the web interface

use crate::area_files;
use crate::areas;
use crate::cache;
use crate::context;
use crate::i18n::translate as tr;
use crate::overpass_query;
use crate::util;
use crate::webframe;
use crate::wsgi_additional;
use crate::wsgi_json;
use crate::yattag;
use anyhow::anyhow;
use anyhow::Context;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;

/// Gets the update date string of a file.
fn get_last_modified(path: &str) -> String {
    webframe::format_timestamp(util::get_timestamp(path) as i64)
}

/// Gets the update date of streets for a relation.
fn get_streets_last_modified(relation: &areas::Relation) -> anyhow::Result<String> {
    Ok(get_last_modified(
        &relation.get_files().get_osm_streets_path(),
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
        webframe::get_toolbar(ctx, Some(relations), "streets", relation_name, osmrelation)?
            .get_value(),
    );

    if action == "view-query" {
        let pre = doc.tag("pre", &[]);
        pre.text(&relation.get_osm_streets_query()?);
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
        let mut guard = stream.borrow_mut();
        let mut read = guard.deref_mut();
        let mut csv_read = util::CsvRead::new(&mut read);
        let table = util::tsv_to_list(&mut csv_read)?;
        doc.append_value(util::html_table_from_list(&table).get_value());
    }

    doc.append_value(webframe::get_footer(&get_streets_last_modified(&relation)?).get_value());
    Ok(doc)
}

/// Gets the update date of house numbers for a relation.
fn get_housenumbers_last_modified(relation: &areas::Relation) -> anyhow::Result<String> {
    Ok(get_last_modified(
        &relation.get_files().get_osm_housenumbers_path(),
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
            Some(relations),
            "street-housenumbers",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    let prefix = ctx.get_ini().get_uri_prefix()?;
    if action == "view-query" {
        let pre = doc.tag("pre", &[]);
        pre.text(&relation.get_osm_housenumbers_query()?);
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
            .path_exists(&relation.get_files().get_osm_housenumbers_path())
        {
            let div = doc.tag("div", &[("id", "no-osm-housenumbers")]);
            div.text(&tr("No existing house numbers"));
        } else {
            let stream = relation.get_files().get_osm_housenumbers_read_stream(ctx)?;
            let mut guard = stream.borrow_mut();
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

    let pre = doc.tag("pre", &[]);
    pre.text(&query);
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
        let ret = cache::get_missing_housenumbers_html(ctx, &mut relation);
        doc = ret.context("get_missing_housenumbers_html() failed")?;
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
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        doc.append_value(webframe::handle_no_osm_streets(&prefix, relation_name).get_value());
        return Ok(doc);
    }

    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path())
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
        let p = doc.tag("p", &[]);
        p.text(
            &tr("OpenStreetMap is possibly missing the below {0} streets.")
                .replace("{0}", &todo_count.to_string()),
        );
        p.text(
            &tr(" (existing: {0}, ready: {1}).")
                .replace("{0}", &done_count.to_string())
                .replace("{1}", &util::format_percent(&percent)?),
        );
        p.stag("br", &[]);
        {
            let a = p.tag(
                "a",
                &[(
                    "href",
                    &format!("{}/missing-streets/{}/view-turbo", prefix, relation_name),
                )],
            );
            a.text(&tr(
                "Overpass turbo query for streets with questionable names",
            ));
        }
        p.stag("br", &[]);
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-streets/{}/view-result.txt",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Plain text format"));
        }
        p.stag("br", &[]);
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-streets/{}/view-result.chkl",
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
    config.set_letter_suffix_style(util::LetterSuffixStyle::Lower);
    relation.set_config(&config);

    let output: String;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path())
    {
        output = tr("No existing house numbers");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path())
    {
        output = tr("No reference house numbers");
    } else {
        output = cache::get_missing_housenumbers_txt(ctx, &mut relation)?;
    }
    Ok(output)
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
    config.set_letter_suffix_style(util::LetterSuffixStyle::Lower);
    relation.set_config(&config);

    let output: String;
    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path())
    {
        output = tr("No existing house numbers");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path())
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
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        output = tr("No existing streets");
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path())
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
    let div = doc.tag("div", &[("id", "update-success")]);
    div.text(&tr("Update successful."));
    Ok(doc)
}

/// Gets the update date for missing house numbers.
fn ref_housenumbers_last_modified(
    relations: &mut areas::Relations,
    name: &str,
) -> anyhow::Result<String> {
    let relation = relations.get_relation(name)?;
    let t_ref = util::get_timestamp(&relation.get_files().get_ref_housenumbers_path());
    let t_housenumbers = util::get_timestamp(&relation.get_files().get_osm_housenumbers_path());
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
            Some(relations),
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
            let pre = doc.tag("pre", &[]);
            let stream = relation.get_files().get_ref_housenumbers_read_stream(ctx)?;
            let mut guard = stream.borrow_mut();
            let mut buffer: Vec<u8> = Vec::new();
            guard.read_to_end(&mut buffer)?;
            pre.text(&String::from_utf8(buffer)?);
        }
        date = get_last_modified(&relation.get_files().get_ref_housenumbers_path());
    } else if action == "update-result" {
        doc.append_value(missing_housenumbers_update(ctx, relations, relation_name)?.get_value())
    } else {
        // assume view-result
        let ret = missing_housenumbers_view_res(ctx, relations, request_uri);
        doc.append_value(
            ret.context("missing_housenumbers_view_res() failed")?
                .get_value(),
        )
    }

    if date.is_empty() {
        date = ref_housenumbers_last_modified(relations, relation_name)?;
    }
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
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
        if relation.get_config().should_show_ref_street(&key) {
            streets.push(key)
        }
    }
    let query = areas::make_turbo_query_for_streets(&relation, &streets);

    let pre = doc.tag("pre", &[]);
    pre.text(&query);
    Ok(doc)
}

/// Gets the update date for missing/additional streets.
fn streets_diff_last_modified(relation: &areas::Relation) -> String {
    let t_ref = util::get_timestamp(&relation.get_files().get_ref_streets_path()) as i64;
    let t_osm = util::get_timestamp(&relation.get_files().get_osm_streets_path()) as i64;
    webframe::format_timestamp(std::cmp::max(t_ref, t_osm))
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
            Some(relations),
            "missing-streets",
            relation_name,
            osmrelation,
        )?
        .get_value(),
    );

    if action == "view-turbo" {
        doc.append_value(missing_streets_view_turbo(relations, request_uri)?.get_value());
    } else if action == "view-query" {
        let pre = doc.tag("pre", &[]);
        let stream = relation.get_files().get_ref_streets_read_stream(ctx)?;
        let mut guard = stream.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        pre.text(&String::from_utf8(buffer)?);
    } else if action == "update-result" {
        doc.append_value(missing_streets_update(ctx, relations, relation_name)?.get_value());
    } else {
        // assume view-result
        doc.append_value(missing_streets_view_result(ctx, relations, request_uri)?.get_value());
    }

    let date = streets_diff_last_modified(&relation);
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
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
            Some(relations),
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

    let date = streets_diff_last_modified(&relation);
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

/// Gets the update date for missing/additional housenumbers.
fn housenumbers_diff_last_modified(relation: &areas::Relation) -> String {
    let t_ref = util::get_timestamp(&relation.get_files().get_ref_housenumbers_path()) as i64;
    let t_osm = util::get_timestamp(&relation.get_files().get_osm_housenumbers_path()) as i64;
    webframe::format_timestamp(std::cmp::max(t_ref, t_osm))
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
            Some(relations),
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

    let date = housenumbers_diff_last_modified(&relation);
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
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
    let files = relation.get_files();
    if ctx
        .get_file_system()
        .path_exists(&files.get_housenumbers_percent_path())
    {
        let stream = files.get_housenumbers_percent_read_stream(ctx)?;
        let mut guard = stream.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        percent = String::from_utf8(buffer)?;
    }

    let doc = yattag::Doc::new();
    if percent != "N/A" {
        let date = get_last_modified(&files.get_housenumbers_percent_path());
        let strong = doc.tag("strong", &[]);
        let a = strong.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        a.text(&util::format_percent(&percent)?);
        return Ok((doc, percent));
    }

    let strong = doc.tag("strong", &[]);
    let a = strong.tag("a", &[("href", &url)]);
    a.text(&tr("missing house numbers"));
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
        .path_exists(&relation.get_files().get_streets_percent_path())
    {
        let stream = relation.get_files().get_streets_percent_read_stream(ctx)?;
        let mut guard = stream.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        percent = String::from_utf8(buffer)?;
    }

    let doc = yattag::Doc::new();
    if percent != "N/A" {
        let date = get_last_modified(&relation.get_files().get_streets_percent_path());
        let strong = doc.tag("strong", &[]);
        let a = strong.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        a.text(&util::format_percent(&percent)?);
        return Ok((doc, percent));
    }

    let strong = doc.tag("strong", &[]);
    let a = strong.tag("a", &[("href", &url)]);
    a.text(&tr("missing streets"));
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
    let files = relation.get_files();
    if ctx
        .get_file_system()
        .path_exists(&files.get_streets_additional_count_path())
    {
        let stream = files.get_streets_additional_count_read_stream(ctx)?;
        let mut guard = stream.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        additional_count = String::from_utf8(buffer)?;
    }

    let doc = yattag::Doc::new();
    if !additional_count.is_empty() {
        let date = get_last_modified(&files.get_streets_additional_count_path());
        let strong = doc.tag("strong", &[]);
        let a = strong.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        a.text(&tr("{} streets").replace("{}", &additional_count));
        return Ok(doc);
    }

    let strong = doc.tag("strong", &[]);
    let a = strong.tag("a", &[("href", &url)]);
    a.text(&tr("additional streets"));
    Ok(doc)
}

fn get_housenr_additional_count(
    ctx: &context::Context,
    files: &area_files::RelationFiles,
) -> anyhow::Result<String> {
    if ctx
        .get_file_system()
        .path_exists(&files.get_housenumbers_additional_count_path())
    {
        let stream = files.get_housenumbers_additional_count_read_stream(ctx)?;

        let mut guard = stream.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        guard.read_to_end(&mut buffer)?;
        return Ok(String::from_utf8(buffer)?.trim().into());
    }

    Ok("".into())
}

/// Handles the housenumber additional count part of the main page.
pub fn handle_main_housenr_additional_count(
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
    let files = relation.get_files();
    let additional_count = get_housenr_additional_count(ctx, files)?;

    let doc = yattag::Doc::new();
    if !additional_count.is_empty() {
        let date = get_last_modified(&files.get_housenumbers_additional_count_path());
        let strong = doc.tag("strong", &[]);
        let a = strong.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        a.text(&tr("{} house numbers").replace("{}", &additional_count));
        return Ok(doc);
    }

    let strong = doc.tag("strong", &[]);
    let a = strong.tag("a", &[("href", &url)]);
    a.text(&tr("additional house numbers"));
    Ok(doc)
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
    let refsettlement_arc = Arc::new(refsettlement_filter.to_string());
    let refsettlement_filter = refsettlement_arc;
    Box::new(move |_complete, relation| {
        let config = relation.get_config();
        config.get_refcounty() == refcounty_filter.as_str()
            && config.get_refsettlement() == refsettlement_filter.as_str()
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
        let a = doc.tag(
            "a",
            &[(
                "href",
                &format!("{}/filter-for/refcounty/{}/whole-county", prefix, refcounty),
            )],
        );
        a.text(&name);
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
                    let a = name_doc.tag("a", &[("href", &href)]);
                    a.text(&name);
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
        let span = doc.tag("span", &[("id", "filter-based-on-position")]);
        let a = span.tag("a", &[("href", "#")]);
        a.text(&tr("Based on position"))
    }
    items.push(doc);

    doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    {
        let a = doc.tag(
            "a",
            &[("href", &format!("{}/filter-for/everything", prefix))],
        );
        a.text(&tr("Show complete areas"));
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
        let h1 = doc.tag("h1", &[]);
        h1.text(&tr("Where to map?"));
    }
    {
        let p = doc.tag("p", &[]);
        p.text(&format!("{} ", tr("Filters:")));
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
        let a = doc.tag(
            "a",
            &[(
                "href",
                &format!(
                    "https://www.openstreetmap.org/relation/{}",
                    relation.get_config().get_osmrelation()
                ),
            )],
        );
        a.text(&tr("area boundary"));
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
            Some(relations),
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
        let p = doc.tag("p", &[]);
        let a = p.tag(
            "a",
            &[(
                "href",
                "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
            )],
        );
        a.text(&tr("Add new area"));
    }

    doc.append_value(webframe::get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
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

/// Produces the <head> tag and its contents.
fn write_html_head(ctx: &context::Context, doc: &yattag::Tag, title: &str) -> anyhow::Result<()> {
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let head = doc.tag("head", &[]);
    head.stag("meta", &[("charset", "UTF-8")]);
    head.stag(
        "meta",
        &[
            ("name", "viewport"),
            ("content", "width=device-width, initial-scale=1"),
        ],
    );
    {
        let title_tag = head.tag("title", &[]);
        title_tag.text(&format!("{}{}", tr("Where to map?"), title))
    }
    head.stag(
        "link",
        &[
            ("rel", "icon"),
            ("type", "image/vnd.microsoft.icon"),
            ("sizes", "16x12"),
            ("href", &format!("{}/favicon.ico", prefix)),
        ],
    );
    head.stag(
        "link",
        &[
            ("rel", "icon"),
            ("type", "image/svg+xml"),
            ("sizes", "any"),
            ("href", &format!("{}/favicon.svg", prefix)),
        ],
    );

    let css_path = format!("{}/{}", ctx.get_ini().get_workdir()?, "osm.min.css");
    let contents = std::fs::read_to_string(css_path)?;
    {
        let style = head.tag("style", &[]);
        style.text(&contents);
    }

    {
        let noscript = head.tag("noscript", &[]);
        let style = noscript.tag("style", &[("type", "text/css")]);
        style.text(".no-js { display: block; }");
        style.text(".js { display: none; }");
    }

    let script = head.tag(
        "script",
        &[
            ("defer", ""),
            ("src", &format!("{}/static/bundle.js", prefix)),
        ],
    );
    drop(script);
    Ok(())
}

/// Dispatches plain text requests based on their URIs.
fn our_application_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<rouille::Response> {
    let mut content_type = "text/plain; charset=utf-8";
    let mut headers: webframe::Headers = Vec::new();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    let mut chkl = false;
    if let Some(value) = std::path::Path::new(request_uri).extension() {
        chkl = value == "chkl";
    }
    let data: Vec<u8>;
    if request_uri.starts_with(&format!("{}/missing-streets/", prefix)) {
        let (output, relation_name) = missing_streets_view_txt(ctx, relations, request_uri, chkl)?;
        if chkl {
            content_type = "application/octet-stream";
            headers.push((
                "Content-Disposition".into(),
                format!(r#"attachment;filename="{}.txt""#, relation_name).into(),
            ));
        }
        data = output.as_bytes().to_vec();
    } else if request_uri.starts_with(&format!("{}/additional-streets/", prefix)) {
        let (output, relation_name) =
            wsgi_additional::additional_streets_view_txt(ctx, relations, request_uri, chkl)?;
        if chkl {
            content_type = "application/octet-stream";
            headers.push((
                "Content-Disposition".into(),
                format!(r#"attachment;filename="{}.txt""#, relation_name).into(),
            ));
        }
        data = output.as_bytes().to_vec();
    } else {
        // assume prefix + "/missing-housenumbers/"
        if chkl {
            let (output, relation_name) =
                missing_housenumbers_view_chkl(ctx, relations, request_uri)?;
            content_type = "application/octet-stream";
            headers.push((
                "Content-Disposition".into(),
                format!(r#"attachment;filename="{}.txt""#, relation_name).into(),
            ));
            data = output.as_bytes().to_vec();
        } else if request_uri.ends_with("robots.txt") {
            data = std::fs::read(&ctx.get_abspath("data/robots.txt"))?;
        } else {
            // assume txt
            let output = missing_housenumbers_view_txt(ctx, relations, request_uri)?;
            data = output.as_bytes().to_vec();
        }
    }
    headers.push(("Content-type".into(), content_type.into()));
    Ok(webframe::make_response(200_u16, headers, data))
}

type Handler = fn(&context::Context, &mut areas::Relations, &str) -> anyhow::Result<yattag::Doc>;

lazy_static! {
    static ref HANDLERS: HashMap<String, Handler> = {
        let mut ret: HashMap<String, Handler> = HashMap::new();
        ret.insert("/streets/".into(), handle_streets);
        ret.insert("/missing-streets/".into(), handle_missing_streets);
        ret.insert("/additional-streets/".into(), handle_additional_streets);
        ret.insert(
            "/additional-housenumbers/".into(),
            handle_additional_housenumbers,
        );
        ret.insert("/street-housenumbers/".into(), handle_street_housenumbers);
        ret.insert("/missing-housenumbers/".into(), handle_missing_housenumbers);
        ret.insert("/housenumber-stats/".into(), webframe::handle_stats);
        ret
    };
}

/// Decides request_uri matches what handler.
fn get_handler(ctx: &context::Context, request_uri: &str) -> anyhow::Result<Option<Handler>> {
    let prefix = ctx.get_ini().get_uri_prefix()?;
    for (key, value) in HANDLERS.iter() {
        if request_uri.starts_with(&format!("{}{}", prefix, key)) {
            return Ok(Some(*value));
        }
    }
    Ok(None)
}

/// Dispatches the request based on its URI.
fn our_application(
    request: &rouille::Request,
    ctx: &context::Context,
) -> anyhow::Result<rouille::Response> {
    let language = util::setup_localization(request.headers());

    let mut relations = areas::Relations::new(ctx).context("areas::Relations::new() failed")?;

    let request_uri = webframe::get_request_uri(request, ctx, &mut relations)?;
    let mut ext: String = "".into();
    if let Some(value) = std::path::Path::new(&request_uri).extension() {
        ext = value.to_str().unwrap().into();
    }

    if ext == "txt" || ext == "chkl" {
        return our_application_txt(ctx, &mut relations, &request_uri);
    }

    let prefix = ctx.get_ini().get_uri_prefix()?;
    if !(request_uri == "/" || request_uri.starts_with(&prefix)) {
        let doc = webframe::handle_404();
        return Ok(webframe::make_response(
            404_u16,
            vec![("Content-type".into(), "text/html; charset=utf-8".into())],
            doc.get_value().as_bytes().to_vec(),
        ));
    }

    if request_uri.starts_with(&format!("{}/static/", prefix))
        || request_uri.ends_with("favicon.ico")
        || request_uri.ends_with("favicon.svg")
    {
        let (output, content_type, mut headers) = webframe::handle_static(ctx, &request_uri)?;
        headers.push(("Content-type".into(), content_type.into()));
        return Ok(webframe::make_response(200_u16, headers, output));
    }

    if ext == "json" {
        return wsgi_json::our_application_json(ctx, &mut relations, &request_uri);
    }

    let doc = yattag::Doc::new();
    util::write_html_header(&doc);
    {
        let html = doc.tag("html", &[("lang", &language)]);
        write_html_head(ctx, &html, &get_html_title(&request_uri))?;

        let body = html.tag("body", &[]);
        let no_such_relation = webframe::check_existing_relation(ctx, &relations, &request_uri)?;
        let handler = get_handler(ctx, &request_uri)?;
        if !no_such_relation.get_value().is_empty() {
            body.append_value(no_such_relation.get_value());
        } else if let Some(handler) = handler {
            let value = handler(ctx, &mut relations, &request_uri)
                .context("handler() failed")?
                .get_value();
            body.append_value(value);
        } else if request_uri.starts_with(&format!("{}/webhooks/github", prefix)) {
            body.append_value(webframe::handle_github_webhook(request, ctx)?.get_value());
        } else {
            body.append_value(handle_main(&request_uri, ctx, &mut relations)?.get_value());
        }
    }

    let err = ctx.get_unit().make_error();
    if !err.is_empty() {
        return Err(anyhow!(err));
    }
    Ok(webframe::make_response(
        200_u16,
        vec![("Content-type".into(), "text/html; charset=utf-8".into())],
        doc.get_value().as_bytes().to_vec(),
    ))
}

/// The entry point of this WSGI app.
pub fn application(request: &rouille::Request, ctx: &context::Context) -> rouille::Response {
    match our_application(request, ctx).context("our_application() failed") {
        // Compress.
        Ok(value) => rouille::content_encoding::apply(request, value),
        Err(err) => webframe::handle_error(request, &format!("{:?}", err)),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::Read;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::io::Write;
    use std::rc::Rc;

    /// Shared struct for wsgi tests.
    pub struct TestWsgi {
        gzip_compress: bool,
        ctx: context::Context,
        headers: Vec<(String, String)>,
        bytes: Vec<u8>,
        absolute_path: bool,
        expected_status: u16,
    }

    impl TestWsgi {
        pub fn new() -> Self {
            let gzip_compress = false;
            let ctx = context::tests::make_test_context().unwrap();
            let headers: Vec<(String, String)> = Vec::new();
            let bytes: Vec<u8> = Vec::new();
            let absolute_path = false;
            let expected_status = 200_u16;
            TestWsgi {
                gzip_compress,
                ctx,
                headers,
                bytes,
                absolute_path,
                expected_status,
            }
        }

        pub fn get_ctx(&mut self) -> &mut context::Context {
            &mut self.ctx
        }

        /// Finds all matching subelements, by tag name or path.
        pub fn find_all(package: &sxd_document::Package, path: &str) -> Vec<String> {
            let document = package.as_document();
            let value = sxd_xpath::evaluate_xpath(&document, &format!("/html/{}", path)).unwrap();
            let mut ret: Vec<String> = Vec::new();
            if let sxd_xpath::Value::Nodeset(nodeset) = value {
                ret = nodeset.iter().map(|i| i.string_value()).collect();
            };
            ret
        }

        /// Generates an XML DOM for a given wsgi path.
        pub fn get_dom_for_path(&mut self, path: &str) -> sxd_document::Package {
            let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
            let abspath: String;
            if self.absolute_path {
                abspath = path.into();
            } else {
                abspath = format!("{}{}", prefix, path);
            }
            if self.gzip_compress {
                self.headers
                    .push(("Accept-Encoding".into(), "gzip, deflate".into()));
            }
            let request = rouille::Request::fake_http(
                "GET",
                abspath,
                self.headers.clone(),
                self.bytes.clone(),
            );
            let response = application(&request, &self.ctx);
            let mut data = Vec::new();
            let (mut reader, _size) = response.data.into_reader_and_size();
            reader.read_to_end(&mut data).unwrap();
            // Make sure the built-in error catcher is not kicking in.
            assert_eq!(response.status_code, self.expected_status);
            let mut headers_map = HashMap::new();
            for (key, value) in response.headers {
                headers_map.insert(key, value);
            }
            assert_eq!(headers_map["Content-type"], "text/html; charset=utf-8");
            assert_eq!(data.is_empty(), false);
            let mut output: Vec<u8> = Vec::new();
            if self.gzip_compress {
                let mut gz = flate2::read::GzDecoder::new(data.as_slice());
                gz.read_to_end(&mut output).unwrap();
            } else {
                output = data;
            }
            let output_xml =
                format!("{}", String::from_utf8(output).unwrap()).replace("<!DOCTYPE html>", "");
            let package = sxd_document::parser::parse(&output_xml).unwrap();
            package
        }

        /// Generates a string for a given wsgi path.
        pub fn get_txt_for_path(&mut self, path: &str) -> String {
            let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
            let abspath = format!("{}{}", prefix, path);
            let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
            let response = application(&request, &self.ctx);
            let mut data = Vec::new();
            let (mut reader, _size) = response.data.into_reader_and_size();
            reader.read_to_end(&mut data).unwrap();
            // Make sure the built-in exception catcher is not kicking in.
            assert_eq!(response.status_code, 200);
            let mut headers_map = HashMap::new();
            for (key, value) in response.headers {
                headers_map.insert(key, value);
            }
            if path.ends_with(".chkl") {
                assert_eq!(headers_map["Content-type"], "application/octet-stream");
            } else {
                assert_eq!(headers_map["Content-type"], "text/plain; charset=utf-8");
            }
            assert_eq!(data.is_empty(), false);
            let output = String::from_utf8(data).unwrap();
            output
        }

        /// Generates an json value for a given wsgi path.
        pub fn get_json_for_path(&mut self, path: &str) -> serde_json::Value {
            let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
            let abspath = format!("{}{}", prefix, path);
            let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
            let response = application(&request, &self.ctx);
            let mut data = Vec::new();
            let (mut reader, _size) = response.data.into_reader_and_size();
            reader.read_to_end(&mut data).unwrap();
            // Make sure the built-in exception catcher is not kicking in.
            assert_eq!(response.status_code, 200);
            let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
            assert_eq!(
                headers_map["Content-type"],
                "application/json; charset=utf-8"
            );
            assert_eq!(data.is_empty(), false);
            let value: serde_json::Value =
                serde_json::from_str(&String::from_utf8(data).unwrap()).unwrap();
            value
        }

        /// Generates a CSS string for a given wsgi path.
        fn get_css_for_path(&mut self, path: &str) -> String {
            let prefix = self.ctx.get_ini().get_uri_prefix().unwrap();
            let abspath = format!("{}{}", prefix, path);
            let request = rouille::Request::fake_http("GET", abspath, vec![], vec![]);
            let response = application(&request, &self.ctx);
            let mut data = Vec::new();
            let (mut reader, _size) = response.data.into_reader_and_size();
            reader.read_to_end(&mut data).unwrap();
            // Make sure the built-in exception catcher is not kicking in.
            assert_eq!(response.status_code, 200);
            let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
            assert_eq!(headers_map["Content-type"], "text/css; charset=utf-8");
            assert_eq!(data.is_empty(), false);
            String::from_utf8(data).unwrap()
        }
    }

    /// Tests handle_streets(): if the output is well-formed.
    #[test]
    fn test_handle_streets_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/streets/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_streets(): if the view-query output is well-formed.
    #[test]
    fn test_handle_streets_view_query_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/streets/gazdagret/view-query");
        let results = TestWsgi::find_all(&root, "body/pre");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_streets(): if the update-result output is well-formed.
    #[test]
    fn test_handle_streets_update_result_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let routes = vec![context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-streets-gazdagret.csv",
        )];
        let network = context::tests::TestNetwork::new(&routes);
        let network_arc: Arc<dyn context::Network> = Arc::new(network);
        test_wsgi.ctx.set_network(&network_arc);
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/streets-gazdagret.csv", &streets_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/streets/gazdagret/update-result");

        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let results = TestWsgi::find_all(&root, "body");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_streets(): if the update-result output on error is well-formed.
    #[test]
    fn test_handle_streets_update_result_error_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let routes = vec![context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "", // no result -> error
        )];
        let network = context::tests::TestNetwork::new(&routes);
        let network_arc: Arc<dyn context::Network> = Arc::new(network);
        test_wsgi.ctx.set_network(&network_arc);

        let root = test_wsgi.get_dom_for_path("/streets/gazdagret/update-result");

        let results = TestWsgi::find_all(&root, "body/div[@id='overpass-error']");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_streets(): if the update-result output is well-formed for
    /// should_check_missing_streets() == "only".
    #[test]
    fn test_handle_streets_update_result_missing_streets_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let routes = vec![context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-streets-ujbuda.csv",
        )];
        let network = context::tests::TestNetwork::new(&routes);
        let network_arc: Arc<dyn context::Network> = Arc::new(network);
        test_wsgi.ctx.set_network(&network_arc);
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/streets-ujbuda.csv", &streets_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/streets/ujbuda/update-result");

        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let results = TestWsgi::find_all(&root, "body");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the output is well-formed.
    #[test]
    fn test_missing_housenumbers_well_formed() {
        let cache_path = "workdir/gazdagret.htmlcache.en";
        std::fs::File::create(cache_path).unwrap();
        std::fs::remove_file(cache_path).unwrap();
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");
        let mut results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);

        // refstreets: >0 invalid osm name
        results = TestWsgi::find_all(&root, "body/div[@id='osm-invalids-container']");
        assert_eq!(results.len(), 1);
        // refstreets: >0 invalid ref name
        results = TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: the output for a non-existing relation.
    #[test]
    fn test_missing_housenumbers_no_such_relation() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret42/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-such-relation-error']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the output is well-formed (URL rewrite).
    #[test]
    fn test_missing_housenumbers_compat() {
        let mut test_wsgi = TestWsgi::new();
        let mut file_system = context::tests::TestFileSystem::new();
        let streets_value = context::tests::TestFileSystem::make_file();
        let htmlcache_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[
                ("workdir/gazdagret.percent", &streets_value),
                ("workdir/gazdagret.htmlcache.en", &htmlcache_value),
            ],
        );
        file_system.set_files(&files);
        // Make sure the cache is outdated.
        let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
        mtimes.insert(
            test_wsgi.ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
            Rc::new(RefCell::new(0_f64)),
        );
        file_system.set_mtimes(&mtimes);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);

        let root = test_wsgi.get_dom_for_path("/suspicious-streets/gazdagret/view-result");

        {
            let mut guard = streets_value.borrow_mut();
            assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        }
        {
            let mut guard = htmlcache_value.borrow_mut();
            assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        }
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the output is well-formed (URL rewrite for relation name).
    #[test]
    fn test_missing_housenumbers_compat_relation() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/suspicious-streets/budapest_22/view-result");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the output is well-formed, no osm streets case.
    #[test]
    fn test_missing_housenumbers_no_osm_streets() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the output is well-formed, no osm housenumbers case.
    #[test]
    fn test_missing_housenumbers_no_osm_housenumbers() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the output is well-formed, no ref housenumbers case.
    #[test]
    fn test_missing_housenumbers_no_ref_housenumbers_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-ref-housenumbers']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: the txt output.
    #[test]
    fn test_missing_housenumbers_view_result_txt() {
        let mut test_wsgi = TestWsgi::new();
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/budafok/view-result.txt");
        // Note how 12 is ordered after 2.
        assert_eq!(result, "Vöröskúti határsor\t[2, 12, 34, 36*]");
    }

    /// Tests the missing house numbers page: the txt output (even-odd streets).
    #[test]
    fn test_missing_housenumbers_view_result_txt_even_odd() {
        let mut test_wsgi = TestWsgi::new();
        let cache_path = test_wsgi.ctx.get_abspath("workdir/gazdagret.txtcache");
        if std::path::Path::new(&cache_path).exists() {
            std::fs::remove_file(&cache_path).unwrap();
        }
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
        let expected = r#"Hamzsabégi út	[1]
Törökugrató utca	[7], [10]
Tűzkő utca	[1], [2]"#;
        assert_eq!(result, expected);
    }

    /// Tests the missing house numbers page: the chkl output.
    #[test]
    fn test_missing_housenumbers_view_result_chkl() {
        let mut test_wsgi = TestWsgi::new();
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/budafok/view-result.chkl");
        // Note how 12 is ordered after 2.
        assert_eq!(result, "[ ] Vöröskúti határsor [2, 12, 34, 36*]");
    }

    /// Tests the missing house numbers page: the chkl output (even-odd streets).
    #[test]
    fn test_missing_housenumbers_view_result_chkl_even_odd() {
        let mut test_wsgi = TestWsgi::new();
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
        let expected = r#"[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [7], [10]
[ ] Tűzkő utca [1], [2]"#;
        assert_eq!(result, expected);
    }

    /// Tests the missing house numbers page: the chkl output (even-odd streets).
    #[test]
    fn test_missing_housenumbers_view_result_chkl_even_odd_split() {
        let mut test_wsgi = TestWsgi::new();
        let hoursnumbers_ref = r#"Hamzsabégi út	1
Ref Name 1	1
Ref Name 1	2
Törökugrató utca	1	comment
Törökugrató utca	10
Törökugrató utca	11
Törökugrató utca	12
Törökugrató utca	2
Törökugrató utca	7
Tűzkő utca	1
Tűzkő utca	2
Tűzkő utca	9
Tűzkő utca	10
Tűzkő utca	12
Tűzkő utca	13
Tűzkő utca	14
Tűzkő utca	15
Tűzkő utca	16
Tűzkő utca	17
Tűzkő utca	18
Tűzkő utca	19
Tűzkő utca	20
Tűzkő utca	21
Tűzkő utca	22
Tűzkő utca	22
Tűzkő utca	24
Tűzkő utca	25
Tűzkő utca	26
Tűzkő utca	27
Tűzkő utca	28
Tűzkő utca	29
Tűzkő utca	30
Tűzkő utca	31
"#;
        let housenumbers_ref_value = context::tests::TestFileSystem::make_file();
        housenumbers_ref_value
            .borrow_mut()
            .write_all(hoursnumbers_ref.as_bytes())
            .unwrap();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[(
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &housenumbers_ref_value,
            )],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
        let expected = r#"[ ] Hamzsabégi út [1]
[ ] Törökugrató utca [7], [10]
[ ] Tűzkő utca [1, 13, 15, 17, 19, 21, 25, 27, 29, 31]
[ ] Tűzkő utca [2, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]"#;
        assert_eq!(result, expected);
    }

    /// Tests the missing house numbers page: the chkl output, no osm streets case.
    #[test]
    fn test_missing_housenumbers_view_result_chkl_no_osm_streets() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
        assert_eq!(result, "No existing streets");
    }

    /// Tests the missing house numbers page: the chkl output, no osm housenumbers case.
    #[test]
    fn test_missing_housenumbers_view_result_chkl_no_osm_housenumbers() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
        assert_eq!(result, "No existing house numbers");
    }

    /// Tests the missing house numbers page: the chkl output, no ref housenumbers case.
    #[test]
    fn test_missing_housenumbers_view_result_chkl_no_ref_housenumbers() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.chkl");
        assert_eq!(result, "No reference house numbers");
    }

    /// Tests the missing house numbers page: the txt output, no osm streets case.
    #[test]
    fn test_missing_housenumbers_view_result_txt_no_osm_streets() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
        assert_eq!(result, "No existing streets");
    }

    /// Tests the missing house numbers page: the txt output, no osm housenumbers case.
    #[test]
    fn test_missing_housenumbers_view_result_txt_no_osm_housenumbers() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
        assert_eq!(result, "No existing house numbers");
    }

    /// Tests the missing house numbers page: the txt output, no ref housenumbers case.
    #[test]
    fn test_missing_housenumbers_view_result_txt_no_ref_housenumbers() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let result = test_wsgi.get_txt_for_path("/missing-housenumbers/gazdagret/view-result.txt");
        assert_eq!(result, "No reference house numbers");
    }

    /// Tests the missing house numbers page: if the view-turbo output is well-formed.
    #[test]
    fn test_missing_housenumbers_view_turbo_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-turbo");
        let results = TestWsgi::find_all(&root, "body/pre");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the view-query output is well-formed.
    #[test]
    fn test_missing_housenumbers_view_query_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/view-query");
        let results = TestWsgi::find_all(&root, "body/pre");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing house numbers page: if the update-result output links back to the correct page.
    #[test]
    fn test_missing_housenumbers_update_result_link() {
        let mut test_wsgi = TestWsgi::new();
        let housenumbers_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[(
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &housenumbers_value,
            )],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);
        let root = test_wsgi.get_dom_for_path("/missing-housenumbers/gazdagret/update-result");
        let mut guard = housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let prefix = test_wsgi.ctx.get_ini().get_uri_prefix().unwrap();
        let results = TestWsgi::find_all(
            &root,
            &format!(
                "body/a[@href='{}/missing-housenumbers/gazdagret/view-result']",
                prefix
            ),
        );
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_street_housenumbers(): view result: the update-result link.
    #[test]
    fn test_housenumbers_view_result_update_result_link() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-result");
        let uri = format!(
            "{}/missing-housenumbers/gazdagret/view-result",
            test_wsgi.ctx.get_ini().get_uri_prefix().unwrap()
        );
        let results = TestWsgi::find_all(
            &root,
            &format!("body/div[@id='toolbar']/a[@href='{}']", uri),
        );
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_street_housenumbers(): if the view-query output is well-formed.
    #[test]
    fn test_housenumbers_view_query_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-query");
        let results = TestWsgi::find_all(&root, "body/pre");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_street_housenumbers(): if the update-result output is well-formed.
    #[test]
    fn test_housenumbers_update_result_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let routes = vec![context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-housenumbers-gazdagret.csv",
        )];
        let network = context::tests::TestNetwork::new(&routes);
        let network_arc: Arc<dyn context::Network> = Arc::new(network);
        test_wsgi.ctx.set_network(&network_arc);
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/street-housenumbers-gazdagret.csv", &streets_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/update-result");

        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let results = TestWsgi::find_all(&root, "body");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_street_housenumbers(): if the update-result output on error is well-formed.
    #[test]
    fn test_housenumbers_update_result_error_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let routes = vec![context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        )];
        let network = context::tests::TestNetwork::new(&routes);
        let network_arc: Arc<dyn context::Network> = Arc::new(network);
        test_wsgi.ctx.set_network(&network_arc);

        let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/update-result");

        let results = TestWsgi::find_all(&root, "body/div[@id='overpass-error']");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_street_housenumbers(): if the output is well-formed, no osm streets case.
    #[test]
    fn test_housenumbers_no_osm_streets_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/street-housenumbers/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: if the output is well-formed.
    #[test]
    fn test_missing_streets_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/gazdagret-streets.percent", &streets_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");

        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let mut results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
        // refstreets: >0 invalid osm name
        results = TestWsgi::find_all(&root, "body/div[@id='osm-invalids-container']");
        assert_eq!(results.len(), 1);
        // refstreets: >0 invalid ref name
        results = TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: if the output is well-formed (URL rewrite).
    #[test]
    fn test_missing_streets_well_formed_compat() {
        let mut test_wsgi = TestWsgi::new();
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/gazdagret-streets.percent", &streets_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/suspicious-relations/gazdagret/view-result");

        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: if the output is well-formed, no osm streets case.
    #[test]
    fn test_missing_streets_no_osm_streets_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: if the output is well-formed, no ref streets case.
    #[test]
    fn test_missing_streets_no_ref_streets_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-result");
        let results = TestWsgi::find_all(&root, "body/div[@id='no-ref-streets']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: the txt output.
    #[test]
    fn test_missing_streets_view_result_txt() {
        let mut test_wsgi = TestWsgi::new();
        let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");
        assert_eq!(result, "Only In Ref utca\n");
    }

    /// Tests the missing streets page: the chkl output.
    #[test]
    fn test_missing_streets_view_result_chkl() {
        let mut test_wsgi = TestWsgi::new();
        let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.chkl");
        assert_eq!(result, "[ ] Only In Ref utca\n");
    }

    /// Tests the missing streets page: the txt output, no osm streets case.
    #[test]
    fn test_missing_streets_view_result_txt_no_osm_streets() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);

        let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

        assert_eq!(result, "No existing streets");
    }

    /// Tests the missing streets page: the txt output, no ref streets case.
    #[test]
    fn test_missing_streets_view_result_txt_no_ref_streets() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);

        let result = test_wsgi.get_txt_for_path("/missing-streets/gazdagret/view-result.txt");

        assert_eq!(result, "No reference streets");
    }

    /// Tests the missing streets page: if the view-query output is well-formed.
    #[test]
    fn test_missing_streets_view_query_well_formed() {
        let mut test_wsgi = TestWsgi::new();

        let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-query");

        let results = TestWsgi::find_all(&root, "body/pre");

        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: the update-result output.
    #[test]
    fn test_missing_streets_update_result() {
        let mut test_wsgi = TestWsgi::new();
        let streets_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/streets-reference-gazdagret.lst", &streets_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/update-result");

        let mut guard = streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);

        let results = TestWsgi::find_all(&root, "body/div[@id='update-success']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the missing streets page: the view-turbo output.
    #[test]
    fn test_missing_streets_view_turbo() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/missing-streets/gazdagret/view-turbo");

        let results = TestWsgi::find_all(&root, "body/pre");

        assert_eq!(results.len(), 1);

        assert_eq!(results[0].contains("OSM Name 1"), true);
        // This is silenced with `show-refstreet: false`.
        assert_eq!(results[0].contains("OSM Name 2"), false);
    }

    /// Tests handle_main(): if the output is well-formed.
    #[test]
    fn test_main_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_main(): the case when the URL is empty (should give the main page).
    #[test]
    fn test_main_no_path() {
        let request = rouille::Request::fake_http("GET", "", vec![], vec![]);
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = areas::Relations::new(&ctx).unwrap();
        let ret = webframe::get_request_uri(&request, &ctx, &mut relations).unwrap();
        assert_eq!(ret, "");
    }

    /// Tests handle_main(): if the /osm/filter-for/incomplete output is well-formed.
    #[test]
    fn test_main_filter_for_incomplete_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/filter-for/incomplete");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_main(): if the /osm/filter-for/everything output is well-formed.
    #[test]
    fn test_main_filter_for_everything_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/filter-for/everything");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_main(): if the /osm/filter-for/refcounty output is well-formed.
    #[test]
    fn test_main_filter_for_refcounty_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/filter-for/refcounty/01/whole-county");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_main(): if the /osm/filter-for/refcounty output is well-formed.
    #[test]
    fn test_main_filter_for_refcounty_no_refsettlement() {
        let mut test_wsgi = TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/filter-for/refcounty/67/whole-county");
        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_main(): the /osm/filter-for/refcounty/<value>/refsettlement/<value> output.
    #[test]
    fn test_main_filter_for_refcounty_refsettlement() {
        let mut test_wsgi = TestWsgi::new();

        let root = test_wsgi.get_dom_for_path("/filter-for/refcounty/01/refsettlement/011");

        let mut results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
        results = TestWsgi::find_all(&root, "body/table/tr");
        // header + 5 relations, was just 1 when the filter was buggy
        assert_eq!(results.len(), 6);
    }

    /// Tests handle_main(): the /osm/filter-for/relations/... output
    #[test]
    fn test_main_filter_for_relations() {
        let mut test_wsgi = TestWsgi::new();

        let root = test_wsgi.get_dom_for_path("/filter-for/relations/44,45");

        let mut results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
        results = TestWsgi::find_all(&root, "body/table/tr");
        // header + the two requested relations
        assert_eq!(results.len(), 3);
    }

    /// Tests handle_main(): the /osm/filter-for/relations/ output.
    #[test]
    fn test_main_filter_for_relations_empty() {
        let mut test_wsgi = TestWsgi::new();

        let root = test_wsgi.get_dom_for_path("/filter-for/relations/");

        let mut results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
        results = TestWsgi::find_all(&root, "body/table/tr");
        // header + no requested relations
        assert_eq!(results.len(), 1);
    }

    /// Tests application(): the error handling case.
    #[test]
    fn test_application_error() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let unit = context::tests::TestUnit::new();
        let unit_arc: Arc<dyn context::Unit> = Arc::new(unit);
        ctx.set_unit(&unit_arc);
        let bytes: Vec<u8> = Vec::new();

        let abspath: String = "/".into();
        let rouille_headers: Vec<(String, String)> = Vec::new();
        let request = rouille::Request::fake_http("GET", abspath, rouille_headers, bytes);
        let response = application(&request, &ctx);
        let mut data = Vec::new();
        let (mut reader, _size) = response.data.into_reader_and_size();
        reader.read_to_end(&mut data).unwrap();

        assert_eq!(response.status_code, 500);
        let headers_map: HashMap<_, _> = response.headers.into_iter().collect();
        assert_eq!(headers_map["Content-type"], "text/html; charset=utf-8");
        assert_eq!(data.is_empty(), false);
        let output = String::from_utf8(data).unwrap();
        assert_eq!(output.contains("TestError"), true);
    }

    /// Tests /osm/webhooks/: /osm/webhooks/github.
    #[test]
    fn test_webhooks_github() {
        let root = serde_json::json!({"ref": "refs/heads/master"});
        let payload = serde_json::to_string(&root).unwrap();
        let query_string: String = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("payload", &payload)
            .finish();
        let mut test_wsgi = TestWsgi::new();
        test_wsgi.bytes = query_string.as_bytes().to_vec();
        let expected_args = format!("make -C {} deploy", test_wsgi.ctx.get_abspath(""));
        let outputs: HashMap<_, _> = vec![(expected_args, "".to_string())].into_iter().collect();
        let subprocess = context::tests::TestSubprocess::new(&outputs);
        let subprocess_arc: Arc<dyn context::Subprocess> = Arc::new(subprocess);
        test_wsgi.ctx.set_subprocess(&subprocess_arc);

        test_wsgi.get_dom_for_path("/webhooks/github");

        let subprocess = subprocess_arc
            .as_any()
            .downcast_ref::<context::tests::TestSubprocess>()
            .unwrap();
        assert_eq!(subprocess.get_runs().is_empty(), false);
        assert_eq!(subprocess.get_exits(), &[1]);
    }

    /// Tests /osm/webhooks/: /osm/webhooks/github, the case when a non-master branch is updated.
    #[test]
    fn test_webhooks_github_branch() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let outputs: HashMap<String, String> = HashMap::new();
        let subprocess = context::tests::TestSubprocess::new(&outputs);
        let subprocess_arc: Arc<dyn context::Subprocess> = Arc::new(subprocess);
        ctx.set_subprocess(&subprocess_arc);
        let root = serde_json::json!({"ref": "refs/heads/stable"});
        let payload = serde_json::to_string(&root).unwrap();
        let query_string: String = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("payload", &payload)
            .finish();
        let buf = query_string.as_bytes().to_vec();
        let request = rouille::Request::fake_http("GET", "/", vec![], buf);

        webframe::handle_github_webhook(&request, &ctx).unwrap();

        let subprocess = subprocess_arc
            .as_any()
            .downcast_ref::<context::tests::TestSubprocess>()
            .unwrap();
        assert_eq!(subprocess.get_runs().is_empty(), true);
    }

    /// Tests handle_stats().
    #[test]
    fn test_handle_stats() {
        let mut test_wsgi = TestWsgi::new();

        let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/");

        let results = TestWsgi::find_all(&root, "body/h2");
        // 8 chart types + note
        assert_eq!(results.len(), 9);
    }

    /// Tests /osm/static/: the css case.
    #[test]
    fn test_static_css() {
        let mut test_wsgi = TestWsgi::new();

        let result = test_wsgi.get_css_for_path("/static/osm.min.css");

        assert_eq!(result.ends_with("}"), true);
    }

    /// Tests /osm/static/: the plain text case.
    #[test]
    fn test_static_text() {
        let mut test_wsgi = TestWsgi::new();

        let result = test_wsgi.get_txt_for_path("/robots.txt");

        assert_eq!(result, "User-agent: *\n");
    }

    /// Tests handle_stats_cityprogress(): if the output is well-formed.
    #[test]
    fn test_handle_stats_cityprogress_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let time = context::tests::make_test_time();
        let time_arc: Arc<dyn context::Time> = Arc::new(time);
        test_wsgi.ctx.set_time(&time_arc);

        let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/cityprogress");

        let results = TestWsgi::find_all(&root, "body/table/tr");
        // header; also budapest_11 is both in ref and osm
        assert_eq!(results.len(), 2);
    }

    /// Tests handle_stats_zipprogress(): if the output is well-formed.
    #[test]
    fn test_handle_stats_zipprogress_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        let time = context::tests::make_test_time();
        let time_arc: Arc<dyn context::Time> = Arc::new(time);
        test_wsgi.ctx.set_time(&time_arc);

        let zips_value = context::tests::TestFileSystem::make_file();
        zips_value.borrow_mut().write_all(b"1111\t10\n").unwrap();
        let files = context::tests::TestFileSystem::make_files(
            &test_wsgi.ctx,
            &[("workdir/stats/2020-05-10.zipcount", &zips_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.ctx.set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/zipprogress");

        let results = TestWsgi::find_all(&root, "body/table/tr");
        // header; also 1111 is both in ref and osm
        assert_eq!(results.len(), 2);
    }

    /// Tests handle_invalid_refstreets(): if the output is well-formed.
    #[test]
    fn test_handle_invalid_refstreets_well_formed() {
        let mut test_wsgi = TestWsgi::new();

        let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/invalid-relations");

        let results = TestWsgi::find_all(&root, "body/h1/a");
        assert_eq!(results.is_empty(), false);
    }

    /// Tests handle_invalid_refstreets(): error handling when osm street list is missing for a relation.
    #[test]
    fn test_handle_invalid_refstreets_no_osm_sreets() {
        let mut test_wsgi = TestWsgi::new();
        let mut relations = areas::Relations::new(&test_wsgi.ctx).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.ctx.set_file_system(&file_system_arc);

        let root = test_wsgi.get_dom_for_path("/housenumber-stats/hungary/invalid-relations");

        let results = TestWsgi::find_all(&root, "body");
        assert_eq!(results.is_empty(), false);
    }

    /// Tests the not-found page: if the output is well-formed.
    #[test]
    fn test_not_found_well_formed() {
        let mut test_wsgi = TestWsgi::new();
        test_wsgi.absolute_path = true;
        test_wsgi.expected_status = 404;

        let root = test_wsgi.get_dom_for_path("/asdf");

        let results = TestWsgi::find_all(&root, "body/h1");

        assert_eq!(results.is_empty(), false);
    }

    /// Tests gzip compress case.
    #[test]
    fn test_compress() {
        let mut test_wsgi = TestWsgi::new();
        test_wsgi.gzip_compress = true;

        let root = test_wsgi.get_dom_for_path("/");

        let results = TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }
}
