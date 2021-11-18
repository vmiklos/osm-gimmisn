/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Generate HTML with Rust.
//!
//! This is more or less a Rust port of <https://www.yattag.org/>, mostly because
//! <https://crates.io/crates/html-builder> would require you to manually escape attribute values.

use std::sync::Arc;
use std::sync::Mutex;

/// Generates xml/html documents.
#[derive(Clone)]
pub struct Doc {
    value: Arc<Mutex<String>>,
}

impl Doc {
    pub fn new() -> Doc {
        Doc {
            value: Arc::new(Mutex::new(String::from(""))),
        }
    }

    /// Factory of yattag.Doc from a string.
    pub fn from_text(text: &str) -> Self {
        let doc = Doc::new();
        doc.text(text);
        doc
    }

    /// Gets the escaped value.
    pub fn get_value(&self) -> String {
        self.value.lock().unwrap().clone()
    }

    /// Appends escaped content to the value.
    pub fn append_value(&self, value: String) {
        self.value.lock().unwrap().push_str(&value)
    }

    /// Starts a new tag.
    pub fn tag(&self, name: &str, attrs: &[(&str, &str)]) -> Tag {
        Tag::new(&self.value, name, attrs)
    }

    /// Starts a new tag and closes it as well.
    pub fn stag(&self, name: &str, attrs: &[(&str, &str)]) {
        self.append_value(format!("<{}", name));
        for attr in attrs {
            let key = attr.0;
            let value = html_escape::encode_double_quoted_attribute(&attr.1);
            self.append_value(format!(" {}=\"{}\"", key, value));
        }
        self.append_value(String::from(" />"))
    }

    /// Appends unescaped content to the document.
    pub fn text(&self, text: &str) {
        let encoded = html_escape::encode_safe(text).to_string();
        self.append_value(encoded);
    }
}

pub type HtmlTable = Vec<Vec<Doc>>;

impl Default for Doc {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts a tag, which is closed automatically.
pub struct Tag {
    value: Arc<Mutex<String>>,
    name: String,
}

impl Tag {
    fn new(value: &Arc<Mutex<String>>, name: &str, attrs: &[(&str, &str)]) -> Tag {
        let mut locked_value = value.lock().unwrap();
        locked_value.push_str(&format!("<{}", name));
        for attr in attrs {
            let key = attr.0;
            let val = html_escape::encode_double_quoted_attribute(&attr.1);
            locked_value.push_str(&format!(" {}=\"{}\"", key, val));
        }
        locked_value.push('>');
        let value = value.clone();
        Tag {
            value,
            name: name.to_string(),
        }
    }
}

impl Drop for Tag {
    fn drop(&mut self) {
        self.value
            .lock()
            .unwrap()
            .push_str(&format!("</{}>", self.name));
    }
}
