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
struct LanguageContext<'a> {
    ctx: &'a context::Context,
}

impl<'a> LanguageContext<'a> {
    /// Switches to the new language.
    fn new(ctx: &'a context::Context, language: &str) -> Self {
        set_language(ctx, language);
        LanguageContext { ctx }
    }
}

impl<'a> Drop for LanguageContext<'a> {
    /// Switches back to the old language.
    fn drop(&mut self) {
        set_language(self.ctx, "en");
    }
}

/// Tests translate().
#[test]
fn test_translate() {
    let ctx = context::tests::make_test_context().unwrap();
    let _lc = LanguageContext::new(&ctx, "hu");
    assert_eq!(translate("Area"), "Terület");
}
