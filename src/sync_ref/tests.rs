/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the sync_ref module.

use super::*;
use std::ops::DerefMut as _;
use std::rc::Rc;

/// Tests main().
#[test]
fn test_main() {
    let argv = vec![
        "".to_string(),
        "--url".to_string(),
        "https://osm.example.com/data/".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://osm.example.com/data/",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/sync-ref.html",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let wsgi_ini_template = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/wsgi.ini.template", &wsgi_ini_template)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    let ret = main(&argv, &mut buf, &mut ctx);
    println!(
        "test_main: buf is '{}'",
        String::from_utf8(buf.into_inner()).unwrap()
    );

    assert_eq!(ret, 0);
    let actual = ctx
        .get_file_system()
        .read_to_string(&ctx.get_abspath("data/wsgi.ini.template"))
        .unwrap();
    let expected = r#"[wsgi]
reference_housenumbers = 'workdir/refs/hazszamok_20221001.tsv workdir/refs/hazszamok_kieg_20221016.tsv'
reference_street = 'workdir/refs/utcak_20221016.tsv'
reference_citycounts = 'workdir/refs/varosok_count_20221001.tsv'
reference_zipcounts = 'workdir/refs/irsz_count_20221001.tsv'
"#;
    assert_eq!(actual, expected);
}

/// Tests main(), the download mode.
#[test]
fn test_main_download() {
    let argv = vec![
        "".to_string(),
        "--mode".to_string(),
        "download".to_string(),
        "--url".to_string(),
        "https://osm.example.com/data/".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let wsgi_ini = context::tests::TestFileSystem::make_file();
    let wsgi_ini_template = context::tests::TestFileSystem::make_file();
    {
        let mut guard = wsgi_ini_template.borrow_mut();
        let write = guard.deref_mut();
        write
            .write_all(
                r#"[wsgi]
reference_housenumbers = 'workdir/refs/hazszamok_20190511.tsv workdir/refs/hazszamok_kieg_20190808.tsv'
reference_street = 'workdir/refs/utcak_20190514.tsv'
reference_citycounts = 'workdir/refs/varosok_count_20190717.tsv'
reference_zipcounts = 'workdir/refs/irsz_count_20200717.tsv'
"#
                .as_bytes(),
            )
            .unwrap();
    }
    let zipcount = context::tests::TestFileSystem::make_file();
    let zipcount_old = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/wsgi.ini.template", &wsgi_ini_template),
            ("workdir/wsgi.ini", &wsgi_ini),
            ("workdir/refs/irsz_count_20200717.tsv", &zipcount),
            ("workdir/refs/irsz_count_20190717.tsv", &zipcount_old),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_hide_paths(&[ctx.get_abspath("workdir/refs/irsz_count_20200717.tsv")]);
    let file_system_rc: Rc<dyn context::FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://osm.example.com/data/irsz_count_20200717.tsv",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/zipcount-new.tsv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_rc: Rc<dyn context::Network> = Rc::new(network);
    ctx.set_network(network_rc);

    let ret = main(&argv, &mut buf, &mut ctx);

    let buf = String::from_utf8(buf.into_inner()).unwrap();
    assert_eq!(
        buf,
        r#"sync-ref: downloading 'https://osm.example.com/data/irsz_count_20200717.tsv'...
sync-ref: removing 'workdir/refs/irsz_count_20190717.tsv'...
sync-ref: removing old index...
sync-ref: ok
"#
    );
    assert_eq!(ret, 0);
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
