/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
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
    fn new(ctx: &context::Context, language: &str) -> Self {
        set_language(ctx, language);
        LanguageContext {}
    }
}

impl<'a> Drop for LanguageContext {
    /// Switches back to the old language.
    fn drop(&mut self) {
        reset_language();
    }
}

/// Resets the language.
pub fn reset_language() {
    LANGUAGE.with(|it| {
        *it.borrow_mut() = None;
    });
    TRANSLATIONS.with(|it| {
        *it.borrow_mut() = None;
    });
}

/// Tests translate().
#[test]
fn test_translate() {
    let ctx = context::tests::make_test_context().unwrap();
    let _lc = LanguageContext::new(&ctx, "hu");
    assert_eq!(translate("Area"), "Terület");
}

/// Tests get_language() when its value is None.
#[test]
fn test_get_language_none() {
    reset_language();

    assert_eq!(get_language(), "en");
}
