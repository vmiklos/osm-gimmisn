/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the i18n module.

use super::*;

/// Context manager for translate().
struct LanguageContext {}

impl LanguageContext {
    /// Switches to the new language.
    fn new(language: &str) -> Self {
        set_language(language);
        LanguageContext {}
    }
}

impl Drop for LanguageContext {
    /// Switches back to the old language.
    fn drop(&mut self) {
        set_language("en");
    }
}

/// Tests translate().
#[test]
fn test_translate() {
    let _lc = LanguageContext::new("hu");
    assert_eq!(translate("Area"), "Terület");
}
