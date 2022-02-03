/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The wsgi_additional module contains functionality for additional streets.

use crate::areas;
use crate::cache;
use crate::context;
use crate::i18n::translate as tr;
use crate::util;
use crate::webframe;
use crate::yattag;
use anyhow::Context;

/// Expected request_uri: e.g. /osm/additional-streets/ujbuda/view-result.txt.
pub fn additional_streets_view_txt(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
    chkl: bool,
) -> anyhow::Result<(String, String)> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations
        .get_relation(relation_name)
        .context("get_relation() failed")?;

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
        let mut streets = relation.get_additional_streets(/*sorted_result=*/ true)?;
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()).unwrap());
        let mut lines: Vec<String> = Vec::new();
        for street in streets {
            if chkl {
                lines.push(format!("[ ] {}\n", street.get_osm_name()));
            } else {
                lines.push(format!("{}\n", street.get_osm_name()));
            }
        }
        output = lines.join("");
    }
    Ok((output, relation_name.into()))
}

/// Expected request_uri: e.g. /osm/additional-streets/budapest_11/view-result.
pub fn additional_streets_view_result(
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
    } else if !ctx
        .get_file_system()
        .path_exists(&relation.get_files().get_ref_streets_path())
    {
        doc.append_value(webframe::handle_no_ref_streets(&prefix, relation_name).get_value());
    } else {
        // Get "only in OSM" streets.
        let mut streets = relation.write_additional_streets()?;
        let count = streets.len();
        streets.sort_by_key(|street| util::get_sort_key(street.get_osm_name()).unwrap());
        let mut table = vec![vec![
            yattag::Doc::from_text(&tr("Identifier")),
            yattag::Doc::from_text(&tr("Type")),
            yattag::Doc::from_text(&tr("Source")),
            yattag::Doc::from_text(&tr("Street name")),
        ]];
        for street in streets {
            let cell = yattag::Doc::new();
            let href = format!(
                "https://www.openstreetmap.org/{}/{}",
                street.get_osm_type(),
                street.get_osm_id()
            );
            {
                let a = cell.tag("a", &[("href", &href), ("target", "_blank")]);
                a.text(&street.get_osm_id().to_string());
            }
            let cells = vec![
                cell,
                yattag::Doc::from_text(street.get_osm_type()),
                yattag::Doc::from_text(street.get_source()),
                yattag::Doc::from_text(street.get_osm_name()),
            ];
            table.push(cells);
        }

        {
            let p = doc.tag("p", &[]);
            p.text(
                &tr("OpenStreetMap additionally has the below {0} streets.")
                    .replace("{0}", &count.to_string()),
            );
            p.stag("br", &[]);
            {
                let a = p.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/additional-streets/{}/view-result.txt",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Plain text format"));
            }
            p.stag("br", &[]);
            {
                let a = p.tag(
                    "a",
                    &[(
                        "href",
                        &format!(
                            "{}/additional-streets/{}/view-result.chkl",
                            prefix, relation_name
                        ),
                    )],
                );
                a.text(&tr("Checklist format"));
            }
            p.stag("br", &[]);
            {
                let a = doc.tag(
                    "a",
                    &[(
                        "href",
                        &format!("{}/additional-streets/{}/view-turbo", prefix, relation_name),
                    )],
                );
                a.text(&tr("Overpass turbo query for the below streets"));
            }
        }

        doc.append_value(util::html_table_from_list(&table).get_value());
        let (osm_invalids, ref_invalids) = relation.get_invalid_refstreets()?;
        doc.append_value(
            util::invalid_refstreets_to_html(&osm_invalids, &ref_invalids).get_value(),
        );
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/budapest_11/view-result.
pub fn additional_housenumbers_view_result(
    ctx: &context::Context,
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let mut relation = relations.get_relation(relation_name)?;

    let doc: yattag::Doc;
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
        doc = cache::get_additional_housenumbers_html(ctx, &mut relation)?;
    }
    Ok(doc)
}

/// Expected request_uri: e.g. /osm/additional-housenumbers/ormezo/view-turbo.
pub fn additional_streets_view_turbo(
    relations: &mut areas::Relations,
    request_uri: &str,
) -> anyhow::Result<yattag::Doc> {
    let mut tokens = request_uri.split('/');
    tokens.next_back();
    let relation_name = tokens.next_back().unwrap();
    let relation = relations.get_relation(relation_name)?;

    let doc = yattag::Doc::new();
    let streets = relation.get_additional_streets(/*sorted_result=*/ false)?;
    let query = areas::make_turbo_query_for_street_objs(&relation, &streets);

    let pre = doc.tag("pre", &[]);
    pre.text(&query);
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::sync::Arc;

    use crate::areas;
    use crate::context;
    use crate::wsgi;

    /// Tests additional streets: the txt output.
    #[test]
    fn test_streets_view_result_txt() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();

        let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

        assert_eq!(result, "Only In OSM utca\n");
    }

    /// Tests additional streets: the chkl output.
    #[test]
    fn test_streets_view_result_chkl() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();

        let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.chkl");

        assert_eq!(result, "[ ] Only In OSM utca\n");
    }

    /// Tests additional streets: the txt output, no osm streets case.
    #[test]
    fn test_streets_view_result_txt_no_osm_streets() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);

        let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

        assert_eq!(result, "No existing streets");
    }

    /// Tests additional streets: the txt output, no ref streets case.
    #[test]
    fn test_streets_view_result_txt_no_ref_streets() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);

        let result = test_wsgi.get_txt_for_path("/additional-streets/gazdagret/view-result.txt");

        assert_eq!(result, "No reference streets");
    }

    /// Tests additional streets: if the view-turbo output is well-formed.
    #[test]
    fn test_streets_view_turbo_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-turbo");
        let results = wsgi::tests::TestWsgi::find_all(&root, "body/pre");
        assert_eq!(results.len(), 1);
    }

    /// Tests handle_main_housenr_additional_count().
    #[test]
    fn test_handle_main_housenr_additional_count() {
        let ctx = context::tests::make_test_context().unwrap();
        let mut relations = areas::Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("budafok").unwrap();
        let actual = wsgi::handle_main_housenr_additional_count(&ctx, &relation).unwrap();
        assert_eq!(actual.get_value().contains("42 house numbers"), true);
    }

    /// Tests handle_main_housenr_additional_count(): what happens when the count file is not there.
    #[test]
    fn test_handle_main_housenr_additional_count_no_count_file() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let mut relations = areas::Relations::new(&ctx).unwrap();
        let relation = relations.get_relation("budafok").unwrap();
        let hide_path = relation
            .get_files()
            .get_housenumbers_additional_count_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        ctx.set_file_system(&file_system_arc);

        let actual = wsgi::handle_main_housenr_additional_count(&ctx, &relation).unwrap();

        // Assert that the info is not there to ensure a fast main page.
        assert_eq!(actual.get_value().contains("42 house numbers"), false);
    }

    /// Tests the additional house numbers page: if the output is well-formed.
    #[test]
    fn test_additional_housenumbers_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");
        let results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional house numbers page: if the output is well-formed, no osm streets case.
    #[test]
    fn test_additional_housenumbers_no_osm_streets_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);
        let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");
        let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional house numbers page: if the output is well-formed, no osm housenumbers case.
    #[test]
    fn test_additional_housenumbers_no_osm_housenumbers_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        let yamls_cache = serde_json::json!({
            "relations.yaml": {
                "gazdagret": {
                    "osmrelation": 42,
                },
            },
        });
        let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
        let files = context::tests::TestFileSystem::make_files(
            test_wsgi.get_ctx(),
            &[("data/yamls.cache", &yamls_cache_value)],
        );
        file_system.set_files(&files);
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);

        let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

        let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-housenumbers']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional house numbers page: if the output is well-formed, no ref housenumbers case.
    #[test]
    fn test_additional_housenumbers_no_ref_housenumbers_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_housenumbers_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);

        let root = test_wsgi.get_dom_for_path("/additional-housenumbers/gazdagret/view-result");

        let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-ref-housenumbers']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional streets page: if the output is well-formed.
    #[test]
    fn test_streets_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let count_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            test_wsgi.get_ctx(),
            &[("workdir/gazdagret-additional-streets.count", &count_value)],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.get_ctx().set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

        let mut guard = count_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let mut results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
        // refstreets: >0 invalid osm name
        results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='osm-invalids-container']");
        assert_eq!(results.len(), 1);
        // refstreets: >0 invalid ref name
        results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='ref-invalids-container']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional streets page: if the output is well-formed when the street name comes
    /// from a housenr.
    #[test]
    fn test_streets_street_from_housenr_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let yamls_cache = serde_json::json!({
            "relations.yaml": {
                "gh611": {
                    "osmrelation": 42,
                },
            },
        });
        let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
        let count_value = context::tests::TestFileSystem::make_file();
        let files = context::tests::TestFileSystem::make_files(
            test_wsgi.get_ctx(),
            &[
                ("data/yamls.cache", &yamls_cache_value),
                ("workdir/gh611-additional-streets.count", &count_value),
            ],
        );
        let file_system = context::tests::TestFileSystem::from_files(&files);
        test_wsgi.get_ctx().set_file_system(&file_system);

        let root = test_wsgi.get_dom_for_path("/additional-streets/gh611/view-result");

        let mut guard = count_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
        let results = wsgi::tests::TestWsgi::find_all(&root, "body/table");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional streets page: if the output is well-formed, no osm streets case.
    #[test]
    fn test_streets_no_osm_streets_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_osm_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);

        let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

        let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-osm-streets']");
        assert_eq!(results.len(), 1);
    }

    /// Tests the additional streets page: if the output is well-formed, no ref streets case.
    #[test]
    fn test_streets_no_ref_streets_well_formed() {
        let mut test_wsgi = wsgi::tests::TestWsgi::new();
        let mut relations = areas::Relations::new(test_wsgi.get_ctx()).unwrap();
        let relation = relations.get_relation("gazdagret").unwrap();
        let hide_path = relation.get_files().get_ref_streets_path();
        let mut file_system = context::tests::TestFileSystem::new();
        file_system.set_hide_paths(&[hide_path]);
        let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
        test_wsgi.get_ctx().set_file_system(&file_system_arc);

        let root = test_wsgi.get_dom_for_path("/additional-streets/gazdagret/view-result");

        let results = wsgi::tests::TestWsgi::find_all(&root, "body/div[@id='no-ref-streets']");
        assert_eq!(results.len(), 1)
    }
}
