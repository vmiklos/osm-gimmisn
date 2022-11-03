/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the sync_ref module.

use super::*;
use std::sync::Arc;

/// Tests main().
#[test]
fn test_main() {
    let argv = vec![
        "".to_string(),
        "https://www.example.com/osm/data/".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://www.example.com/osm/data/",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/sync-ref.html",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let wsgi_ini_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/wsgi.ini.template", &wsgi_ini_template)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 0);
    let actual = ctx
        .get_file_system()
        .read_to_string(&ctx.get_abspath("data/wsgi.ini.template"))
        .unwrap();
    let expected = r#"[wsgi]
reference_housenumbers = refdir/hazszamok_20221001.tsv refdir/hazszamok_kieg_20221016.tsv
reference_street = refdir/utcak_20221016.tsv
reference_citycounts = refdir/varosok_count_20221001.tsv
reference_zipcounts = refdir/irsz_count_20221001.tsv
"#;
    assert_eq!(actual, expected);
}

/// Tests main(), missing URL.
#[test]
fn test_main_no_url() {
    let argv = vec!["".to_string()]; // No URL argument.
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 1);
}
