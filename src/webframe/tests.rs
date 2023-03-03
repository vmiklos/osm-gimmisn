/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the webframe module.

use super::*;
use crate::context::Unit;
use std::io::Write;
use std::sync::Arc;

/// Tests handle_static().
#[test]
fn test_handle_static() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let css = context::tests::TestFileSystem::make_file();
    {
        let mut guard = css.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"/* comment */").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files =
        context::tests::TestFileSystem::make_files(&ctx, &[("target/browser/osm.min.css", &css)]);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("target/browser/osm.min.css");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/osm.min.css")).unwrap();

    assert_eq!(content.is_empty(), false);
    assert_eq!(content_type, "text/css; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the generated javascript case.
#[test]
fn test_handle_static_generated_javascript() {
    let ctx = context::tests::make_test_context().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/bundle.js")).unwrap();
    assert_eq!("// bundle.js\n".as_bytes(), content);
    assert_eq!(content_type, "application/x-javascript; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the json case.
#[test]
fn test_handle_static_json() {
    let ctx = context::tests::make_test_context().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/stats-empty.json")).unwrap();
    assert_eq!(content.starts_with(b"{"), true);
    assert_eq!(content_type, "application/json; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the ico case.
#[test]
fn test_handle_static_ico() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let ico = context::tests::TestFileSystem::make_file();
    {
        let mut guard = ico.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"\0").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[("favicon.ico", &ico)]);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("favicon.ico");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let (content, content_type, extra_headers) = handle_static(&ctx, "/favicon.ico").unwrap();

    assert_eq!(content.is_empty(), false);
    assert_eq!(content_type, "image/x-icon");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests handle_static: the svg case.
#[test]
fn test_handle_static_svg() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let svg = context::tests::TestFileSystem::make_file();
    {
        let mut guard = svg.borrow_mut();
        let write = guard.deref_mut();
        write.write_all(b"<svg").unwrap();
    }
    let mut file_system = context::tests::TestFileSystem::new();
    let files = context::tests::TestFileSystem::make_files(&ctx, &[("favicon.svg", &svg)]);
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    let path = ctx.get_abspath("favicon.svg");
    mtimes.insert(
        path,
        Rc::new(RefCell::new(time::OffsetDateTime::UNIX_EPOCH)),
    );
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let (content, content_type, extra_headers) = handle_static(&ctx, "/favicon.svg").unwrap();

    assert_eq!(content.is_empty(), false);
    assert_eq!(content_type, "image/svg+xml; charset=utf-8");
    assert_eq!(extra_headers.len(), 1);
    assert_eq!(extra_headers[0].0, "Last-Modified");
}

/// Tests the case when the content type is not recognized.
#[test]
fn test_handle_static_else() {
    let ctx = context::tests::make_test_context().unwrap();
    let prefix = ctx.get_ini().get_uri_prefix();
    let (content, content_type, extra_headers) =
        handle_static(&ctx, &format!("{prefix}/static/test.xyz")).unwrap();
    assert_eq!(content.is_empty(), true);
    assert_eq!(content_type.is_empty(), true);
    // No last modified non-existing file.
    assert_eq!(extra_headers.is_empty(), true);
}

/// Tests fill_missing_header_items().
#[test]
fn test_fill_missing_header_items() {
    let streets = "no";
    let relation_name = "gazdagret";
    let mut items: Vec<yattag::Doc> = Vec::new();
    let additional_housenumbers = true;
    let ctx = context::tests::make_test_context().unwrap();
    items = fill_missing_header_items(
        &ctx,
        streets,
        additional_housenumbers,
        relation_name,
        &items,
    )
    .unwrap();
    let html = items[0].get_value();
    assert_eq!(html.contains("Missing house numbers"), true);
    assert_eq!(html.contains("Missing streets"), false);
}

/// Tests handle_error().
#[test]
fn test_handle_error() {
    let headers = vec![("Accept-Language".to_string(), ",".to_string())];
    let request = rouille::Request::fake_http("GET", "/", headers, vec![]);

    let unit = context::tests::TestUnit::new();
    let err = unit.make_error();

    let response = handle_error(&request, &format!("{err:?}"));
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

/// Tests get_toolbar().
#[test]
fn test_get_toolbar() {
    let ctx = context::tests::make_test_context().unwrap();

    let ret = get_toolbar(&ctx, None, "myfunc", "myrel", 42).unwrap();

    assert_eq!(ret.get_value().is_empty(), false);
}
