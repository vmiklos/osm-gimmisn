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
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub type BoxResult<T> = Result<T, Box<dyn std::error::Error>>;

/// File system interface.
trait FileSystem {
    /// Test whether a path exists.
    fn path_exists(&self, path: &str) -> bool;

    /// Return the last modification time of a file.
    fn getmtime(&self, path: &str) -> BoxResult<f64>;

    /// Opens a file for reading in binary mode.
    fn open_read(&self, path: &str) -> BoxResult<Box<dyn Read>>;

    /// Opens a file for writing in binary mode.
    fn open_write(&self, path: &str) -> BoxResult<Box<dyn Write>>;
}

/// File system implementation, backed by the Rust stdlib.
struct StdFileSystem {}

impl FileSystem for StdFileSystem {
    fn path_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn getmtime(&self, path: &str) -> BoxResult<f64> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        let mtime = modified.duration_since(std::time::SystemTime::UNIX_EPOCH)?;
        Ok(mtime.as_secs_f64())
    }

    fn open_read(&self, path: &str) -> BoxResult<Box<dyn Read>> {
        let ret: Box<dyn Read> = Box::new(std::fs::File::open(path)?);
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> BoxResult<Box<dyn Write>> {
        let ret: Box<dyn Write> = Box::new(std::fs::File::create(path)?);
        Ok(ret)
    }
}

/// Network interface.
trait Network {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> BoxResult<String>;
}

/// Network implementation, backed by the reqwest.
struct StdNetwork {}

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
        let network = StdNetwork {};
        PyStdNetwork { network }
    }

    fn urlopen(&self, url: &str, data: &str) -> (String, String) {
        match self.network.urlopen(url, data) {
            Ok(value) => (value, String::from("")),
            Err(err) => (String::from(""), err.to_string()),
        }
    }
}
