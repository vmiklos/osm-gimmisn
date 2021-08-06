/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Abstractions to help writing unit tests: filesystem, network, etc.

use pyo3::prelude::*;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Network interface.
trait Network {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> BoxResult<String>;
}

/// Network implementation, backed by the reqwest.
struct StdNetwork {
}

impl Network for StdNetwork {
    fn urlopen(&self, url: &str, data: &str) -> BoxResult<String> {
        if !data.is_empty() {
            let client = reqwest::blocking::Client::new();
            let body = String::from(data);
            let buf = client.post(url).body(body).send()?;
            return Ok(buf.text()?);
        }

        let buf = reqwest::blocking::get(url)?;

        Ok(buf.text()?)
    }
}

#[pyclass]
pub struct PyStdNetwork {
    network: StdNetwork,
}

#[pymethods]
impl PyStdNetwork {
    #[new]
    fn new() -> Self {
        let network = StdNetwork{};
        PyStdNetwork { network }
    }

    fn urlopen(&self, url: &str, data: &str) -> (String, String) {
        match self.network.urlopen(url, data) {
            Ok(value) => (value, String::from("")),
            Err(err) => (String::from(""), err.to_string())
        }
    }
}
