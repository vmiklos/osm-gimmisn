/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The ranges module contains functionality related to the Ranges class.

/// A range object represents an odd or even range of integer numbers.
#[derive(Clone, Copy, Debug)]
pub struct Range {
    start: i64,
    end: i64,
    is_odd: Option<bool>,
}

impl Range {
    pub fn new(start: i64, end: i64, interpolation: &str) -> Self {
        let mut is_odd = Some(start % 2 == 1);
        if interpolation == "all" {
            is_odd = None
        }
        Range { start, end, is_odd }
    }

    fn contains(&self, item: i64) -> bool {
        if let Some(is_odd) = self.is_odd {
            if is_odd != (item % 2 == 1) {
                return false;
            }
        }

        if self.start <= item && item <= self.end {
            return true;
        }

        false
    }
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        if self.start != other.start {
            return false;
        }

        if self.end != other.end {
            return false;
        }

        if self.is_odd != other.is_odd {
            return false;
        }

        true
    }
}

/// A Ranges object contains an item if any of its Range objects contains it.
#[derive(Clone, Debug)]
pub struct Ranges {
    items: Vec<Range>,
}

impl Ranges {
    pub fn new(items: Vec<Range>) -> Self {
        Ranges { items }
    }

    pub fn contains(&self, item: i64) -> bool {
        for i in &self.items {
            if i.contains(item) {
                return true;
            }
        }

        false
    }
}

impl PartialEq for Ranges {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Factory for Range without specifying interpolation.
    fn make_range(start: i64, end: i64) -> Range {
        Range::new(start, end, "".into())
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
}
