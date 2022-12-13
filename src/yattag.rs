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

use std::cell::RefCell;
use std::fmt::Write as _;
use std::rc::Rc;

/// Generates xml/html documents.
#[derive(Clone)]
pub struct Doc {
    value: Rc<RefCell<String>>,
}

impl Doc {
    /// Creates an empty Doc.
    pub fn new() -> Doc {
        Doc {
            value: Rc::new(RefCell::new(String::from(""))),
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
        self.value.borrow().clone()
    }

    /// Appends escaped content to the value.
    pub fn append_value(&self, value: String) {
        self.value.borrow_mut().push_str(&value)
    }

    /// Starts a new tag.
    pub fn tag(&self, name: &str, attrs: &[(&str, &str)]) -> Tag {
        Tag::new(&self.value, name, attrs)
    }

    /// Starts a new tag and closes it as well.
    pub fn stag(&self, name: &str) {
        self.append_value(format!("<{} />", name));
    }

    /// Appends unescaped content to the document.
    pub fn text(&self, text: &str) {
        let encoded = html_escape::encode_safe(text).to_string();
        self.append_value(encoded);
    }
}

/// HtmlTable is a matrix (rows, then cols) of Doc instances.
pub type HtmlTable = Vec<Vec<Doc>>;

impl Default for Doc {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts a tag, which is closed automatically.
pub struct Tag {
    value: Rc<RefCell<String>>,
    name: String,
}

impl Tag {
    fn new(value: &Rc<RefCell<String>>, name: &str, attrs: &[(&str, &str)]) -> Tag {
        let mut guard = value.borrow_mut();
        guard.push_str(&format!("<{}", name));
        for attr in attrs {
            let key = attr.0;
            let val = html_escape::encode_double_quoted_attribute(&attr.1);
            guard.push_str(&format!(" {}=\"{}\"", key, val));
        }
        guard.push('>');
        let value = value.clone();
        Tag {
            value,
            name: name.to_string(),
        }
    }

    /// Appends unescaped content inside a tag.
    pub fn text(&self, text: &str) {
        let encoded = html_escape::encode_safe(text).to_string();
        self.value.borrow_mut().push_str(&encoded)
    }

    /// Starts a new tag inside a tag.
    pub fn tag(&self, name: &str, attrs: &[(&str, &str)]) -> Tag {
        Tag::new(&self.value, name, attrs)
    }

    /// Starts a new tag and closes it as well, inside a tag.
    pub fn stag(&self, name: &str, attrs: &[(&str, &str)]) {
        self.append_value(format!("<{}", name));
        for attr in attrs {
            let key = attr.0;
            let value = html_escape::encode_double_quoted_attribute(&attr.1);
            self.append_value(format!(" {}=\"{}\"", key, value));
        }
        self.append_value(String::from("/>"))
    }

    /// Appends escaped content inside a tag.
    pub fn append_value(&self, value: String) {
        self.value.borrow_mut().push_str(&value)
    }
}

impl Drop for Tag {
    fn drop(&mut self) {
        let _ = write!(self.value.borrow_mut(), "</{}>", self.name);
    }
}

#[cfg(test)]
mod tests;
