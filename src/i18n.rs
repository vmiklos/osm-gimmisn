/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The i18n module allows UI translation via gettext.

use crate::context;

thread_local! {
    static TRANSLATIONS: std::cell::RefCell<Option<gettext::Catalog>> = std::cell::RefCell::new(None);
    static LANGUAGE: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

/// Sets the language of the current thread.
pub fn set_language(ctx: &context::Context, language: &str) {
    // Not using ctx.get_abspath() here, tests/ doesn't have its own dummy translations.
    let root_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!(
        "{}/locale/{}/LC_MESSAGES/osm-gimmisn.mo",
        root_dir, language
    );

    if ctx.get_file_system().path_exists(&path) {
        // The file exists, so this should not fail.
        let file = std::fs::File::open(path).expect("File::open() failed");
        // We produce this build-time, so this should not fail.
        let catalog = gettext::Catalog::parse(file).expect("Catalog::parse() failed");
        TRANSLATIONS.with(|it| {
            *it.borrow_mut() = Some(catalog);
        });
    } else {
        TRANSLATIONS.with(|it| {
            *it.borrow_mut() = None;
        });
    }
    LANGUAGE.with(|it| {
        *it.borrow_mut() = Some(String::from(language));
    });
}

/// Gets the language of the current thread.
pub fn get_language() -> String {
    LANGUAGE.with(|language| {
        let language = language.borrow();
        match *language {
            Some(ref language) => language.clone(),
            None => String::from("en"),
        }
    })
}

/// Translates English input according to the current UI language.
pub fn translate(english: &str) -> String {
    TRANSLATIONS.with(|translations| {
        let translations = translations.borrow();
        match *translations {
            Some(ref translations) => translations.gettext(english).to_string(),
            None => english.to_string(),
        }
    })
}

#[cfg(test)]
mod tests;
