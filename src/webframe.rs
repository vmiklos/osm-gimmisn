/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The webframe module provides the header, toolbar and footer code.

use crate::areas;
use crate::context;
use crate::i18n::translate as tr;
use crate::stats;
use crate::util;
use crate::yattag;
use anyhow::Context;
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
                git_version::git_version!(args = ["--always", "--long"]),
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
                        &format!("{prefix}/street-housenumbers/{relation_name}/update-result"),
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
                        &format!("{prefix}/missing-housenumbers/{relation_name}/update-result"),
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
                        &format!("{prefix}/streets/{relation_name}/update-result"),
                    )],
                );
                a.text(&tr("Update from OSM"));
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
                        &format!("{prefix}/street-housenumbers/{relation_name}/update-result"),
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
                    &format!("{prefix}/street-housenumbers/{relation_name}/view-query"),
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
                        &format!("{prefix}/streets/{relation_name}/update-result"),
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
                    &format!("{prefix}/streets/{relation_name}/view-query"),
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
                    &format!("{prefix}/missing-housenumbers/{relation_name}/view-result"),
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
                        &format!("{prefix}/additional-housenumbers/{relation_name}/view-result"),
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
                    &format!("{prefix}/missing-streets/{relation_name}/view-result"),
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
                    &format!("{prefix}/additional-streets/{relation_name}/view-result"),
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
                    &format!("{prefix}/street-housenumbers/{relation_name}/view-result"),
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
                &format!("{prefix}/streets/{relation_name}/view-result"),
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
    relations: Option<&mut areas::Relations<'_>>,
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
                    &format!("https://www.openstreetmap.org/relation/{relation_osmid}"),
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
                    &(ctx.get_ini().get_uri_prefix() + "/housenumber-stats/whole-country/"),
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
                    &(ctx.get_ini().get_uri_prefix() + "/lints/whole-country/"),
                )],
            );
            a.text(&tr("Lints"));
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let a = doc.tag("a", &[("href", &tr("https://vmiklos.hu/osm-gimmisn"))]);
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
    let path = tokens.next_back().context("next_back() failed")?;
    let extra_headers = Vec::new();

    if request_uri.ends_with(".js") {
        let content_type = "application/x-javascript; charset=utf-8";
        let (content, extra_headers) =
            get_content_with_meta(ctx, &ctx.get_abspath(&format!("target/browser/{path}")))?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".css") {
        let content_type = "text/css; charset=utf-8";
        let (content, extra_headers) =
            get_content_with_meta(ctx, &ctx.get_abspath(&format!("target/browser/{path}")))
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
pub fn format_timestamp(timestamp: &time::OffsetDateTime) -> anyhow::Result<String> {
    let format = time::format_description::parse("[year]-[month]-[day] [hour]:[minute]")?;
    Ok(timestamp.format(&format)?)
}

/// Expected request_uri: e.g. /osm/housenumber-stats/whole-country/cityprogress.
fn handle_stats_cityprogress(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
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
    let mut csv_reader = util::make_csv_reader(&mut read);
    for result in csv_reader.deserialize() {
        let row: util::CityCount = result?;
        ref_citycounts.insert(row.city, row.count);
    }
    let date_time = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = date_time.format(&format)?;
    let mut osm_citycounts: HashMap<String, u64> = HashMap::new();
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select city, count from stats_citycounts where date = ?1")?;
    let mut rows = stmt.query([&today])?;
    while let Some(row) = rows.next()? {
        let city: String = row.get(0).unwrap();
        let count: String = row.get(1).unwrap();
        osm_citycounts.insert(city, count.parse()?);
    }
    let ref_cities: Vec<_> = ref_citycounts
        .keys()
        .map(|k| util::Street::from_string(k))
        .collect();
    let osm_cities: Vec<_> = osm_citycounts
        .keys()
        .map(|k| util::Street::from_string(k))
        .collect();
    let in_both = util::get_in_both(&ref_cities, &osm_cities);
    let mut cities: Vec<_> = in_both.iter().map(|i| i.get_osm_name()).collect();
    cities.sort_by_key(|i| util::get_sort_key(i));
    let mut table: Vec<Vec<yattag::Doc>> = vec![vec![
        yattag::Doc::from_text(&tr("City name")),
        yattag::Doc::from_text(&tr("House number coverage")),
        yattag::Doc::from_text(&tr("OSM count")),
        yattag::Doc::from_text(&tr("Reference count")),
    ]];
    for city in cities {
        let mut percent = 100_f64;
        if ref_citycounts[city] > 0 && osm_citycounts[city] < ref_citycounts[city] {
            let osm_count = osm_citycounts[city] as f64;
            let ref_count = ref_citycounts[city] as f64;
            percent = osm_count / ref_count * 100_f64;
        }
        let percent = util::format_percent(percent).context("util::format_percent() failed:")?;
        table.push(vec![
            yattag::Doc::from_text(city),
            yattag::Doc::from_text(&percent),
            yattag::Doc::from_text(&osm_citycounts[city].to_string()),
            yattag::Doc::from_text(&ref_citycounts[city].to_string()),
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

/// Expected request_uri: e.g. /osm/housenumber-stats/whole-country/zipprogress.
fn handle_stats_zipprogress(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
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
    let mut csv_reader = util::make_csv_reader(&mut read);
    for result in csv_reader.deserialize() {
        let row: util::ZipCount = result?;
        ref_zipcounts.insert(row.zip, row.count);
    }
    let now = ctx.get_time().now();
    let format = time::format_description::parse("[year]-[month]-[day]")?;
    let today = now.format(&format)?;
    let mut osm_zipcounts: HashMap<String, u64> = HashMap::new();
    let conn = ctx.get_database_connection()?;
    let mut stmt = conn.prepare("select zip, count from stats_zipcounts where date = ?1")?;
    let mut rows = stmt.query([&today])?;
    while let Some(row) = rows.next()? {
        let zip: String = row.get(0).unwrap();
        let count: String = row.get(1).unwrap();
        osm_zipcounts.insert(zip, count.parse()?);
    }
    let ref_zips: Vec<_> = ref_zipcounts
        .keys()
        .map(|k| util::Street::from_string(k))
        .collect();
    let osm_zips: Vec<_> = osm_zipcounts
        .keys()
        .map(|k| util::Street::from_string(k))
        .collect();
    let in_both = util::get_in_both(&ref_zips, &osm_zips);
    let mut zips: Vec<_> = in_both.iter().map(|i| i.get_osm_name()).collect();
    zips.sort_by_key(|i| util::get_sort_key(i));
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

/// Expected request uri: /housenumber-stats/whole-country/invalid-addr-cities.
fn handle_invalid_addr_cities(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
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

    let mut table: Vec<Vec<yattag::Doc>> = Vec::new();
    let mut count = 0;
    {
        let conn = ctx.get_database_connection()?;
        let mut stmt = conn
        .prepare("select osm_id, osm_type, postcode, city, street, housenumber, user, timestamp, fixme from stats_invalid_addr_cities")?;
        let mut invalids = stmt.query([])?;
        {
            let cells: Vec<yattag::Doc> = vec![
                yattag::Doc::from_text(&tr("Identifier")),
                yattag::Doc::from_text(&tr("Type")),
                yattag::Doc::from_text(&tr("Postcode")),
                yattag::Doc::from_text(&tr("City")),
                yattag::Doc::from_text(&tr("Street")),
                yattag::Doc::from_text(&tr("Housenumber")),
                yattag::Doc::from_text(&tr("User")),
                yattag::Doc::from_text(&tr("Timestamp")),
                yattag::Doc::from_text(&tr("Fixme")),
            ];
            table.push(cells);
        }
        while let Some(invalid) = invalids.next()? {
            let mut cells: Vec<yattag::Doc> = Vec::new();
            let osm_id: String = invalid.get(0).unwrap();
            let osm_type: String = invalid.get(1).unwrap();
            {
                let cell = yattag::Doc::new();
                let href = format!("https://www.openstreetmap.org/{osm_type}/{osm_id}");
                {
                    let a = cell.tag("a", &[("href", href.as_str()), ("target", "_blank")]);
                    a.text(&osm_id.to_string());
                }
                cells.push(cell);
            }
            cells.push(yattag::Doc::from_text(&osm_type));
            let postcode: String = invalid.get(2).unwrap();
            cells.push(yattag::Doc::from_text(&postcode));
            let city: String = invalid.get(3).unwrap();
            cells.push(yattag::Doc::from_text(&city));
            let street: String = invalid.get(4).unwrap();
            cells.push(yattag::Doc::from_text(&street));
            let housenumber: String = invalid.get(5).unwrap();
            cells.push(yattag::Doc::from_text(&housenumber));
            let user: String = invalid.get(6).unwrap();
            cells.push(yattag::Doc::from_text(&user));
            let timestamp: String = invalid.get(7).unwrap();
            cells.push(yattag::Doc::from_text(&timestamp));
            let fixme: String = invalid.get(8).unwrap();
            cells.push(yattag::Doc::from_text(&fixme));
            table.push(cells);
            count += 1;
        }
    }
    {
        let p = doc.tag("p", &[]);
        p.text(
            &tr("The addr:city key of the below {0} objects probably has an invalid value.")
                .replace("{0}", &count.to_string()),
        );
    }
    doc.append_value(util::html_table_from_list(&table).get_value());
    let date = format_timestamp(&stats::get_sql_mtime(ctx, "stats/invalid-addr-cities")?)?;
    doc.append_value(get_footer(&date).get_value());
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/lints/whole-country/invalid-relations."""
fn handle_invalid_refstreets(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
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
        if !stats::has_sql_mtime(ctx, &format!("streets/{}", relation.get_name())).unwrap() {
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
                        &format!("{prefix}/streets/{relation_name}/view-result"),
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

/// Expected request_uri: e.g. /osm/housenumber-stats/whole-country/.
pub fn handle_stats(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    if request_uri.ends_with("/cityprogress") {
        return handle_stats_cityprogress(ctx, relations)
            .context("handle_stats_cityprogress() failed");
    }

    if request_uri.ends_with("/zipprogress") {
        return handle_stats_zipprogress(ctx, relations)
            .context("handle_stats_zipprogress() failed");
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
        (
            "str-invalid-addr-cities-title",
            tr("Invalid addr:city values, last 2 weeks, as of {}"),
        ),
        ("str-invalid-addr-cities-x-axis", tr("During this day")),
        (
            "str-invalid-addr-cities-y-axis",
            tr("Invalid addr:city values"),
        ),
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
        (
            tr("Invalid addr:city values history"),
            "stats-invalid-addr-cities",
        ),
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
                        &format!("{prefix}/housenumber-stats/whole-country/cityprogress"),
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
                        &format!("{prefix}/housenumber-stats/whole-country/zipprogress"),
                    )],
                );
                a.text(title);
                continue;
            }
            let a = li.tag("a", &[("href", &format!("#_{identifier}"))]);
            a.text(title);
        }
    }

    for (title, identifier) in title_ids {
        let identifier = identifier.to_string();
        if identifier == "cityprogress" || identifier == "zipprogress" {
            continue;
        }
        {
            let h2 = doc.tag("h2", &[("id", &format!("_{identifier}"))]);
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

/// Expected request_uri: /lints/whole-country/.
pub fn handle_lints(
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    if request_uri.ends_with("/invalid-relations") {
        return handle_invalid_refstreets(ctx, relations);
    }

    if request_uri.ends_with("/invalid-addr-cities") {
        return handle_invalid_addr_cities(ctx, relations);
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

    let title_ids = &[
        (tr("Invalid relation settings"), "invalid-relations"),
        (tr("Invalid addr:city values"), "invalid-addr-cities"),
    ];

    {
        let ul = doc.tag("ul", &[]);
        for (title, identifier) in title_ids {
            let identifier = identifier.to_string();
            let li = ul.tag("li", &[]);
            if identifier == "invalid-relations" {
                let a = li.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{prefix}/lints/whole-country/invalid-relations"),
                    )],
                );
                a.text(title);
                continue;
            }

            // Assume invalid-addr-cities.
            let a = li.tag(
                "a",
                &[(
                    "href",
                    &format!("{prefix}/lints/whole-country/invalid-addr-cities"),
                )],
            );
            a.text(title);
        }
    }

    doc.append_value(get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

/// Finds out the request URI.
pub fn get_request_uri(
    request: &rouille::Request,
    ctx: &context::Context,
    relations: &mut areas::Relations<'_>,
) -> anyhow::Result<String> {
    let mut request_uri = request.url();

    let prefix = ctx.get_ini().get_uri_prefix();
    if !request_uri.is_empty() {
        // Compatibility.
        if request_uri.starts_with(&format!("{prefix}/suspicious-streets/")) {
            request_uri = request_uri.replace("suspicious-streets", "missing-housenumbers");
        } else if request_uri.starts_with(&format!("{prefix}/suspicious-relations/")) {
            request_uri = request_uri.replace("suspicious-relations", "missing-streets");
        }

        // Performance: don't bother with relation aliases for non-relation requests.
        if !request_uri.starts_with(&format!("{prefix}/streets/"))
            && !request_uri.starts_with(&format!("{prefix}/missing-streets/"))
            && !request_uri.starts_with(&format!("{prefix}/street-housenumbers/"))
            && !request_uri.starts_with(&format!("{prefix}/missing-housenumbers/"))
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
    relations: &areas::Relations<'_>,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix();
    if !request_uri.starts_with(&format!("{prefix}/streets/"))
        && !request_uri.starts_with(&format!("{prefix}/missing-streets/"))
        && !request_uri.starts_with(&format!("{prefix}/street-housenumbers/"))
        && !request_uri.starts_with(&format!("{prefix}/missing-housenumbers/"))
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
    let link = format!("{prefix}/streets/{relation_name}/uppdate-result");
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
    let link = format!("{prefix}/street-housenumbers/{relation_name}/uppdate-result");
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
    let link = format!("{prefix}/missing-housenumbers/{relation_name}/uppdate-result");
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

    let extra_headers: Headers = vec![(
        "Last-Modified".into(),
        mtime
            .format(&time::format_description::well_known::Rfc2822)?
            .into(),
    )];
    Ok((buf, extra_headers))
}

#[cfg(test)]
mod tests;
