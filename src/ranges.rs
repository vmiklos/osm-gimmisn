/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The ranges module contains functionality related to the Ranges class.

/// A range object represents an odd or even range of integer numbers.
#[derive(Clone, Debug)]
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
        if let Some(is_odd) = self.is_odd
            && is_odd != (item % 2 == 1)
        {
            return false;
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
mod tests;
