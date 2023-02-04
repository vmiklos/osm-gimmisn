/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the ranges module.

use super::*;

/// Factory for Range without specifying interpolation.
fn make_range(start: i64, end: i64) -> Range {
    Range::new(start, end, "")
}

/// Range: Tests an odd range with an even number.
#[test]
fn test_range_isodd_bad() {
    let test = make_range(1, 3);
    assert_eq!(test.contains(2), false);
}

/// Range: Tests an odd range with a large number.
#[test]
fn test_range_bad() {
    let test = make_range(1, 3);
    assert_eq!(test.contains(5), false);
}

/// Range: Tests the happy path.
#[test]
fn test_range_happy() {
    let test = make_range(1, 5);
    assert_eq!(test.contains(1), true);
    assert_eq!(test.contains(3), true);
    assert_eq!(test.contains(5), true);
    assert_eq!(test.start, 1);
    assert_eq!(test.end, 5);
}

/// Range: Tests equality code.
#[test]
fn test_range_eq() {
    assert_eq!(make_range(1, 5) != make_range(3, 5), true);
    assert_eq!(make_range(1, 5) != make_range(1, 3), true);
    assert_eq!(
        make_range(1, 3) != Range::new(1, 3, /*interpolation=*/ "all"),
        true
    );
}

/// Range: Tests the interpolation modes.
#[test]
fn test_range_interpolation_all() {
    assert_eq!(make_range(1, 3).contains(2), false);
    assert_eq!(Range::new(1, 3, /*interpolation=*/ "all").contains(2), true);
}

/// Range: test traits.
#[test]
fn test_range_traits() {
    let range = make_range(1, 3);
    assert_eq!(
        format!("{range:?}"),
        "Range { start: 1, end: 3, is_odd: Some(true) }"
    );
    let range2 = range.clone();
    assert_eq!(range2, range);
}

/// Ranges: Tests when the arg is in the first range.
#[test]
fn test_ranges_a() {
    let test = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
    assert_eq!(test.contains(0), true);
}

/// Ranges: Tests when the arg is in the second range.
#[test]
fn test_ranges_b() {
    let test = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
    assert_eq!(test.contains(1), true);
}

/// Ranges: Tests when the arg is in both ranges.
#[test]
fn test_ranges_ab() {
    let test = Ranges::new(vec![make_range(1, 1), make_range(1, 1)]);
    assert_eq!(test.contains(1), true);
}

/// Ranges: Tests when the arg is in neither ranges.
#[test]
fn test_ranges_none() {
    let test = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
    assert_eq!(test.contains(2), false);
}

/// Ranges: test traits.
#[test]
fn test_ranges_traits() {
    let ranges = Ranges::new(vec![make_range(0, 0), make_range(1, 1)]);
    let expected = "Ranges { items: [Range { start: 0, end: 0, is_odd: Some(false) }, Range { start: 1, end: 1, is_odd: Some(true) }] }";
    assert_eq!(format!("{ranges:?}"), expected);
}
