/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the missing_housenumbers module.

use super::*;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

/// Tests main().
#[test]
fn test_main() {
    let argv = vec!["".to_string(), "gh195".to_string()];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    let mut ctx = context::tests::make_test_context().unwrap();

    main(&argv, &mut buf, &mut ctx).unwrap();

    buf.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    buf.read_to_end(&mut actual).unwrap();
    assert_eq!(
        actual,
        b"Kalotaszeg utca\t3\n[\"25\", \"27-37\", \"31*\"]\n"
    );
}
