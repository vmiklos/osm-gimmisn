/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The webframe module provides the header, toolbar and footer code.

use crate::areas;
use crate::context;
use crate::i18n::translate as tr;
use crate::util;
use crate::yattag;
use anyhow::Context;
use git_version::git_version;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::ops::DerefMut;
use std::rc::Rc;

/// Produces the end of the page.
pub fn get_footer(last_updated: &str) -> yattag::Doc {
    let mut items: Vec<yattag::Doc> = Vec::new();
    {
        let doc = yattag::Doc::new();
        doc.text(&tr("Version: "));
        doc.append_value(
            util::git_link(
                git_version!(),
                "https://github.com/vmiklos/osm-gimmisn/commit/",
            )
            .get_value(),
        );
        items.push(doc);
        items.push(yattag::Doc::from_text(&tr(
            "OSM data © OpenStreetMap contributors.",
        )));
        if !last_updated.is_empty() {
            items.push(yattag::Doc::from_text(
                &(tr("Last update: ") + last_updated),
            ));
        }
    }
    let doc = yattag::Doc::new();
    doc.stag("hr");
    {
        let div = doc.tag("div", &[]);
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                div.text(" ¦ ");
            }
            div.append_value(item.get_value());
        }
    }
    doc
}

/// Fills items with function-specific links in the header. Returns the extended list.
fn fill_header_function(
    ctx: &context::Context,
    function: &str,
    relation_name: &str,
    items: &[yattag::Doc],
) -> anyhow::Result<Vec<yattag::Doc>> {
    let mut items: Vec<yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix();
    if function == "missing-housenumbers" {
        // The OSM data source changes much more frequently than the ref one, so add a dedicated link
        // to update OSM house numbers first.
        let doc = yattag::Doc::new();
        {
            let span = doc.tag("span", &[("id", "trigger-street-housenumbers-update")]);
            {
                // TODO consider using HTTP POST here, see
                // https://stackoverflow.com/questions/1367409/how-to-make-button-look-like-a-link
                let a = span.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/street-housenumbers/{}/update-result",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Update from OSM"));
            }
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let span = doc.tag("span", &[("id", "trigger-missing-housenumbers-update")]);
            {
                let a = span.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/missing-housenumbers/{}/update-result",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Update from reference"));
            }
        }
        items.push(doc);
    } else if function == "missing-streets" || function == "additional-streets" {
        // The OSM data source changes much more frequently than the ref one, so add a dedicated link
        // to update OSM streets first.
        let doc = yattag::Doc::new();
        {
            let span = doc.tag("span", &[("id", "trigger-streets-update")]);
            {
                let a = span.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/streets/{}/update-result", prefix, relation_name),
                    )],
                );
                a.text(&tr("Update from OSM"));
            }
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let span = doc.tag("span", &[("id", "trigger-missing-streets-update")]);
            {
                let a = span.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/missing-streets/{}/update-result", prefix, relation_name),
                    )],
                );
                a.text(&tr("Update from reference"));
            }
        }
        items.push(doc);
    } else if function == "street-housenumbers" {
        let doc = yattag::Doc::new();
        {
            let span = doc.tag("span", &[("id", "trigger-street-housenumbers-update")]);
            {
                let a = span.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/street-housenumbers/{}/update-result",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Call Overpass to update"));
            }
        }
        items.push(doc);
        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/street-housenumbers/{}/view-query",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("View query"));
        }
        items.push(doc);
    } else if function == "streets" {
        let doc = yattag::Doc::new();
        {
            let span = doc.tag("span", &[("id", "trigger-streets-update")]);
            {
                let a = span.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/streets/{}/update-result", prefix, relation_name),
                    )],
                );
                a.text(&tr("Call Overpass to update"));
            }
        }
        items.push(doc);
        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("{}/streets/{}/view-query", prefix, relation_name),
                )],
            );
            a.text(&tr("View query"));
        }
        items.push(doc);
    }
    Ok(items)
}

/// Generates the 'missing house numbers/streets' part of the header.
fn fill_missing_header_items(
    ctx: &context::Context,
    streets: &str,
    additional_housenumbers: bool,
    relation_name: &str,
    items: &[yattag::Doc],
) -> anyhow::Result<Vec<yattag::Doc>> {
    let mut items: Vec<yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix();
    if streets != "only" {
        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-result",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Missing house numbers"));
        }
        items.push(doc);

        if additional_housenumbers {
            let doc = yattag::Doc::new();
            {
                let a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/additional-housenumbers/{}/view-result",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Additional house numbers"));
            }
            items.push(doc);
        }
    }
    if streets != "no" {
        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("{}/missing-streets/{}/view-result", prefix, relation_name),
                )],
            );
            a.text(&tr("Missing streets"));
        }
        items.push(doc);
        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/additional-streets/{}/view-result",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Additional streets"));
        }
        items.push(doc);
    }
    Ok(items)
}

/// Generates the 'existing house numbers/streets' part of the header.
fn fill_existing_header_items(
    ctx: &context::Context,
    streets: &str,
    relation_name: &str,
    items: &[yattag::Doc],
) -> anyhow::Result<Vec<yattag::Doc>> {
    let mut items: Vec<yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix();
    if streets != "only" {
        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/street-housenumbers/{}/view-result",
                        prefix, relation_name
                    ),
                )],
            );
            a.text(&tr("Existing house numbers"));
        }
        items.push(doc);
    }

    let doc = yattag::Doc::new();
    {
        let a = doc.tag(
            "a",
            &[(
                "href",
                &format!("{}/streets/{}/view-result", prefix, relation_name),
            )],
        );
        a.text(&tr("Existing streets"));
    }
    items.push(doc);
    Ok(items)
}

/// Emit localized strings for JS purposes.
pub fn emit_l10n_strings_for_js(doc: &yattag::Doc, string_pairs: &[(&str, String)]) {
    let div = doc.tag("div", &[("style", "display: none;")]);
    for (key, value) in string_pairs {
        let div = div.tag("div", &[("id", key), ("data-value", value)]);
        drop(div);
    }
}

/// Produces the start of the page. Note that the content depends on the function and the
/// relation, but not on the action to keep a balance between too generic and too specific
/// content.
pub fn get_toolbar(
    ctx: &context::Context,
    relations: Option<&mut areas::Relations>,
    function: &str,
    relation_name: &str,
    relation_osmid: u64,
) -> anyhow::Result<yattag::Doc> {
    let mut items: Vec<yattag::Doc> = Vec::new();

    let mut streets: String = "".into();
    let mut additional_housenumbers = false;
    if let Some(relations) = relations {
        if !relation_name.is_empty() {
            let relation = relations.get_relation(relation_name)?;
            streets = relation.get_config().should_check_missing_streets();
            additional_housenumbers = relation.get_config().should_check_additional_housenumbers();
        }
    }

    let doc = yattag::Doc::new();
    {
        let a = doc.tag("a", &[("href", &(ctx.get_ini().get_uri_prefix() + "/"))]);
        a.text(&tr("Area list"))
    }
    items.push(doc);

    if !relation_name.is_empty() {
        items = fill_missing_header_items(
            ctx,
            &streets,
            additional_housenumbers,
            relation_name,
            &items,
        )?;
    }

    items = fill_header_function(ctx, function, relation_name, &items)?;

    if !relation_name.is_empty() {
        items = fill_existing_header_items(ctx, &streets, relation_name, &items)?;
    }

    let doc = yattag::Doc::new();

    let string_pairs = &[
        ("str-toolbar-overpass-wait", tr("Waiting for Overpass...")),
        ("str-toolbar-overpass-error", tr("Error from Overpass: ")),
        (
            "str-toolbar-reference-wait",
            tr("Creating from reference..."),
        ),
        ("str-toolbar-reference-error", tr("Error from reference: ")),
    ];
    emit_l10n_strings_for_js(&doc, string_pairs);

    {
        let a = doc.tag("a", &[("href", "https://overpass-turbo.eu/")]);
        a.text(&tr("Overpass turbo"));
    }
    items.push(doc);

    let doc = yattag::Doc::new();
    if relation_osmid > 0 {
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("https://www.openstreetmap.org/relation/{}", relation_osmid),
                )],
            );
            a.text(&tr("Area boundary"))
        }
        items.push(doc);
    } else {
        // These are on the main page only.
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    &(ctx.get_ini().get_uri_prefix() + "/housenumber-stats/hungary/"),
                )],
            );
            a.text(&tr("Statistics"));
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let a = doc.tag(
                "a",
                &[(
                    "href",
                    "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
                )],
            );
            a.text(&tr("Documentation"));
        }
        items.push(doc);
    }

    let doc = yattag::Doc::new();
    {
        let div = doc.tag("div", &[("id", "toolbar")]);
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                div.text(" ¦ ");
            }
            div.append_value(item.get_value());
        }
    }
    doc.stag("hr");
    Ok(doc)
}

pub type Headers = Vec<(Cow<'static, str>, Cow<'static, str>)>;

/// Handles serving static content.
pub fn handle_static(
    ctx: &context::Context,
    request_uri: &str,
) -> anyhow::Result<(Vec<u8>, String, Headers)> {
    let mut tokens = request_uri.split('/');
    let path = tokens.next_back().unwrap();
    let extra_headers = Vec::new();

    if request_uri.ends_with(".js") {
        let content_type = "application/x-javascript; charset=utf-8";
        let (content, extra_headers) =
            get_content_with_meta(ctx, &ctx.get_abspath(&format!("target/browser/{}", path)))?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".css") {
        let content_type = "text/css; charset=utf-8";
        let (content, extra_headers) =
            get_content_with_meta(ctx, &ctx.get_abspath(&format!("target/browser/{}", path)))
                .context("get_content_with_meta() failed")?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".json") {
        let content_type = "application/json; charset=utf-8";
        let (content, extra_headers) = get_content_with_meta(
            ctx,
            &format!("{}/stats/{}", ctx.get_ini().get_workdir(), path),
        )?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".ico") {
        let content_type = "image/x-icon";
        let (content, extra_headers) = get_content_with_meta(ctx, &ctx.get_abspath(path))?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".svg") {
        let content_type = "image/svg+xml; charset=utf-8";
        let (content, extra_headers) = get_content_with_meta(ctx, &ctx.get_abspath(path))?;
        return Ok((content, content_type.into(), extra_headers));
    }

    let bytes: Vec<u8> = Vec::new();
    Ok((bytes, "".into(), extra_headers))
}

/// Displays an unhandled error on the page.
pub fn handle_error(request: &rouille::Request, error: &str) -> rouille::Response {
    let doc = yattag::Doc::new();
    util::write_html_header(&doc);
    {
        let pre = doc.tag("pre", &[]);
        let url = request.url();
        pre.text(&format!(
            "{}\n",
            tr("Internal error when serving {0}").replace("{0}", &url)
        ));
        pre.text(error);
    }
    make_response(
        500_u16,
        vec![("Content-type".into(), "text/html; charset=utf-8".into())],
        doc.get_value().as_bytes().to_vec(),
    )
}

/// Displays a not-found page.
pub fn handle_404() -> yattag::Doc {
    let doc = yattag::Doc::new();
    util::write_html_header(&doc);
    {
        let html = doc.tag("html", &[]);
        {
            let body = html.tag("body", &[]);
            {
                let h1 = body.tag("h1", &[]);
                h1.text(&tr("Not Found"));
            }
            {
                let p = doc.tag("p", &[]);
                p.text(&tr("The requested URL was not found on this server."));
            }
        }
    }
    doc
}

/// Formats timestamp as UI date-time.
pub fn format_timestamp(timestamp: i64) -> String {
    let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let utc: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(naive, chrono::Utc);
    let local: chrono::DateTime<chrono::Local> = chrono::DateTime::from(utc);
    local.format("%Y-%m-%d %H:%M").to_string()
}

/// Expected request_uri: e.g. /osm/housenumber-stats/hungary/cityprogress.
fn handle_stats_cityprogress(
    ctx: &context::Context,
    relations: &mut areas::Relations,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    doc.append_value(
        get_toolbar(
            ctx,
            Some(relations),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    let mut ref_citycounts: HashMap<String, u64> = HashMap::new();
    let csv_stream: Rc<RefCell<dyn Read>> = ctx
        .get_file_system()
        .open_read(&ctx.get_ini().get_reference_citycounts_path()?)?;
    let mut guard = csv_stream.borrow_mut();
    let mut read = guard.deref_mut();
    let mut csv_read = util::CsvRead::new(&mut read);
    let mut first = true;
    for result in csv_read.records() {
        if first {
            first = false;
            continue;
        }
        let row = result?;
        let city = row.get(0).unwrap();
        let count: u64 = row.get(1).unwrap().parse()?;
        ref_citycounts.insert(city.into(), count);
    }
    let timestamp = ctx.get_time().now();
    let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let today = naive.format("%Y-%m-%d").to_string();
    let mut osm_citycounts: HashMap<String, u64> = HashMap::new();
    let path = format!("{}/stats/{}.citycount", ctx.get_ini().get_workdir(), today);
    let csv_stream: Rc<RefCell<dyn Read>> = ctx.get_file_system().open_read(&path)?;
    let mut guard = csv_stream.borrow_mut();
    let mut read = guard.deref_mut();
    let mut csv_read = util::CsvRead::new(&mut read);
    for result in csv_read.records() {
        let row = result.context(format!("failed to read row in {}", path))?;
        let city = row.get(0).unwrap();
        let count: u64 = row.get(1).unwrap().parse()?;
        osm_citycounts.insert(city.into(), count);
    }
    let ref_cities: Vec<_> = ref_citycounts
        .iter()
        .map(|(k, _v)| util::Street::from_string(k))
        .collect();
    let osm_cities: Vec<_> = osm_citycounts
        .iter()
        .map(|(k, _v)| util::Street::from_string(k))
        .collect();
    let in_both = util::get_in_both(&ref_cities, &osm_cities);
    let mut cities: Vec<_> = in_both.iter().map(|i| i.get_osm_name()).collect();
    cities.sort_by_key(|i| util::get_sort_key(i).unwrap());
    let mut table: Vec<Vec<yattag::Doc>> = vec![vec![
        yattag::Doc::from_text(&tr("City name")),
        yattag::Doc::from_text(&tr("House number coverage")),
        yattag::Doc::from_text(&tr("OSM count")),
        yattag::Doc::from_text(&tr("Reference count")),
    ]];
    for city in cities {
        let mut percent = 100_f64;
        if *ref_citycounts.get(city).unwrap() > 0
            && osm_citycounts.get(city).unwrap() < ref_citycounts.get(city).unwrap()
        {
            let osm_count = osm_citycounts[city] as f64;
            let ref_count = ref_citycounts[city] as f64;
            percent = osm_count / ref_count * 100_f64;
        }
        let percent = util::format_percent(percent).context("util::format_percent() failed:")?;
        table.push(vec![
            yattag::Doc::from_text(city),
            yattag::Doc::from_text(&percent),
            yattag::Doc::from_text(&osm_citycounts.get(city).unwrap().to_string()),
            yattag::Doc::from_text(&ref_citycounts.get(city).unwrap().to_string()),
        ]);
    }
    doc.append_value(util::html_table_from_list(&table).get_value());

    {
        let h2 = doc.tag("h2", &[]);
        h2.text(&tr("Note"));
    }
    {
        let div = doc.tag("div", &[]);
        div.text(&tr(
            r#"These statistics are estimates, not taking house number filters into account.
Only cities with house numbers in OSM are considered."#,
        ));
    }

    doc.append_value(get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/housenumber-stats/hungary/zipprogress.
fn handle_stats_zipprogress(
    ctx: &context::Context,
    relations: &mut areas::Relations,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    doc.append_value(
        get_toolbar(
            ctx,
            Some(relations),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );
    let mut ref_zipcounts: HashMap<String, u64> = HashMap::new();
    let csv_stream: Rc<RefCell<dyn Read>> = ctx
        .get_file_system()
        .open_read(&ctx.get_ini().get_reference_zipcounts_path()?)?;
    let mut guard = csv_stream.borrow_mut();
    let mut read = guard.deref_mut();
    let mut csv_read = util::CsvRead::new(&mut read);
    let mut first = true;
    for result in csv_read.records() {
        if first {
            first = false;
            continue;
        }
        let row = result?;
        let zip = row.get(0).unwrap();
        let count: u64 = row.get(1).unwrap().parse()?;
        ref_zipcounts.insert(zip.into(), count);
    }
    let timestamp = ctx.get_time().now();
    let naive = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let today = naive.format("%Y-%m-%d").to_string();
    let mut osm_zipcounts: HashMap<String, u64> = HashMap::new();
    let path = format!("{}/stats/{}.zipcount", ctx.get_ini().get_workdir(), today);
    let csv_stream: Rc<RefCell<dyn Read>> = ctx.get_file_system().open_read(&path)?;
    let mut guard = csv_stream.borrow_mut();
    let mut read = guard.deref_mut();
    let mut csv_read = util::CsvRead::new(&mut read);
    for result in csv_read.records() {
        let row = result.context(format!("failed to read row in {}", path))?;
        let zip = row.get(0).unwrap();
        let count: u64 = row.get(1).unwrap().parse()?;
        osm_zipcounts.insert(zip.into(), count);
    }
    let ref_zips: Vec<_> = ref_zipcounts
        .iter()
        .map(|(k, _v)| util::Street::from_string(k))
        .collect();
    let osm_zips: Vec<_> = osm_zipcounts
        .iter()
        .map(|(k, _v)| util::Street::from_string(k))
        .collect();
    let in_both = util::get_in_both(&ref_zips, &osm_zips);
    let mut zips: Vec<_> = in_both.iter().map(|i| i.get_osm_name()).collect();
    zips.sort_by_key(|i| util::get_sort_key(i).unwrap());
    let mut table: Vec<Vec<yattag::Doc>> = vec![vec![
        yattag::Doc::from_text(&tr("ZIP code")),
        yattag::Doc::from_text(&tr("House number coverage")),
        yattag::Doc::from_text(&tr("OSM count")),
        yattag::Doc::from_text(&tr("Reference count")),
    ]];
    for zip in zips {
        let mut percent = 100_f64;
        if *ref_zipcounts.get(zip).unwrap() > 0
            && osm_zipcounts.get(zip).unwrap() < ref_zipcounts.get(zip).unwrap()
        {
            let osm_count = osm_zipcounts[zip] as f64;
            let ref_count = ref_zipcounts[zip] as f64;
            percent = osm_count / ref_count * 100_f64;
        }
        let percent = util::format_percent(percent).context("util::format_percent() failed:")?;
        table.push(vec![
            yattag::Doc::from_text(zip),
            yattag::Doc::from_text(&percent),
            yattag::Doc::from_text(&osm_zipcounts.get(zip).unwrap().to_string()),
            yattag::Doc::from_text(&ref_zipcounts.get(zip).unwrap().to_string()),
        ]);
    }
    doc.append_value(util::html_table_from_list(&table).get_value());

    {
        let h2 = doc.tag("h2", &[]);
        h2.text(&tr("Note"));
    }
    {
        let div = doc.tag("div", &[]);
        div.text(&tr(
            r#"These statistics are estimates, not taking house number filters into account.
Only zip codes with house numbers in OSM are considered."#,
        ));
    }

    doc.append_value(get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/housenumber-stats/hungary/invalid-relations."""
fn handle_invalid_refstreets(
    ctx: &context::Context,
    relations: &mut areas::Relations,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    doc.append_value(
        get_toolbar(
            ctx,
            Some(relations),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    let prefix = ctx.get_ini().get_uri_prefix();
    for relation in relations.get_relations()? {
        if !ctx
            .get_file_system()
            .path_exists(&relation.get_files().get_osm_streets_path())
        {
            continue;
        }
        let (osm_invalids, ref_invalids) = relation
            .get_invalid_refstreets()
            .context("get_invalid_refstreets() failed")?;
        let key_invalids = relation.get_invalid_filter_keys()?;
        if osm_invalids.is_empty() && ref_invalids.is_empty() && key_invalids.is_empty() {
            continue;
        }
        {
            let h1 = doc.tag("h1", &[]);
            let relation_name = relation.get_name();
            {
                let a = h1.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/streets/{}/view-result", prefix, relation_name),
                    )],
                );
                a.text(&relation_name);
            }
        }
        doc.append_value(
            util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value(),
        );
        doc.append_value(util::invalid_filter_keys_to_html(&key_invalids).get_value());
    }

    doc.append_value(get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/housenumber-stats/hungary/.
pub fn handle_stats(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    if request_uri.ends_with("/cityprogress") {
        return handle_stats_cityprogress(ctx, relations);
    }

    if request_uri.ends_with("/zipprogress") {
        return handle_stats_zipprogress(ctx, relations);
    }

    if request_uri.ends_with("/invalid-relations") {
        return handle_invalid_refstreets(ctx, relations);
    }

    let doc = yattag::Doc::new();
    doc.append_value(
        get_toolbar(
            ctx,
            Some(relations),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    let prefix = ctx.get_ini().get_uri_prefix();

    let string_pairs = &[
        (
            "str-daily-title",
            tr("New house numbers, last 2 weeks, as of {}"),
        ),
        ("str-daily-x-axis", tr("During this day")),
        ("str-daily-y-axis", tr("New house numbers")),
        (
            "str-monthly-title",
            tr("New house numbers, last year, as of {}"),
        ),
        ("str-monthly-x-axis", tr("During this month")),
        ("str-monthly-y-axis", tr("New house numbers")),
        (
            "str-monthlytotal-title",
            tr("All house numbers, last year, as of {}"),
        ),
        ("str-monthlytotal-x-axis", tr("Latest for this month")),
        ("str-monthlytotal-y-axis", tr("All house numbers")),
        (
            "str-dailytotal-title",
            tr("All house numbers, last 2 weeks, as of {}"),
        ),
        ("str-dailytotal-x-axis", tr("At the start of this day")),
        ("str-dailytotal-y-axis", tr("All house numbers")),
        (
            "str-topusers-title",
            tr("Top house number editors, as of {}"),
        ),
        ("str-topusers-x-axis", tr("User name")),
        (
            "str-topusers-y-axis",
            tr("Number of house numbers last changed by this user"),
        ),
        ("str-topcities-title", tr("Top edited cities, as of {}")),
        ("str-topcities-x-axis", tr("City name")),
        (
            "str-topcities-y-axis",
            tr("Number of house numbers added in the past 30 days"),
        ),
        ("str-topcities-empty", tr("(empty)")),
        ("str-topcities-invalid", tr("(invalid)")),
        (
            "str-usertotal-title",
            tr("Number of house number editors, as of {}"),
        ),
        ("str-usertotal-x-axis", tr("All editors")),
        (
            "str-usertotal-y-axis",
            tr("Number of editors, at least one housenumber is last changed by these users"),
        ),
        ("str-progress-title", tr("Coverage is {1}%, as of {2}")),
        (
            "str-progress-x-axis",
            tr("Number of house numbers in database"),
        ),
        ("str-progress-y-axis", tr("Data source")),
        (
            "str-capital-progress-title",
            tr("Coverage is {1}% for the capital, as of {2}"),
        ),
        (
            "str-capital-progress-x-axis",
            tr("Number of house numbers in database for the capital"),
        ),
        ("str-reference", tr("Reference")),
    ];
    emit_l10n_strings_for_js(&doc, string_pairs);

    let title_ids = &[
        (tr("New house numbers"), "daily"),
        (tr("All house numbers"), "dailytotal"),
        (tr("New house numbers, monthly"), "monthly"),
        (tr("All house numbers, monthly"), "monthlytotal"),
        (tr("Top house number editors"), "topusers"),
        (tr("Top edited cities"), "topcities"),
        (tr("All house number editors"), "usertotal"),
        (tr("Coverage"), "progress"),
        (tr("Capital coverage"), "capital-progress"),
        (tr("Per-city coverage"), "cityprogress"),
        (tr("Per-ZIP coverage"), "zipprogress"),
        (tr("Invalid relation settings"), "invalid-relations"),
    ];

    {
        let ul = doc.tag("ul", &[]);
        for (title, identifier) in title_ids {
            let identifier = identifier.to_string();
            let li = ul.tag("li", &[]);
            if identifier == "cityprogress" {
                let a = li.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/housenumber-stats/hungary/cityprogress", prefix),
                    )],
                );
                a.text(title);
                continue;
            }
            if identifier == "zipprogress" {
                let a = li.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/housenumber-stats/hungary/zipprogress", prefix),
                    )],
                );
                a.text(title);
                continue;
            }
            if identifier == "invalid-relations" {
                let a = li.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/housenumber-stats/hungary/invalid-relations", prefix),
                    )],
                );
                a.text(title);
                continue;
            }
            let a = li.tag("a", &[("href", &format!("#_{}", identifier))]);
            a.text(title);
        }
    }

    for (title, identifier) in title_ids {
        let identifier = identifier.to_string();
        if identifier == "cityprogress"
            || identifier == "zipprogress"
            || identifier == "invalid-relations"
        {
            continue;
        }
        {
            let h2 = doc.tag("h2", &[("id", &format!("_{}", identifier))]);
            h2.text(title);
        }

        let div = doc.tag("div", &[("class", "canvasblock js")]);
        let canvas = div.tag("canvas", &[("id", &identifier)]);
        drop(canvas);
    }

    {
        let h2 = doc.tag("h2", &[]);
        h2.text(&tr("Note"));
    }
    {
        let div = doc.tag("div", &[]);
        div.text(&tr(
            r#"These statistics are provided purely for interested editors, and are not
intended to reflect quality of work done by any given editor in OSM. If you want to use
them to motivate yourself, that's fine, but keep in mind that a bit of useful work is
more meaningful than a lot of useless work."#,
        ));
    }

    doc.append_value(get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

/// Finds out the request URI.
pub fn get_request_uri(
    request: &rouille::Request,
    ctx: &context::Context,
    relations: &mut areas::Relations,
) -> anyhow::Result<String> {
    let mut request_uri = request.url();

    let prefix = ctx.get_ini().get_uri_prefix();
    if !request_uri.is_empty() {
        // Compatibility.
        if request_uri.starts_with(&format!("{}/suspicious-streets/", prefix)) {
            request_uri = request_uri.replace("suspicious-streets", "missing-housenumbers");
        } else if request_uri.starts_with(&format!("{}/suspicious-relations/", prefix)) {
            request_uri = request_uri.replace("suspicious-relations", "missing-streets");
        }

        // Performance: don't bother with relation aliases for non-relation requests.
        if !request_uri.starts_with(&format!("{}/streets/", prefix))
            && !request_uri.starts_with(&format!("{}/missing-streets/", prefix))
            && !request_uri.starts_with(&format!("{}/street-housenumbers/", prefix))
            && !request_uri.starts_with(&format!("{}/missing-housenumbers/", prefix))
        {
            return Ok(request_uri);
        }

        // Relation aliases.
        let aliases = relations.get_aliases()?;
        let mut tokens = request_uri.split('/');
        tokens.next_back();
        let relation_name = tokens.next_back().unwrap();
        if let Some(value) = aliases.get(relation_name) {
            request_uri = request_uri.replace(relation_name, value);
        }
    }

    Ok(request_uri)
}

/// Prevents serving outdated data from a relation that has been renamed.
pub fn check_existing_relation(
    ctx: &context::Context,
    relations: &areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix();
    if !request_uri.starts_with(&format!("{}/streets/", prefix))
        && !request_uri.starts_with(&format!("{}/missing-streets/", prefix))
        && !request_uri.starts_with(&format!("{}/street-housenumbers/", prefix))
        && !request_uri.starts_with(&format!("{}/missing-housenumbers/", prefix))
    {
        return Ok(doc);
    }

    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name: &String = &tokens.next_back().unwrap().to_string();
    if relations.get_names().contains(relation_name) {
        return Ok(doc);
    }

    {
        let div = doc.tag("div", &[("id", "no-such-relation-error")]);
        div.text(&tr("No such relation: {0}").replace("{0}", relation_name));
    }
    Ok(doc)
}

/// Handles the no-osm-streets error on a page using JS.
pub fn handle_no_osm_streets(prefix: &str, relation_name: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let link = format!("{}/streets/{}/uppdate-result", prefix, relation_name);
    {
        let div = doc.tag("div", &[("id", "no-osm-streets")]);
        let a = div.tag("a", &[("href", &link)]);
        a.text(&tr("No existing streets: call Overpass to create..."));
    }
    let string_pairs = &[
        (
            "str-overpass-wait",
            tr("No existing streets: waiting for Overpass..."),
        ),
        ("str-overpass-error", tr("Error from Overpass: ")),
    ];
    emit_l10n_strings_for_js(&doc, string_pairs);
    doc
}

/// Handles the no-osm-housenumbers error on a page using JS.
pub fn handle_no_osm_housenumbers(prefix: &str, relation_name: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let link = format!(
        "{}/street-housenumbers/{}/uppdate-result",
        prefix, relation_name
    );
    {
        let div = doc.tag("div", &[("id", "no-osm-housenumbers")]);
        let a = div.tag("a", &[("href", &link)]);
        a.text(&tr("No existing house numbers: call Overpass to create..."));
    }
    // Emit localized strings for JS purposes.
    let string_pairs = &[
        (
            "str-overpass-wait",
            tr("No existing house numbers: waiting for Overpass..."),
        ),
        ("str-overpass-error", tr("Error from Overpass: ")),
    ];
    emit_l10n_strings_for_js(&doc, string_pairs);
    doc
}

/// Handles the no-ref-housenumbers error on a page using JS.
pub fn handle_no_ref_housenumbers(prefix: &str, relation_name: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let link = format!(
        "{}/missing-housenumbers/{}/uppdate-result",
        prefix, relation_name
    );
    {
        let div = doc.tag("div", &[("id", "no-ref-housenumbers")]);
        let a = div.tag("a", &[("href", &link)]);
        a.text(&tr("No reference house numbers: create from reference..."));
    }
    // Emit localized strings for JS purposes.
    let string_pairs = &[
        (
            "str-reference-wait",
            tr("No reference house numbers: creating from reference..."),
        ),
        ("str-reference-error", tr("Error from reference: ")),
    ];
    emit_l10n_strings_for_js(&doc, string_pairs);
    doc
}

/// Handles the no-ref-streets error on a page using JS.
pub fn handle_no_ref_streets(prefix: &str, relation_name: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let link = format!("{}/missing-streets/{}/update-result", prefix, relation_name);
    {
        let div = doc.tag("div", &[("id", "no-ref-streets")]);
        let a = div.tag("a", &[("href", &link)]);
        a.text(&tr("No street list: create from reference..."));
    }
    let string_pairs = &[
        (
            "str-reference-wait",
            tr("No reference streets: creating from reference..."),
        ),
        ("str-reference-error", tr("Error from reference: ")),
    ];
    emit_l10n_strings_for_js(&doc, string_pairs);
    doc
}

/// Handles a GitHub style webhook.
pub fn handle_github_webhook(
    request: &rouille::Request,
    ctx: &context::Context,
) -> anyhow::Result<yattag::Doc> {
    let mut request_data = Vec::new();
    let mut reader = request.data().context("data() gave None")?;
    reader.read_to_end(&mut request_data)?;

    let pairs = url::form_urlencoded::parse(&request_data);
    let payloads: Vec<String> = pairs
        .filter(|(key, _value)| key == "payload")
        .map(|(_key, value)| value.into())
        .collect();
    let payload = &payloads[0];
    let value: serde_json::Value = serde_json::from_str(payload)?;
    let branch = value
        .as_object()
        .unwrap()
        .get("ref")
        .unwrap()
        .as_str()
        .unwrap();
    if branch == "refs/heads/master" {
        ctx.get_subprocess().run(vec![
            "make".into(),
            "-C".into(),
            ctx.get_abspath(""),
            "deploy".into(),
        ])?;
        // Nominally a failure, so the service gets restarted.
        println!("Stopping the server after deploy.");
        ctx.get_subprocess().exit(1);
    }

    Ok(yattag::Doc::from_text(""))
}

/// Factory for rouille::Response.
pub fn make_response(status_code: u16, headers: Headers, data: Vec<u8>) -> rouille::Response {
    rouille::Response {
        status_code,
        headers,
        data: rouille::ResponseBody::from_data(data),
        upgrade: None,
    }
}

/// Gets the content of a file in workdir with metadata.
fn get_content_with_meta(ctx: &context::Context, path: &str) -> anyhow::Result<(Vec<u8>, Headers)> {
    let stream = ctx
        .get_file_system()
        .open_read(path)
        .context("open_read() failed")?;
    let mut buf: Vec<u8> = Vec::new();
    let mut guard = stream.borrow_mut();
    guard.read_to_end(&mut buf).unwrap();

    let mtime = ctx
        .get_file_system()
        .getmtime(path)
        .context("getmtime() failed")?;
    let naive = chrono::NaiveDateTime::from_timestamp_opt(mtime as i64, 0).unwrap();
    let utc: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(naive, chrono::Utc);

    let extra_headers = vec![("Last-Modified".into(), utc.to_rfc2822().into())];
    Ok((buf, extra_headers))
}

#[cfg(test)]
mod tests;
