/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
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
use crate::stats;
use crate::util;
use crate::webframe;
use crate::wsgi_additional;
use crate::wsgi_json;
use crate::yattag;
use anyhow::Context;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;

/// Gets the update date string of a file.
fn get_last_modified(ctx: &context::Context, path: &str) -> anyhow::Result<String> {
    webframe::format_timestamp(&util::get_mtime(ctx, path))
}

/// Gets the update date of streets for a relation.
fn get_streets_last_modified(
    ctx: &context::Context,
    relation: &areas::Relation<'_>,
) -> anyhow::Result<String> {
    let format = tr("{0} (osm), {1} (areas)");
    let osm = webframe::format_timestamp(&stats::get_sql_mtime(
        ctx,
        &format!("streets/{}/osm-base", relation.get_name()),
    )?)?;
    let areas = webframe::format_timestamp(&stats::get_sql_mtime(
        ctx,
        &format!("streets/{}/areas-base", relation.get_name()),
    )?)?;
    Ok(format.replace("{0}", &osm).replace("{1}", &areas))
}

/// Expected request_uri: e.g. /osm/streets/ormezo/view-query.
fn handle_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().context("no action")?;
    let relation_name = tokens.next_back().context("no relation_name")?;

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
        // Old style: CSV.
        let query = relation.get_osm_streets_query()?;
        match overpass_query::overpass_query(ctx, &query) {
            Ok(buf) => {
                relation.get_files().write_osm_streets(ctx, &buf)?;
                let streets = relation.get_config().should_check_missing_streets();
                if streets != "only" {
                    doc.text(&tr("Update successful: "));
                    let prefix = ctx.get_ini().get_uri_prefix();
                    let link = format!("{prefix}/missing-housenumbers/{relation_name}/view-result");
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
        // New style: JSON.
        let query = relation.get_osm_streets_json_query()?;
        match overpass_query::overpass_query(ctx, &query) {
            Ok(buf) => {
                relation.get_files().write_osm_json_streets(ctx, &buf)?;
            }
            Err(err) => {
                doc.append_value(util::handle_overpass_error(ctx, &err.to_string()).get_value());
            }
        }
    } else {
        // assume view-result
        let mut csv: String =
            String::from("@id\tname\thighway\tservice\tsurface\tleisure\t@type\n");
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn.prepare("select osm_id, name, highway, service, surface, leisure, osm_type from osm_streets where relation = ?1")?;
        let mut rows = stmt.query([&relation_name])?;
        while let Some(row) = rows.next()? {
            let osm_id: String = row.get(0).unwrap();
            let name: String = row.get(1).unwrap();
            let highway: String = row.get(2).unwrap();
            let service: String = row.get(3).unwrap();
            let surface: String = row.get(4).unwrap();
            let leisure: String = row.get(5).unwrap();
            let osm_type: String = row.get(6).unwrap();
            csv += &format!(
                "{osm_id}\t{name}\t{highway}\t{service}\t{surface}\t{leisure}\t{osm_type}\n"
            );
        }
        let mut read = csv.as_bytes();
        let table = util::tsv_to_list(&mut read)?;
        doc.append_value(util::html_table_from_list(&table).get_value());
    }

    doc.append_value(webframe::get_footer(&get_streets_last_modified(ctx, &relation)?).get_value());
    Ok(doc)
}

/// Gets the update date of house numbers for a relation.
fn get_housenumbers_last_modified(
    ctx: &context::Context,
    relation: &areas::Relation<'_>,
) -> anyhow::Result<String> {
    get_last_modified(ctx, &relation.get_files().get_osm_housenumbers_path())
}

/// Expected request_uri: e.g. /osm/street-housenumbers/ormezo/view-query.
fn handle_street_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().context("no action")?;
    let relation_name = tokens.next_back().context("no relation_name")?;

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

    let prefix = ctx.get_ini().get_uri_prefix();
    if action == "view-query" {
        let pre = doc.tag("pre", &[]);
        pre.text(&relation.get_osm_housenumbers_query()?);
    } else if action == "update-result" {
        let query = relation.get_osm_housenumbers_query()?;
        match overpass_query::overpass_query(ctx, &query) {
            Ok(buf) => {
                relation.get_files().write_osm_housenumbers(ctx, &buf)?;
                doc.text(&tr("Update successful: "));
                let link = format!("{prefix}/missing-housenumbers/{relation_name}/view-result");
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
            doc.append_value(
                util::html_table_from_list(&util::tsv_to_list(&mut read)?).get_value(),
            );
        }
    }

    let date = get_housenumbers_last_modified(ctx, &relation)?;
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-turbo.
fn missing_housenumbers_view_turbo(
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;

    let doc = yattag::Doc::new();
    let mut relation = relations.get_relation(relation_name)?;
    let ongoing_streets = relation.get_missing_housenumbers()?.ongoing_streets;
    let mut streets: Vec<String> = Vec::new();
    for result in ongoing_streets {
        streets.push(result.street.get_osm_name().into());
    }
    let query = areas::make_turbo_query_for_streets(&relation, &streets);

    let pre = doc.tag("pre", &[]);
    pre.text(&query);
    Ok(doc)
}

/// Expected request uri: /osm/missing-housenumbers/ormezo/view-lints.
fn missing_housenumbers_view_lints(
    ctx: &context::Context,
    relation: &mut areas::Relation<'_>,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();

    // Update lints if they are outdated.
    cache::get_missing_housenumbers_json(relation)?;

    let mut table: Vec<Vec<yattag::Doc>> = Vec::new();
    let mut count = 0;
    {
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn
        .prepare("select street_name, source, housenumber, reason, object_id, object_type from relation_lints where relation_name = ?1")?;
        let mut lints = stmt.query([relation.get_name()])?;
        {
            let cells: Vec<yattag::Doc> = vec![
                yattag::Doc::from_text(&tr("Street")),
                yattag::Doc::from_text(&tr("Source")),
                yattag::Doc::from_text(&tr("Housenumber")),
                yattag::Doc::from_text(&tr("Reason")),
                yattag::Doc::from_text(&tr("Identifier")),
                yattag::Doc::from_text(&tr("Type")),
            ];
            table.push(cells);
        }
        while let Some(lint) = lints.next()? {
            let mut cells: Vec<yattag::Doc> = Vec::new();
            let street: String = lint.get(0).unwrap();
            let source: areas::RelationLintSource = lint.get(1).unwrap();
            let source_string = match source {
                areas::RelationLintSource::Range => tr("street ranges"),
                areas::RelationLintSource::Invalid => tr("invalid housenumbers"),
            };
            let housenumber: String = lint.get(2).unwrap();
            let reason: areas::RelationLintReason = lint.get(3).unwrap();
            let id: String = lint.get(4).unwrap();
            let object_type: String = lint.get(5).unwrap();
            let reason_string = match reason {
                areas::RelationLintReason::CreatedInOsm => tr("created in OSM"),
                areas::RelationLintReason::DeletedFromRef => tr("deleted from reference"),
                areas::RelationLintReason::OutOfRange => tr("out of range"),
            };
            cells.push(yattag::Doc::from_text(&street));
            cells.push(yattag::Doc::from_text(&source_string));
            cells.push(yattag::Doc::from_text(&housenumber));
            {
                let doc = yattag::Doc::new();
                let div = doc.tag("div", &[("data-value", &reason.to_string())]);
                div.text(&reason_string);
                cells.push(doc);
            }
            if id != "0" {
                let cell = yattag::Doc::new();
                let href = format!("https://www.openstreetmap.org/{}/{}", object_type, id,);
                {
                    let a = cell.tag("a", &[("href", &href), ("target", "_blank")]);
                    a.text(&id.to_string());
                }
                cells.push(cell);
            } else {
                cells.push(yattag::Doc::new());
            }
            cells.push(yattag::Doc::from_text(&object_type));
            table.push(cells);
            count += 1;
        }
    }
    {
        let p = doc.tag("p", &[]);
        p.text(
            &tr("The below {0} filters for this relation are probably no longer necessary.")
                .replace("{0}", &count.to_string()),
        );
    }
    doc.append_value(util::html_table_from_list(&table).get_value());

    Ok(doc)
}

/// The actual HTML part of missing_housenumbers_view_res().
fn missing_housenumbers_view_res_html(
    ctx: &context::Context,
    relation: &mut areas::Relation<'_>,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let (todo_street_count, todo_count, done_count, percent, table) = relation
        .write_missing_housenumbers()
        .context("write_missing_housenumbers() failed")?;

    {
        let p = doc.tag("p", &[]);
        let prefix = ctx.get_ini().get_uri_prefix();
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
                    &tr("https://vmiklos.hu/osm-gimmisn/usage.html#filtering-out-incorrect-information"),
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
                    &format!("{prefix}/missing-housenumbers/{relation_name}/view-turbo"),
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
                    &format!("{prefix}/missing-housenumbers/{relation_name}/view-result.txt"),
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
                    &format!("{prefix}/missing-housenumbers/{relation_name}/view-result.chkl"),
                )],
            );
            a.text(&tr("Checklist format"));
        }
        doc.stag("br");
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("{prefix}/missing-housenumbers/{relation_name}/view-lints"),
                )],
            );
            a.text(&tr("View lints"));
        }
    }

    doc.append_value(util::html_table_from_list(&table).get_value());
    if let Ok((osm_invalids, ref_invalids)) = relation.get_invalid_refstreets() {
        doc.append_value(
            util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value(),
        );
    }
    doc.append_value(
        util::invalid_filter_keys_to_html(&relation.get_invalid_filter_keys()?).get_value(),
    );

    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.
fn missing_housenumbers_view_res(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;

    let doc: yattag::Doc;
    let mut relation = relations.get_relation(relation_name)?;
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
        let ret = missing_housenumbers_view_res_html(ctx, &mut relation);
        doc = ret.context("get_missing_housenumbers_html() failed")?;
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-streets/budapest_11/view-result.
fn missing_streets_view_result(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;
    let relation = relations.get_relation(relation_name)?;

    let doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix();
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
    streets.sort_by_key(|i| util::get_sort_key(i));
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
                .replace("{1}", &util::format_percent(percent)?),
        );
        p.stag("br", &[]);
        {
            let a = p.tag(
                "a",
                &[(
                    "href",
                    &format!("{prefix}/missing-streets/{relation_name}/view-turbo"),
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
                    &format!("{prefix}/missing-streets/{relation_name}/view-result.txt"),
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
                    &format!("{prefix}/missing-streets/{relation_name}/view-result.chkl"),
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
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<String> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;
    let mut relation = relations.get_relation(relation_name)?;

    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_streets_path())
    {
        return Ok(tr("No existing streets"));
    }

    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_osm_housenumbers_path())
    {
        return Ok(tr("No existing house numbers"));
    }

    if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_housenumbers_path())
    {
        return Ok(tr("No reference house numbers"));
    }

    let json = cache::get_missing_housenumbers_json(&mut relation)?;
    let missing_housenumbers: areas::MissingHousenumbers = serde_json::from_str(&json)?;
    let ongoing_streets = missing_housenumbers.ongoing_streets;
    let mut table: Vec<String> = Vec::new();
    for result in ongoing_streets {
        let range_list = util::get_housenumber_ranges(&result.house_numbers);
        let mut range_strings: Vec<String> = range_list
            .iter()
            .map(|i| i.get_lowercase_number())
            .collect();
        // Street name, only_in_reference items.
        let row: String = if !relation
            .get_config()
            .get_street_is_even_odd(result.street.get_osm_name())
        {
            range_strings.sort_by_key(|i| util::split_house_number(i));
            format!(
                "{}\t[{}]",
                result.street.get_osm_name(),
                range_strings.join(", ")
            )
        } else {
            let elements = util::format_even_odd(&range_list);
            format!(
                "{}\t[{}]",
                result.street.get_osm_name(),
                elements.join("], [")
            )
        };
        table.push(row);
    }
    table.sort_by_key(|i| util::get_sort_key(i));
    let output = table.join("\n");
    Ok(output)
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-result.chkl.
fn missing_housenumbers_view_chkl(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;
    let mut relation = relations.get_relation(relation_name)?;

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
        let ongoing_streets = relation.get_missing_housenumbers()?.ongoing_streets;

        let mut table: Vec<String> = Vec::new();
        for result in ongoing_streets {
            let range_list = util::get_housenumber_ranges(&result.house_numbers);
            if !relation
                .get_config()
                .get_street_is_even_odd(result.street.get_osm_name())
            {
                let mut result_sorted: Vec<String> =
                    range_list.iter().map(|i| i.get_number().into()).collect();
                result_sorted.sort_by_key(|i| util::split_house_number(i));
                let row = format!(
                    "[ ] {} [{}]",
                    result.street.get_osm_name(),
                    result_sorted.join(", ")
                );
                table.push(row);
            } else {
                let elements = util::format_even_odd(&range_list);
                if elements.len() > 1 && range_list.len() > 20 {
                    for element in elements {
                        let row = format!("[ ] {} [{}]", result.street.get_osm_name(), element);
                        table.push(row);
                    }
                } else {
                    let row = format!(
                        "[ ] {} [{}]",
                        result.street.get_osm_name(),
                        elements.join("], [")
                    );
                    table.push(row);
                }
            }
        }
        table.sort_by_key(|i| util::get_sort_key(i));
        output = table.join("\n");
    }
    Ok((output, relation_name.into()))
}

/// Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-result.txt.
fn missing_streets_view_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
    chkl: bool,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;
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
        todo_streets.sort_by_key(|i| util::get_sort_key(i));
        let mut lines: Vec<String> = Vec::new();
        for street in todo_streets {
            if chkl {
                lines.push(format!("[ ] {street}\n"));
            } else {
                lines.push(format!("{street}\n"));
            }
        }
        output = lines.join("");
    }
    Ok((output, relation_name.into()))
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/update-result.
fn missing_housenumbers_update(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    relation_name: &str,
) -> anyhow::Result<yattag::Doc> {
    let references = ctx.get_ini().get_reference_housenumber_paths()?;
    let relation = relations.get_relation(relation_name)?;
    relation.write_ref_housenumbers(&references)?;
    let doc = yattag::Doc::new();
    doc.text(&tr("Update successful: "));
    let prefix = ctx.get_ini().get_uri_prefix();
    let link = format!("{prefix}/missing-housenumbers/{relation_name}/view-result");
    doc.append_value(util::gen_link(&link, &tr("View missing house numbers")).get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-streets/ujbuda/update-result.
fn missing_streets_update(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
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
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    name: &str,
) -> anyhow::Result<String> {
    let relation = relations.get_relation(name)?;
    let t_ref = util::get_mtime(ctx, &relation.get_files().get_ref_housenumbers_path());
    let t_housenumbers = util::get_mtime(ctx, &relation.get_files().get_osm_housenumbers_path());
    webframe::format_timestamp(std::cmp::max(&t_ref, &t_housenumbers))
}

/// Expected request_uri: e.g. /osm/missing-housenumbers/ormezo/view-[result|query].
fn handle_missing_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().context("no action")?;
    let relation_name = tokens.next_back().context("no relation_name")?;
    let mut date = "".into();

    let mut relation = relations.get_relation(relation_name)?;
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
        date = get_last_modified(ctx, &relation.get_files().get_ref_housenumbers_path())?;
    } else if action == "update-result" {
        doc.append_value(missing_housenumbers_update(ctx, relations, relation_name)?.get_value())
    } else if action == "view-lints" {
        doc.append_value(missing_housenumbers_view_lints(ctx, &mut relation)?.get_value())
    } else {
        // assume view-result
        let ret = missing_housenumbers_view_res(ctx, relations, request_uri);
        doc.append_value(
            ret.context("missing_housenumbers_view_res() failed")?
                .get_value(),
        )
    }

    if date.is_empty() {
        date = ref_housenumbers_last_modified(ctx, relations, relation_name)?;
    }
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/missing-streets/ormezo/view-turbo.
fn missing_streets_view_turbo(
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;

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

/// Expected request_uri: e.g. /osm/missing-streets/ujbuda/view-[result|query].
fn handle_missing_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().context("no action")?;
    let relation_name = tokens.next_back().context("no relation_name")?;

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

    doc.append_value(webframe::get_footer(&get_streets_last_modified(ctx, &relation)?).get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-[result|query].
fn handle_additional_streets(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let action = tokens.next_back().context("no action")?;
    let relation_name = tokens.next_back().context("no relation_name")?;

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

    doc.append_value(webframe::get_footer(&get_streets_last_modified(ctx, &relation)?).get_value());
    Ok(doc)
}

/// Gets the update date for missing/additional housenumbers.
fn relation_housenumbers_get_last_modified(
    ctx: &context::Context,
    relation: &areas::Relation<'_>,
) -> anyhow::Result<String> {
    let t_ref = util::get_mtime(ctx, &relation.get_files().get_ref_housenumbers_path());
    let t_osm = util::get_mtime(ctx, &relation.get_files().get_osm_housenumbers_path());
    webframe::format_timestamp(std::cmp::max(&t_ref, &t_osm))
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/ujbuda/view-[result|query].
fn handle_additional_housenumbers(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    let _action = tokens.next_back();
    let relation_name = tokens.next_back().context("no relation_name")?;

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

    let date = relation_housenumbers_get_last_modified(ctx, &relation)?;
    doc.append_value(webframe::get_footer(&date).get_value());
    Ok(doc)
}

/// Handles the house number percent part of the main page.
fn handle_main_housenr_percent(
    ctx: &context::Context,
    relation: &areas::Relation<'_>,
) -> anyhow::Result<(yattag::Doc, f64)> {
    let prefix = ctx.get_ini().get_uri_prefix();
    let url = format!(
        "{}/missing-housenumbers/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut percent: Option<f64> = None;
    if relation.has_osm_housenumber_coverage()? {
        let string = relation.get_osm_housenumber_coverage()?;
        percent = Some(string.parse::<f64>().context("parse to f64 failed")?);
    }

    let doc = yattag::Doc::new();
    if let Some(percent) = percent {
        let date = webframe::format_timestamp(&relation.get_osm_housenumber_coverage_mtime()?)?;
        let strong = doc.tag("strong", &[]);
        let a = strong.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        let percent_string =
            util::format_percent(percent).context("util::format_percent() failed")?;
        a.text(&percent_string);
        return Ok((doc, percent));
    }

    let strong = doc.tag("strong", &[]);
    let a = strong.tag("a", &[("href", &url)]);
    a.text(&tr("missing house numbers"));
    Ok((doc, 0_f64))
}

/// Handles the street percent part of the main page.
fn handle_main_street_percent(
    ctx: &context::Context,
    relation: &areas::Relation<'_>,
) -> anyhow::Result<(yattag::Doc, f64)> {
    let prefix = ctx.get_ini().get_uri_prefix();
    let url = format!(
        "{}/missing-streets/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut percent: Option<f64> = None;
    if relation.has_osm_street_coverage()? {
        let string = relation.get_osm_street_coverage()?;
        percent = Some(string.parse::<f64>().context("parse to f64 failed")?);
    }

    let doc = yattag::Doc::new();
    if let Some(percent) = percent {
        let date = webframe::format_timestamp(&relation.get_osm_street_coverage_mtime()?)?;
        let strong = doc.tag("strong", &[]);
        let a = strong.tag(
            "a",
            &[
                ("href", &url),
                ("title", &format!("{} {}", tr("updated"), date)),
            ],
        );
        let percent_string =
            util::format_percent(percent).context("util::format_percent() failed")?;
        a.text(&percent_string);
        return Ok((doc, percent));
    }

    let strong = doc.tag("strong", &[]);
    let a = strong.tag("a", &[("href", &url)]);
    a.text(&tr("missing streets"));
    Ok((doc, 0_f64))
}

/// Handles the street additional count part of the main page.
fn handle_main_street_additional_count(
    ctx: &context::Context,
    relation: &areas::Relation<'_>,
) -> anyhow::Result<yattag::Doc> {
    let prefix = ctx.get_ini().get_uri_prefix();
    let url = format!(
        "{}/additional-streets/{}/view-result",
        prefix,
        relation.get_name()
    );
    let mut additional_count: String = "".into();
    let files = relation.get_files();
    let path = files.get_streets_additional_count_path();
    if ctx.get_file_system().path_exists(&path) {
        additional_count = ctx.get_file_system().read_to_string(&path)?;
    }

    let doc = yattag::Doc::new();
    if !additional_count.is_empty() {
        let date = get_last_modified(ctx, &path)?;
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
    relation: &areas::Relation<'_>,
) -> anyhow::Result<yattag::Doc> {
    if !relation.get_config().should_check_additional_housenumbers() {
        return Ok(yattag::Doc::new());
    }

    let prefix = ctx.get_ini().get_uri_prefix();
    let url = format!(
        "{}/additional-housenumbers/{}/view-result",
        prefix,
        relation.get_name()
    );
    let files = relation.get_files();
    let additional_count = get_housenr_additional_count(ctx, files)?;

    let doc = yattag::Doc::new();
    if !additional_count.is_empty() {
        let date = get_last_modified(ctx, &files.get_housenumbers_additional_count_path())?;
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
fn filter_for_everything(_complete: bool, _relation: &areas::Relation<'_>) -> bool {
    true
}

/// Filters out complete items.
fn filter_for_incomplete(complete: bool, _relation: &areas::Relation<'_>) -> bool {
    !complete
}

type RelationFilter = dyn Fn(bool, &areas::Relation<'_>) -> bool;

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
        for relation in relation_filter.split(',') {
            if let Ok(val) = relation.parse() {
                relations.push(val);
            }
        }
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
fn setup_main_filter_for(request_uri: &str) -> anyhow::Result<(Box<RelationFilter>, String)> {
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
        refcounty = filters.get("refcounty").context("no refcounty")?;
        filter_for = create_filter_for_refcounty_refsettlement(
            filters.get("refcounty").context("no refcounty")?,
            filters.get("refsettlement").context("no refsettlement")?,
        );
    } else if filters.contains_key("refcounty") {
        // /osm/filter-for/refcounty/<value>/whole-county
        refcounty = filters.get("refcounty").context("no refcounty")?;
        filter_for = create_filter_for_refcounty(refcounty);
    } else if filters.contains_key("relations") {
        // /osm/filter-for/relations/<id1>,<id2>
        let relations = filters.get("relations").context("no relations")?;
        filter_for = create_filter_for_relations(relations);
    }
    Ok((filter_for, refcounty.into()))
}

/// Handles one refcounty in the filter part of the main wsgi page.
///
/// refcounty_id is the county we filter for.
/// refcounty is one item in the county list.
fn handle_main_filters_refcounty(
    ctx: &context::Context,
    relations: &areas::Relations<'_>,
    refcounty_id: &str,
    refcounty: &str,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let name = relations.refcounty_get_name(refcounty);
    if name.is_empty() {
        return Ok(doc);
    }

    let prefix = ctx.get_ini().get_uri_prefix();
    {
        let a = doc.tag(
            "a",
            &[(
                "href",
                &format!("{prefix}/filter-for/refcounty/{refcounty}/whole-county"),
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
                        "{prefix}/filter-for/refcounty/{refcounty}/refsettlement/{refsettlement_id}"
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
    relations: &mut areas::Relations<'_>,
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
    let prefix = ctx.get_ini().get_uri_prefix();
    {
        let a = doc.tag("a", &[("href", &format!("{prefix}/filter-for/everything"))]);
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
    relations: &mut areas::Relations<'_>,
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
        let (cell, percent) = handle_main_housenr_percent(ctx, &relation)
            .context("handle_main_housenr_percent() failed")?;
        let doc = yattag::Doc::new();
        doc.append_value(cell.get_value());
        row.push(doc);
        complete &= percent >= 100_f64;

        row.push(handle_main_housenr_additional_count(ctx, &relation)?);
    } else {
        row.push(yattag::Doc::new());
        row.push(yattag::Doc::new());
    }

    if streets != "no" {
        let (cell, percent) = handle_main_street_percent(ctx, &relation)?;
        row.push(cell);
        complete &= percent >= 100_f64;
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
    relations: &mut areas::Relations<'_>,
) -> anyhow::Result<yattag::Doc> {
    let (filter_for, refcounty) = setup_main_filter_for(request_uri)?;

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
        let row = handle_main_relation(ctx, relations, &filter_for, &relation_name)
            .context("handle_main_relation() failed")?;
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
                &tr("https://vmiklos.hu/osm-gimmisn/usage.html#how-to-add-a-new-area"),
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
    let mut function = "";
    let mut relation_name = "";
    if tokens.len() > 3 {
        function = &tokens[2];
        relation_name = &tokens[3];
    }
    match function {
        "missing-housenumbers" => format!(
            " - {}",
            tr("{0} missing house numbers").replace("{0}", relation_name)
        ),
        "missing-streets" => format!(" - {} {}", relation_name, tr("missing streets")),
        "street-housenumbers" => format!(" - {} {}", relation_name, tr("existing house numbers")),
        "streets" => format!(" - {} {}", relation_name, tr("existing streets")),
        _ => "".into(),
    }
}

/// Produces the <head> tag and its contents.
fn write_html_head(ctx: &context::Context, doc: &yattag::Tag, title: &str) -> anyhow::Result<()> {
    let prefix = ctx.get_ini().get_uri_prefix();
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
            ("href", &format!("{prefix}/favicon.ico")),
        ],
    );
    head.stag(
        "link",
        &[
            ("rel", "icon"),
            ("type", "image/svg+xml"),
            ("sizes", "any"),
            ("href", &format!("{prefix}/favicon.svg")),
        ],
    );

    let css_path = ctx.get_abspath("target/browser/osm.min.css");
    if ctx.get_file_system().path_exists(&css_path) {
        let stream = ctx.get_file_system().open_read(&css_path)?;
        let mut buf: Vec<u8> = Vec::new();
        let mut guard = stream.borrow_mut();
        guard.read_to_end(&mut buf)?;
        let contents = String::from_utf8(buf)?;
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
            ("src", &format!("{prefix}/static/bundle.js")),
        ],
    );
    drop(script);
    Ok(())
}

/// Dispatches GPX requests based on their URIs.
fn our_application_gpx(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<rouille::Response> {
    let content_type = "text/gpx+xml; charset=utf-8";
    let mut headers: webframe::Headers = Vec::new();
    // assume prefix + "/additional-streets/"
    let (output, relation_name) =
        wsgi_additional::additional_streets_view_gpx(ctx, relations, request_uri)
            .context("additional_streets_view_gpx() failed")?;
    headers.push((
        "Content-Disposition".into(),
        format!(r#"attachment;filename="{relation_name}.gpx""#).into(),
    ));
    let data = output.as_bytes().to_vec();
    headers.push(("Content-type".into(), content_type.into()));
    Ok(webframe::make_response(200_u16, headers, data))
}

/// Dispatches plain text requests based on their URIs.
fn our_application_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<rouille::Response> {
    let mut content_type = "text/plain; charset=utf-8";
    let mut headers: webframe::Headers = Vec::new();
    let prefix = ctx.get_ini().get_uri_prefix();
    let mut chkl = false;
    let tokens: Vec<_> = request_uri.split('.').collect();
    if let Some((last, _elements)) = tokens.split_last() {
        chkl = last == &"chkl";
    }
    let data: Vec<u8>;
    if request_uri.starts_with(&format!("{prefix}/missing-streets/")) {
        let (output, relation_name) = missing_streets_view_txt(ctx, relations, request_uri, chkl)?;
        if chkl {
            content_type = "application/octet-stream";
            headers.push((
                "Content-Disposition".into(),
                format!(r#"attachment;filename="{relation_name}.txt""#).into(),
            ));
        }
        data = output.as_bytes().to_vec();
    } else if request_uri.starts_with(&format!("{prefix}/additional-streets/")) {
        let (output, relation_name) =
            wsgi_additional::additional_streets_view_txt(ctx, relations, request_uri, chkl)?;
        if chkl {
            content_type = "application/octet-stream";
            headers.push((
                "Content-Disposition".into(),
                format!(r#"attachment;filename="{relation_name}.txt""#).into(),
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
                format!(r#"attachment;filename="{relation_name}.txt""#).into(),
            ));
            data = output.as_bytes().to_vec();
        } else if request_uri.ends_with("robots.txt") {
            data = ctx
                .get_file_system()
                .read_to_string(&ctx.get_abspath("data/robots.txt"))?
                .as_bytes()
                .into();
        } else {
            // assume txt
            let output = missing_housenumbers_view_txt(ctx, relations, request_uri)?;
            data = output.as_bytes().to_vec();
        }
    }
    headers.push(("Content-type".into(), content_type.into()));
    Ok(webframe::make_response(200_u16, headers, data))
}

type Handler =
    fn(&context::Context, &mut areas::Relations<'_>, &str) -> anyhow::Result<yattag::Doc>;

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
        ret.insert("/lints/".into(), webframe::handle_lints);
        ret
    };
}

/// Decides request_uri matches what handler.
fn get_handler(ctx: &context::Context, request_uri: &str) -> anyhow::Result<Option<Handler>> {
    let prefix = ctx.get_ini().get_uri_prefix();
    for (key, value) in HANDLERS.iter() {
        if request_uri.starts_with(&format!("{prefix}{key}")) {
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
    let language = util::setup_localization(ctx, request.headers());

    let mut relations = areas::Relations::new(ctx).context("areas::Relations::new() failed")?;

    let request_uri = webframe::get_request_uri(request, ctx, &mut relations)
        .context("get_request_uri() failed")?;
    let mut ext: String = "".into();
    let tokens: Vec<_> = request_uri.split('.').collect();
    if let Some((last, _elements)) = tokens.split_last() {
        ext = (*last).into();
    }

    if ext == "txt" || ext == "chkl" {
        return our_application_txt(ctx, &mut relations, &request_uri);
    }

    if ext == "gpx" {
        return our_application_gpx(ctx, &mut relations, &request_uri)
            .context("our_application_gpx() failed");
    }

    let prefix = ctx.get_ini().get_uri_prefix();
    if !(request_uri == "/" || request_uri.starts_with(&prefix)) {
        let doc = webframe::handle_404();
        return Ok(webframe::make_response(
            404_u16,
            vec![("Content-type".into(), "text/html; charset=utf-8".into())],
            doc.get_value().as_bytes().to_vec(),
        ));
    }

    if request_uri.starts_with(&format!("{prefix}/static/"))
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
        write_html_head(ctx, &html, &get_html_title(&request_uri))
            .context("write_html_head() failed")?;

        let body = html.tag("body", &[]);
        let no_such_relation = webframe::check_existing_relation(ctx, &relations, &request_uri)?;
        let handler = get_handler(ctx, &request_uri).context("get_handler() failed")?;
        if !no_such_relation.get_value().is_empty() {
            body.append_value(no_such_relation.get_value());
        } else if let Some(handler) = handler {
            let value = handler(ctx, &mut relations, &request_uri)
                .context("handler() failed")?
                .get_value();
            body.append_value(value);
        } else if request_uri.starts_with(&format!("{prefix}/webhooks/github")) {
            body.append_value(webframe::handle_github_webhook(request, ctx)?.get_value());
        } else {
            let doc =
                handle_main(&request_uri, ctx, &mut relations).context("handle_main() failed")?;
            body.append_value(doc.get_value());
        }
    }

    ctx.get_unit().make_error()?;
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
        Err(err) => webframe::handle_error(request, &format!("{err:?}")),
    }
}

#[cfg(test)]
pub mod tests;
