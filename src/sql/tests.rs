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

/// Tests ignore_unique_constraint(), when the error is a unique constraint violation.
#[test]
fn test_ignore_unique_constraint_mapped_to_ok() {
    let ret = ignore_unique_constraint(Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error {
            code: rusqlite::ErrorCode::Unknown,
            extended_code: 0,
        },
        None,
    )));

    assert!(ret.is_err());
}

/// Tests ignore_unique_constraint(), when the error is something else.
#[test]
fn test_ignore_unique_constraint_err() {
    let ret = ignore_unique_constraint(Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error {
            code: rusqlite::ErrorCode::ConstraintViolation,
            extended_code: rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE,
        },
        None,
    )));

    assert!(ret.is_ok());
}
