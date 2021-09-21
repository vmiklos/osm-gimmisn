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

#[pyfunction]
fn py_fill_header_function(
    ctx: crate::context::PyContext,
    function: &str,
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
    let ret = match fill_header_function(&ctx.context, function, relation_name, &items) {
        Ok(value) => value,
        Err(err) => {
            return Err(pyo3::exceptions::PyOSError::new_err(format!(
                "fill_header_function() failed: {}",
                err.to_string()
            )));
        }
    };
    Ok(ret
        .iter()
        .map(|i| crate::yattag::PyDoc { doc: i.clone() })
        .collect())
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_get_footer, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_fill_header_function, module)?)?;
    Ok(())
}
