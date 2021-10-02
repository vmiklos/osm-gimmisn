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
use anyhow::anyhow;
use anyhow::Context;
use git_version::git_version;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::collections::HashMap;
use std::io::Read;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

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
    doc.stag("hr", &[]);
    {
        let _div = doc.tag("div", &[]);
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                doc.text(" ¦ ");
            }
            doc.append_value(item.get_value());
        }
    }
    doc
}

#[pyfunction]
fn py_get_footer(last_updated: &str) -> yattag::PyDoc {
    let ret = get_footer(last_updated);
    yattag::PyDoc { doc: ret }
}

/// Fills items with function-specific links in the header. Returns the extended list.
fn fill_header_function(
    ctx: &context::Context,
    function: &str,
    relation_name: &str,
    items: &[yattag::Doc],
) -> anyhow::Result<Vec<yattag::Doc>> {
    let mut items: Vec<yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if function == "missing-housenumbers" {
        // The OSM data source changes much more frequently than the ref one, so add a dedicated link
        // to update OSM house numbers first.
        let doc = yattag::Doc::new();
        {
            let _span = doc.tag("span", &[("id", "trigger-street-housenumbers-update")]);
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/street-housenumbers/{}/update-result",
                            prefix, relation_name
                        ),
                    )],
                );
                doc.text(&tr("Update from OSM"));
            }
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let _span = doc.tag("span", &[("id", "trigger-missing-housenumbers-update")]);
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/missing-housenumbers/{}/update-result",
                            prefix, relation_name
                        ),
                    )],
                );
                doc.text(&tr("Update from reference"));
            }
        }
        items.push(doc);
    } else if function == "missing-streets" || function == "additional-streets" {
        // The OSM data source changes much more frequently than the ref one, so add a dedicated link
        // to update OSM streets first.
        let doc = yattag::Doc::new();
        {
            let _span = doc.tag("span", &[("id", "trigger-streets-update")]);
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/streets/{}/update-result", prefix, relation_name),
                    )],
                );
                doc.text(&tr("Update from OSM"));
            }
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let _span = doc.tag("span", &[("id", "trigger-missing-streets-update")]);
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/missing-streets/{}/update-result", prefix, relation_name),
                    )],
                );
                doc.text(&tr("Update from reference"));
            }
        }
        items.push(doc);
    } else if function == "street-housenumbers" {
        let doc = yattag::Doc::new();
        {
            let _span = doc.tag("span", &[("id", "trigger-street-housenumbers-update")]);
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/street-housenumbers/{}/update-result",
                            prefix, relation_name
                        ),
                    )],
                );
                doc.text(&tr("Call Overpass to update"));
            }
        }
        items.push(doc);
        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/street-housenumbers/{}/view-query",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("View query"));
        }
        items.push(doc);
    } else if function == "streets" {
        let doc = yattag::Doc::new();
        {
            let _span = doc.tag("span", &[("id", "trigger-streets-update")]);
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/streets/{}/update-result", prefix, relation_name),
                    )],
                );
                doc.text(&tr("Call Overpass to update"));
            }
        }
        items.push(doc);
        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("{}/streets/{}/view-query", prefix, relation_name),
                )],
            );
            doc.text(&tr("View query"));
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
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if streets != "only" {
        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/missing-housenumbers/{}/view-result",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("Missing house numbers"));
        }
        items.push(doc);

        if additional_housenumbers {
            let doc = yattag::Doc::new();
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/additional-housenumbers/{}/view-result",
                            prefix, relation_name
                        ),
                    )],
                );
                doc.text(&tr("Additional house numbers"));
            }
            items.push(doc);
        }
    }
    if streets != "no" {
        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("{}/missing-streets/{}/view-result", prefix, relation_name),
                )],
            );
            doc.text(&tr("Missing streets"));
        }
        items.push(doc);
        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/additional-streets/{}/view-result",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("Additional streets"));
        }
        items.push(doc);
    }
    Ok(items)
}

#[pyfunction]
fn py_fill_missing_header_items(
    ctx: context::PyContext,
    streets: &str,
    additional_housenumbers: bool,
    relation_name: &str,
    items: Vec<PyObject>,
) -> PyResult<Vec<yattag::PyDoc>> {
    let gil = Python::acquire_gil();
    let items: Vec<yattag::Doc> = items
        .iter()
        .map(|i| {
            let i: PyRefMut<'_, yattag::PyDoc> = i.extract(gil.python()).unwrap();
            i.doc.clone()
        })
        .collect();
    let ret = match fill_missing_header_items(
        &ctx.context,
        streets,
        additional_housenumbers,
        relation_name,
        &items,
    ) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "fill_missing_header_items() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(ret
        .iter()
        .map(|i| yattag::PyDoc { doc: i.clone() })
        .collect())
}

/// Generates the 'existing house numbers/streets' part of the header.
fn fill_existing_header_items(
    ctx: &context::Context,
    streets: &str,
    relation_name: &str,
    items: &[yattag::Doc],
) -> anyhow::Result<Vec<yattag::Doc>> {
    let mut items: Vec<yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if streets != "only" {
        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!(
                        "{}/street-housenumbers/{}/view-result",
                        prefix, relation_name
                    ),
                )],
            );
            doc.text(&tr("Existing house numbers"));
        }
        items.push(doc);
    }

    let doc = yattag::Doc::new();
    {
        let _a = doc.tag(
            "a",
            &[(
                "href",
                &format!("{}/streets/{}/view-result", prefix, relation_name),
            )],
        );
        doc.text(&tr("Existing streets"));
    }
    items.push(doc);
    Ok(items)
}

/// Emit localized strings for JS purposes.
fn emit_l10n_strings_for_js(doc: &yattag::Doc, string_pairs: &[(&str, String)]) {
    let _div = doc.tag("div", &[("style", "display: none;")]);
    for (key, value) in string_pairs {
        let _div = doc.tag("div", &[("id", key), ("data-value", value)]);
    }
}

/// Produces the start of the page. Note that the content depends on the function and the
/// relation, but not on the action to keep a balance between too generic and too specific
/// content.
pub fn get_toolbar(
    ctx: &context::Context,
    relations: &Option<areas::Relations>,
    function: &str,
    relation_name: &str,
    relation_osmid: u64,
) -> anyhow::Result<yattag::Doc> {
    let mut items: Vec<yattag::Doc> = Vec::new();

    let mut streets: String = "".into();
    let mut additional_housenumbers = false;
    if !relations.is_none() && !relation_name.is_empty() {
        let relation = relations
            .as_ref()
            .unwrap()
            .clone()
            .get_relation(relation_name)?;
        streets = relation.get_config().should_check_missing_streets();
        additional_housenumbers = relation.get_config().should_check_additional_housenumbers();
    }

    let doc = yattag::Doc::new();
    {
        let _a = doc.tag("a", &[("href", &(ctx.get_ini().get_uri_prefix()? + "/"))]);
        doc.text(&tr("Area list"))
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
        let _a = doc.tag("a", &[("href", "https://overpass-turbo.eu/")]);
        doc.text(&tr("Overpass turbo"));
    }
    items.push(doc);

    let doc = yattag::Doc::new();
    if relation_osmid > 0 {
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &format!("https://www.openstreetmap.org/relation/{}", relation_osmid),
                )],
            );
            doc.text(&tr("Area boundary"))
        }
        items.push(doc);
    } else {
        // These are on the main page only.
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    &(ctx.get_ini().get_uri_prefix()? + "/housenumber-stats/hungary/"),
                )],
            );
            doc.text(&tr("Statistics"));
        }
        items.push(doc);

        let doc = yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                &[(
                    "href",
                    "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
                )],
            );
            doc.text(&tr("Documentation"));
        }
        items.push(doc);
    }

    let doc = yattag::Doc::new();
    {
        let _div = doc.tag("div", &[("id", "toolbar")]);
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                doc.text(" ¦ ");
            }
            doc.append_value(item.get_value());
        }
    }
    doc.stag("hr", &[]);
    Ok(doc)
}

#[pyfunction]
fn py_get_toolbar(
    ctx: context::PyContext,
    relations: Option<areas::PyRelations>,
    function: &str,
    relation_name: &str,
    relation_osmid: u64,
) -> PyResult<yattag::PyDoc> {
    let relations = match relations {
        Some(value) => Some(value.relations),
        None => None,
    };
    let ret = match get_toolbar(
        &ctx.context,
        &relations,
        function,
        relation_name,
        relation_osmid,
    ) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "get_toolbar() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(yattag::PyDoc { doc: ret })
}

pub type Headers = Vec<(String, String)>;

/// Handles serving static content.
fn handle_static(
    ctx: &context::Context,
    request_uri: &str,
) -> anyhow::Result<(Vec<u8>, String, Headers)> {
    let mut tokens = request_uri.split('/');
    let path = tokens.next_back().unwrap();
    let extra_headers: Vec<(String, String)> = Vec::new();

    if request_uri.ends_with(".js") {
        let content_type = "application/x-javascript";
        let (content, extra_headers) =
            util::get_content_with_meta(&ctx.get_abspath(&format!("builddir/{}", path))?)?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".css") {
        let content_type = "text/css";
        let (content, extra_headers) =
            util::get_content_with_meta(&format!("{}/{}", ctx.get_ini().get_workdir()?, path))?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".json") {
        let content_type = "application/json";
        let (content, extra_headers) = util::get_content_with_meta(&format!(
            "{}/stats/{}",
            ctx.get_ini().get_workdir()?,
            path
        ))?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".ico") {
        let content_type = "image/x-icon";
        let (content, extra_headers) = util::get_content_with_meta(&ctx.get_abspath(path)?)?;
        return Ok((content, content_type.into(), extra_headers));
    }
    if request_uri.ends_with(".svg") {
        let content_type = "image/svg+xml";
        let (content, extra_headers) = util::get_content_with_meta(&ctx.get_abspath(path)?)?;
        return Ok((content, content_type.into(), extra_headers));
    }

    let bytes: Vec<u8> = Vec::new();
    Ok((bytes, "".into(), extra_headers))
}

#[pyfunction]
fn py_handle_static(
    ctx: context::PyContext,
    request_uri: &str,
) -> PyResult<(PyObject, String, Headers)> {
    let (content, content_type, extra_headers) = match handle_static(&ctx.context, request_uri) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "handle_static() failed: {}",
                err.to_string()
            )));
        }
    };

    let gil = Python::acquire_gil();
    Ok((
        PyBytes::new(gil.python(), &content).into(),
        content_type,
        extra_headers,
    ))
}

/// A HTTP response, to be sent by send_response().
#[derive(Clone)]
pub struct Response {
    content_type: String,
    status: String,
    output_bytes: Vec<u8>,
    headers: Headers,
}

impl Response {
    pub fn new(
        content_type: &str,
        status: &str,
        output_bytes: &[u8],
        headers: &[(String, String)],
    ) -> Self {
        Response {
            content_type: content_type.into(),
            status: status.into(),
            output_bytes: output_bytes.to_vec(),
            headers: headers.to_vec(),
        }
    }

    /// Gets the Content-type value.
    fn get_content_type(&self) -> &String {
        &self.content_type
    }

    /// Gets the HTTP status.
    fn get_status(&self) -> &String {
        &self.status
    }

    /// Gets the encoded output.
    fn get_output_bytes(&self) -> &Vec<u8> {
        &self.output_bytes
    }

    /// Gets the HTTP headers.
    fn get_headers(&self) -> &Headers {
        &self.headers
    }
}

#[pyclass]
#[derive(Clone)]
struct PyResponse {
    response: Response,
}

#[pymethods]
impl PyResponse {
    #[new]
    fn new(content_type: &str, status: &str, output_bytes: Vec<u8>, headers: Headers) -> Self {
        let response = Response::new(content_type, status, &output_bytes, &headers);
        PyResponse { response }
    }

    fn get_content_type(&self) -> String {
        self.response.get_content_type().clone()
    }

    fn get_status(&self) -> String {
        self.response.get_status().clone()
    }

    fn get_output_bytes(&self) -> PyObject {
        let gil = Python::acquire_gil();
        PyBytes::new(gil.python(), self.response.get_output_bytes()).into()
    }

    fn get_headers(&self) -> Headers {
        self.response.get_headers().clone()
    }
}

/// Turns an output string into a byte array and sends it.
pub fn send_response(
    environ: &HashMap<String, String>,
    response: &Response,
) -> anyhow::Result<(String, Headers, Vec<Vec<u8>>)> {
    let mut content_type: String = response.get_content_type().into();
    if content_type != "application/octet-stream" {
        content_type.push_str("; charset=utf-8");
    }

    // Apply content encoding: gzip, etc.
    let accept_encodings = environ.get("HTTP_ACCEPT_ENCODING");
    let mut output_bytes = response.get_output_bytes().clone();
    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(value) = accept_encodings {
        let request = rouille::Request::fake_http(
            "GET",
            "/",
            vec![("Accept-Encoding".to_owned(), value.into())],
            Vec::<u8>::new(),
        );
        let response = rouille::Response::from_data("application/x-javascript", output_bytes);
        let compressed = rouille::content_encoding::apply(&request, response);
        let (mut reader, _size) = compressed.data.into_reader_and_size();
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        output_bytes = buffer;
        let content_encodings: Vec<String> = compressed
            .headers
            .iter()
            .filter(|(key, _value)| key == "Content-Encoding")
            .map(|(_key, value)| value.to_string())
            .collect();
        if let Some(value) = content_encodings.get(0) {
            headers.push(("Content-Encoding".into(), value.into()));
        }
    }
    let content_length = output_bytes.len();
    headers.push(("Content-type".into(), content_type));
    headers.push(("Content-Length".into(), content_length.to_string()));
    headers.append(&mut response.get_headers().clone());
    let status = response.get_status();
    Ok((status.into(), headers, vec![output_bytes]))
}

#[pyfunction]
fn py_send_response(
    environ: HashMap<String, String>,
    response: PyResponse,
) -> PyResult<(String, Headers, Vec<PyObject>)> {
    let (status, headers, output_byte_list) = match send_response(&environ, &response.response) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "send_response() failed: {}",
                err.to_string()
            )));
        }
    };

    let gil = Python::acquire_gil();
    let output_byte_list: Vec<PyObject> = output_byte_list
        .iter()
        .map(|i| PyBytes::new(gil.python(), i).into())
        .collect();
    Ok((status, headers, output_byte_list))
}

/// Displays an unhandled exception on the page.
fn handle_exception(
    environ: &HashMap<String, String>,
    error: &str,
) -> anyhow::Result<(String, Headers, Vec<Vec<u8>>)> {
    let status = "500 Internal Server Error";
    let request_uri = environ
        .get("PATH_INFO")
        .ok_or_else(|| anyhow!("no PATH_INFO in the environment"))?;
    let doc = yattag::Doc::new();
    util::write_html_header(&doc);
    {
        let _pre = doc.tag("pre", &[]);
        doc.text(&format!(
            "{}\n",
            tr("Internal error when serving {0}").replace("{0}", request_uri)
        ));
        doc.text(error);
    }
    let response_properties = Response::new("text/html", status, doc.get_value().as_bytes(), &[]);
    send_response(environ, &response_properties)
}

#[pyfunction]
fn py_handle_exception(
    environ: HashMap<String, String>,
    error: String,
) -> PyResult<(String, Headers, Vec<PyObject>)> {
    let (status, headers, output_byte_list) = match handle_exception(&environ, &error) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "handle_exception() failed: {}",
                err.to_string()
            )));
        }
    };

    let gil = Python::acquire_gil();
    let output_byte_list: Vec<PyObject> = output_byte_list
        .iter()
        .map(|i| PyBytes::new(gil.python(), i).into())
        .collect();
    Ok((status, headers, output_byte_list))
}

/// Displays a not-found page.
fn handle_404() -> yattag::Doc {
    let doc = yattag::Doc::new();
    util::write_html_header(&doc);
    {
        let _html = doc.tag("html", &[]);
        {
            let _body = doc.tag("body", &[]);
            {
                let _h1 = doc.tag("h1", &[]);
                doc.text(&tr("Not Found"));
            }
            {
                let _p = doc.tag("p", &[]);
                doc.text(&tr("The requested URL was not found on this server."));
            }
        }
    }
    doc
}

/// Formats timestamp as UI date-time.
pub fn format_timestamp(timestamp: i64) -> String {
    let naive = chrono::NaiveDateTime::from_timestamp(timestamp, 0);
    let utc: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(naive, chrono::Utc);
    let local: chrono::DateTime<chrono::Local> = chrono::DateTime::from(utc);
    local.format("%Y-%m-%d %H:%M").to_string()
}

#[pyfunction]
fn py_format_timestamp(timestamp: f64) -> String {
    format_timestamp(timestamp as i64)
}

#[pyfunction]
fn py_handle_404() -> yattag::PyDoc {
    let doc = handle_404();
    yattag::PyDoc { doc }
}

/// Expected request_uri: e.g. /osm/housenumber-stats/hungary/cityprogress.
fn handle_stats_cityprogress(
    ctx: &context::Context,
    relations: &areas::Relations,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    doc.append_value(
        get_toolbar(
            ctx,
            &Some(relations.clone()),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    let mut ref_citycounts: HashMap<String, u64> = HashMap::new();
    let csv_stream: Arc<Mutex<dyn Read + Send>> = ctx
        .get_file_system()
        .open_read(&ctx.get_ini().get_reference_citycounts_path()?)?;
    let mut guard = csv_stream.lock().unwrap();
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
    let naive = chrono::NaiveDateTime::from_timestamp(timestamp, 0);
    let today = naive.format("%Y-%m-%d").to_string();
    let mut osm_citycounts: HashMap<String, u64> = HashMap::new();
    let path = format!("{}/stats/{}.citycount", ctx.get_ini().get_workdir()?, today);
    let csv_stream: Arc<Mutex<dyn Read + Send>> = ctx.get_file_system().open_read(&path)?;
    let mut guard = csv_stream.lock().unwrap();
    let mut read = guard.deref_mut();
    let mut csv_read = util::CsvRead::new(&mut read);
    for result in csv_read.records() {
        let row = result.with_context(|| format!("failed to read row in {}", path))?;
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
        let mut percent: String = "100.00".into();
        if *ref_citycounts.get(city).unwrap() > 0
            && osm_citycounts.get(city).unwrap() < ref_citycounts.get(city).unwrap()
        {
            let osm_count = osm_citycounts[city] as f64;
            let ref_count = ref_citycounts[city] as f64;
            percent = format!("{0:.2}", osm_count / ref_count * 100_f64);
        }
        table.push(vec![
            yattag::Doc::from_text(city),
            yattag::Doc::from_text(
                &util::format_percent(&percent).context("util::format_percent() failed:")?,
            ),
            yattag::Doc::from_text(&osm_citycounts.get(city).unwrap().to_string()),
            yattag::Doc::from_text(&ref_citycounts.get(city).unwrap().to_string()),
        ]);
    }
    doc.append_value(util::html_table_from_list(&table).get_value());

    {
        let _h2 = doc.tag("h2", &[]);
        doc.text(&tr("Note"));
    }
    {
        let _div = doc.tag("div", &[]);
        doc.text(&tr(
            r#"These statistics are estimates, not taking house number filters into account.
Only cities with house numbers in OSM are considered."#,
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
            &Some(relations.clone()),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    let prefix = ctx.get_ini().get_uri_prefix()?;
    for relation in relations.get_relations()? {
        if !ctx
            .get_file_system()
            .path_exists(&relation.get_files().get_osm_streets_path()?)
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
            let _h1 = doc.tag("h1", &[]);
            let relation_name = relation.get_name();
            {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/streets/{}/view-result", prefix, relation_name),
                    )],
                );
                doc.text(&relation_name);
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
fn handle_stats(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    if request_uri.ends_with("/cityprogress") {
        return handle_stats_cityprogress(ctx, relations);
    }

    if request_uri.ends_with("/invalid-relations") {
        return handle_invalid_refstreets(ctx, relations);
    }

    let doc = yattag::Doc::new();
    doc.append_value(
        get_toolbar(
            ctx,
            &Some(relations.clone()),
            /*function=*/ "",
            /*relation_name=*/ "",
            /*relation_osmid=*/ 0,
        )?
        .get_value(),
    );

    let prefix = ctx.get_ini().get_uri_prefix()?;

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
        (tr("Per-city coverage"), "cityprogress"),
        (tr("Invalid relation settings"), "invalid-relations"),
    ];

    {
        let _ul = doc.tag("ul", &[]);
        for (title, identifier) in title_ids {
            let identifier = identifier.to_string();
            let _li = doc.tag("li", &[]);
            if identifier == "cityprogress" {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/housenumber-stats/hungary/cityprogress", prefix),
                    )],
                );
                doc.text(title);
                continue;
            }
            if identifier == "invalid-relations" {
                let _a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/housenumber-stats/hungary/invalid-relations", prefix),
                    )],
                );
                doc.text(title);
                continue;
            }
            let _a = doc.tag("a", &[("href", &format!("#_{}", identifier))]);
            doc.text(title);
        }
    }

    for (title, identifier) in title_ids {
        let identifier = identifier.to_string();
        if identifier == "cityprogress" || identifier == "invalid-relations" {
            continue;
        }
        {
            let _h2 = doc.tag("h2", &[("id", &format!("_{}", identifier))]);
            doc.text(title);
        }

        let _div = doc.tag("div", &[("class", "canvasblock js")]);
        let _canvas = doc.tag("canvas", &[("id", &identifier)]);
    }

    {
        let _h2 = doc.tag("h2", &[]);
        doc.text(&tr("Note"));
    }
    {
        let _div = doc.tag("div", &[]);
        doc.text(&tr(
            r#"These statistics are provided purely for interested editors, and are not
intended to reflect quality of work done by any given editor in OSM. If you want to use
them to motivate yourself, that's fine, but keep in mind that a bit of useful work is
more meaningful than a lot of useless work."#,
        ));
    }

    doc.append_value(get_footer(/*last_updated=*/ "").get_value());
    Ok(doc)
}

#[pyfunction]
fn py_handle_stats(
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    let doc = match handle_stats(&ctx.context, &mut relations.relations, request_uri)
        .context("handle_stats() failed")
    {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err)));
        }
    };

    Ok(yattag::PyDoc { doc })
}

/// Finds out the request URI.
fn get_request_uri(
    environ: &HashMap<String, String>,
    ctx: &context::Context,
    relations: &mut areas::Relations,
) -> anyhow::Result<String> {
    let mut request_uri: String = environ.get("PATH_INFO").unwrap().into();

    let prefix = ctx.get_ini().get_uri_prefix()?;
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

#[pyfunction]
fn py_get_request_uri(
    environ: HashMap<String, String>,
    ctx: context::PyContext,
    mut relations: areas::PyRelations,
) -> PyResult<String> {
    match get_request_uri(&environ, &ctx.context, &mut relations.relations)
        .context("get_request_uri() failed")
    {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

/// Prevents serving outdated data from a relation that has been renamed.
fn check_existing_relation(
    ctx: &context::Context,
    relations: &areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let doc = yattag::Doc::new();
    let prefix = ctx.get_ini().get_uri_prefix()?;
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
        let _div = doc.tag("div", &[("id", "no-such-relation-error")]);
        doc.text(&tr("No such relation: {0}").replace("{0}", relation_name));
    }
    Ok(doc)
}

#[pyfunction]
fn py_check_existing_relation(
    ctx: context::PyContext,
    relations: areas::PyRelations,
    request_uri: &str,
) -> PyResult<yattag::PyDoc> {
    let doc = match check_existing_relation(&ctx.context, &relations.relations, request_uri)
        .context("check_existing_relation() failed")
    {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err)));
        }
    };

    Ok(yattag::PyDoc { doc })
}

/// Handles the no-osm-streets error on a page using JS.
pub fn handle_no_osm_streets(prefix: &str, relation_name: &str) -> yattag::Doc {
    let doc = yattag::Doc::new();
    let link = format!("{}/streets/{}/uppdate-result", prefix, relation_name);
    {
        let _div = doc.tag("div", &[("id", "no-osm-streets")]);
        let _a = doc.tag("a", &[("href", &link)]);
        doc.text(&tr("No existing streets: call Overpass to create..."));
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
        let _div = doc.tag("div", &[("id", "no-osm-housenumbers")]);
        let _a = doc.tag("a", &[("href", &link)]);
        doc.text(&tr("No existing house numbers: call Overpass to create..."));
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
        let _div = doc.tag("div", &[("id", "no-ref-housenumbers")]);
        let _a = doc.tag("a", &[("href", &link)]);
        doc.text(&tr("No reference house numbers: create from reference..."));
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
        let _div = doc.tag("div", &[("id", "no-ref-streets")]);
        let _a = doc.tag("a", &[("href", &link)]);
        doc.text(&tr("No street list: create from reference..."));
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
fn handle_github_webhook(
    stream: &mut dyn Read,
    ctx: &context::Context,
) -> anyhow::Result<yattag::Doc> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;
    let prefixed = format!("http://www.example.com/?{}", String::from_utf8(buffer)?);
    let url = reqwest::Url::parse(&prefixed)?;
    let body = url.query_pairs();
    let payloads: Vec<String> = body
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
        let mut my_env: HashMap<String, String> = HashMap::new();
        my_env.insert(
            "PATH".into(),
            format!("osm-gimmisn-env/bin:{}", std::env::var("PATH")?),
        );
        ctx.get_subprocess().run(
            vec![
                "make".into(),
                "-C".into(),
                ctx.get_abspath("")?,
                "deploy".into(),
            ],
            my_env,
        )?;
        // Nominally a failure, so the service gets restarted.
        ctx.get_subprocess().exit(1);
    }

    Ok(yattag::Doc::from_text(""))
}

#[pyfunction]
fn py_handle_github_webhook(stream: PyObject, ctx: context::PyContext) -> PyResult<yattag::PyDoc> {
    let gil = Python::acquire_gil();
    let any = match stream.call_method0(gil.python(), "read") {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "read() failed: {}",
                err.to_string()
            )));
        }
    };
    let bytes = match any.as_ref(gil.python()).downcast::<PyBytes>() {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "read() didn't return bytes: {}",
                err.to_string()
            )));
        }
    };
    let mut read: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(bytes.extract().unwrap());
    let doc = match handle_github_webhook(&mut read, &ctx.context)
        .context("handle_github_webhook() failed")
    {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err)));
        }
    };

    Ok(yattag::PyDoc { doc })
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_get_footer, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_fill_missing_header_items,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_toolbar, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_static, module)?)?;
    module.add_class::<PyResponse>()?;
    module.add_function(pyo3::wrap_pyfunction!(py_send_response, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_exception, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_404, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_format_timestamp, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_stats, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_request_uri, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_check_existing_relation, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_handle_github_webhook, module)?)?;
    Ok(())
}
