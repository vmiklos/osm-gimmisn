/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the missing_housenumbers module.

use super::*;
use std::io::Read;
use std::io::Seek;
use std::rc::Rc;

/// Tests main().
#[test]
fn test_main() {
    let argv = vec!["".to_string(), "gh195".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 0);
    buf.rewind().unwrap();
    let mut actual: Vec<u8> = Vec::new();
    buf.read_to_end(&mut actual).unwrap();
    assert_eq!(
        String::from_utf8(actual).unwrap(),
        "Kalotaszeg utca\t3\n[\"25\", \"27-37\", \"31*\"]\n"
    );
}

/// Tests main(), the failing case.
#[test]
fn test_main_error() {
    let argv = vec!["".to_string(), "gh195".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_rc: Rc<dyn context::Unit> = Rc::new(unit);
    ctx.set_unit(&unit_rc);

    let ret = main(&argv, &mut buf, &mut ctx);

    assert_eq!(ret, 1);
}
