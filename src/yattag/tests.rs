/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the yattag module.

use super::*;

/// Tests the required escaping.
#[test]
fn test_escape() {
    let doc = Doc::default();
    {
        let a = doc.tag("a", &[("href", r#"https://www.example.com/"x"#)]);
        a.text("here>y");
    }
    assert_eq!(
        doc.get_value(),
        r#"<a href="https://www.example.com/&quot;x">here&gt;y</a>"#
    );
}
