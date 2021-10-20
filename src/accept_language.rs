/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The accept_language module parses an Accept-Language HTTP header.

#[cfg(test)]
mod tests {
    /// Tests accept_language::parse().
    #[test]
    fn test_accept_language_parse() {
        let parsed = accept_language::parse("hu,en;q=0.9,en-US;q=0.8");
        assert_eq!(parsed[0], "hu");
    }

    /// Tests accept_language::parse(): when the language is not explicitly set.
    #[test]
    fn test_accept_language_parse_english() {
        let parsed = accept_language::parse("en-US,en;q=0.5");
        assert_eq!(parsed[0], "en-US");
    }
}
