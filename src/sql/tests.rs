/*
 * Copyright 2023 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the sql module.

use super::*;
use crate::context;

/// Tests init().
#[test]
fn test_init() {
    let ctx = context::tests::make_test_context().unwrap();
    let conn = ctx.get_database_connection().unwrap();

    // Check that init() for an already up to date schema results in no errors.
    init(&conn).unwrap();
}
