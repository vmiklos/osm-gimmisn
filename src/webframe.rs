/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The webframe module provides the header, toolbar and footer code.

use crate::i18n::translate as tr;
use git_version::git_version;
use pyo3::prelude::*;

/// Produces the end of the page.
fn get_footer(last_updated: &str) -> crate::yattag::Doc {
    let mut items: Vec<crate::yattag::Doc> = Vec::new();
    {
        let doc = crate::yattag::Doc::new();
        doc.text(&tr("Version: "));
        doc.append_value(
            crate::util::git_link(
                git_version!(),
                "https://github.com/vmiklos/osm-gimmisn/commit/",
            )
            .get_value(),
        );
        items.push(doc);
        items.push(crate::yattag::Doc::from_text(&tr(
            "OSM data © OpenStreetMap contributors.",
        )));
        if !last_updated.is_empty() {
            items.push(crate::yattag::Doc::from_text(
                &(tr("Last update: ") + last_updated),
            ));
        }
    }
    let doc = crate::yattag::Doc::new();
    doc.stag("hr", vec![]);
    {
        let _div = doc.tag("div", vec![]);
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
fn py_get_footer(last_updated: &str) -> crate::yattag::PyDoc {
    let ret = get_footer(last_updated);
    crate::yattag::PyDoc { doc: ret }
}

/// Fills items with function-specific links in the header. Returns the extended list.
fn fill_header_function(
    ctx: &crate::context::Context,
    function: &str,
    relation_name: &str,
    items: &[crate::yattag::Doc],
) -> anyhow::Result<Vec<crate::yattag::Doc>> {
    let mut items: Vec<crate::yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if function == "missing-housenumbers" {
        // The OSM data source changes much more frequently than the ref one, so add a dedicated link
        // to update OSM house numbers first.
        let doc = crate::yattag::Doc::new();
        {
            let _span = doc.tag("span", vec![("id", "trigger-street-housenumbers-update")]);
            {
                let _a = doc.tag(
                    "a",
                    vec![(
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

        let doc = crate::yattag::Doc::new();
        {
            let _span = doc.tag("span", vec![("id", "trigger-missing-housenumbers-update")]);
            {
                let _a = doc.tag(
                    "a",
                    vec![(
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
        let doc = crate::yattag::Doc::new();
        {
            let _span = doc.tag("span", vec![("id", "trigger-streets-update")]);
            {
                let _a = doc.tag(
                    "a",
                    vec![(
                        "href",
                        &format!("{}/streets/{}/update-result", prefix, relation_name),
                    )],
                );
                doc.text(&tr("Update from OSM"));
            }
        }
        items.push(doc);

        let doc = crate::yattag::Doc::new();
        {
            let _span = doc.tag("span", vec![("id", "trigger-missing-streets-update")]);
            {
                let _a = doc.tag(
                    "a",
                    vec![(
                        "href",
                        &format!("{}/missing-streets/{}/update-result", prefix, relation_name),
                    )],
                );
                doc.text(&tr("Update from reference"));
            }
        }
        items.push(doc);
    } else if function == "street-housenumbers" {
        let doc = crate::yattag::Doc::new();
        {
            let _span = doc.tag("span", vec![("id", "trigger-street-housenumbers-update")]);
            {
                let _a = doc.tag(
                    "a",
                    vec![(
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
        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
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
        let doc = crate::yattag::Doc::new();
        {
            let _span = doc.tag("span", vec![("id", "trigger-streets-update")]);
            {
                let _a = doc.tag(
                    "a",
                    vec![(
                        "href",
                        &format!("{}/streets/{}/update-result", prefix, relation_name),
                    )],
                );
                doc.text(&tr("Call Overpass to update"));
            }
        }
        items.push(doc);
        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
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
    ctx: &crate::context::Context,
    streets: &str,
    additional_housenumbers: bool,
    relation_name: &str,
    items: &[crate::yattag::Doc],
) -> anyhow::Result<Vec<crate::yattag::Doc>> {
    let mut items: Vec<crate::yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if streets != "only" {
        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
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
            let doc = crate::yattag::Doc::new();
            {
                let _a = doc.tag(
                    "a",
                    vec![(
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
        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
                    "href",
                    &format!("{}/missing-streets/{}/view-result", prefix, relation_name),
                )],
            );
            doc.text(&tr("Missing streets"));
        }
        items.push(doc);
        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
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
    ctx: crate::context::PyContext,
    streets: &str,
    additional_housenumbers: bool,
    relation_name: &str,
    items: Vec<PyObject>,
) -> PyResult<Vec<crate::yattag::PyDoc>> {
    let gil = Python::acquire_gil();
    let items: Vec<crate::yattag::Doc> = items
        .iter()
        .map(|i| {
            let i: PyRefMut<'_, crate::yattag::PyDoc> = i.extract(gil.python()).unwrap();
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
        .map(|i| crate::yattag::PyDoc { doc: i.clone() })
        .collect())
}

/// Generates the 'existing house numbers/streets' part of the header.
fn fill_existing_header_items(
    ctx: &crate::context::Context,
    streets: &str,
    relation_name: &str,
    items: &[crate::yattag::Doc],
) -> anyhow::Result<Vec<crate::yattag::Doc>> {
    let mut items: Vec<crate::yattag::Doc> = items.to_vec();
    let prefix = ctx.get_ini().get_uri_prefix()?;
    if streets != "only" {
        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
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

    let doc = crate::yattag::Doc::new();
    {
        let _a = doc.tag(
            "a",
            vec![(
                "href",
                &format!("{}/streets/{}/view-result", prefix, relation_name),
            )],
        );
        doc.text(&tr("Existing streets"));
    }
    items.push(doc);
    Ok(items)
}

/// Produces the start of the page. Note that the content depends on the function and the
/// relation, but not on the action to keep a balance between too generic and too specific
/// content.
fn get_toolbar(
    ctx: &crate::context::Context,
    relations: &Option<crate::areas::Relations>,
    function: &str,
    relation_name: &str,
    relation_osmid: u64,
) -> anyhow::Result<crate::yattag::Doc> {
    let mut items: Vec<crate::yattag::Doc> = Vec::new();

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

    let doc = crate::yattag::Doc::new();
    {
        let _a = doc.tag(
            "a",
            vec![("href", &(ctx.get_ini().get_uri_prefix()? + "/"))],
        );
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

    let doc = crate::yattag::Doc::new();

    // Emit localized strings for JS purposes.
    {
        let _div = doc.tag("div", vec![("style", "display: none;")]);
        let string_pairs = vec![
            ("str-toolbar-overpass-wait", tr("Waiting for Overpass...")),
            ("str-toolbar-overpass-error", tr("Error from Overpass: ")),
            (
                "str-toolbar-reference-wait",
                tr("Creating from reference..."),
            ),
            ("str-toolbar-reference-error", tr("Error from reference: ")),
        ];
        for (key, value) in string_pairs {
            let _div = doc.tag("div", vec![("id", key), ("data-value", &value)]);
        }
    }

    {
        let _a = doc.tag("a", vec![("href", "https://overpass-turbo.eu/")]);
        doc.text(&tr("Overpass turbo"));
    }
    items.push(doc);

    let doc = crate::yattag::Doc::new();
    if relation_osmid > 0 {
        {
            let _a = doc.tag(
                "a",
                vec![(
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
                vec![(
                    "href",
                    &(ctx.get_ini().get_uri_prefix()? + "/housenumber-stats/hungary/"),
                )],
            );
            doc.text(&tr("Statistics"));
        }
        items.push(doc);

        let doc = crate::yattag::Doc::new();
        {
            let _a = doc.tag(
                "a",
                vec![(
                    "href",
                    "https://github.com/vmiklos/osm-gimmisn/tree/master/doc",
                )],
            );
            doc.text(&tr("Documentation"));
        }
        items.push(doc);
    }

    let doc = crate::yattag::Doc::new();
    {
        let _div = doc.tag("div", vec![("id", "toolbar")]);
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                doc.text(" ¦ ");
            }
            doc.append_value(item.get_value());
        }
    }
    doc.stag("hr", vec![]);
    Ok(doc)
}

#[pyfunction]
fn py_get_toolbar(
    ctx: crate::context::PyContext,
    relations: Option<crate::areas::PyRelations>,
    function: &str,
    relation_name: &str,
    relation_osmid: u64,
) -> PyResult<crate::yattag::PyDoc> {
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
    Ok(crate::yattag::PyDoc { doc: ret })
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_get_footer, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(
        py_fill_missing_header_items,
        module
    )?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_get_toolbar, module)?)?;
    Ok(())
}
