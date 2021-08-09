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
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use anyhow::anyhow;

/// File system interface.
trait FileSystem {
    /// Test whether a path exists.
    fn path_exists(&self, path: &str) -> bool;

    /// Return the last modification time of a file.
    fn getmtime(&self, path: &str) -> anyhow::Result<f64>;

    /// Opens a file for reading in binary mode.
    fn open_read(&self, path: &str) -> anyhow::Result<Box<dyn Read>>;

    /// Opens a file for writing in binary mode.
    fn open_write(&self, path: &str) -> anyhow::Result<Box<dyn Write>>;
}

/// File system implementation, backed by the Rust stdlib.
struct StdFileSystem {}

impl FileSystem for StdFileSystem {
    fn path_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn getmtime(&self, path: &str) -> anyhow::Result<f64> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        let mtime = modified.duration_since(std::time::SystemTime::UNIX_EPOCH)?;
        Ok(mtime.as_secs_f64())
    }

    fn open_read(&self, path: &str) -> anyhow::Result<Box<dyn Read>> {
        let ret: Box<dyn Read> = Box::new(std::fs::File::open(path)?);
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Box<dyn Write>> {
        let ret: Box<dyn Write> = Box::new(std::fs::File::create(path)?);
        Ok(ret)
    }
}

#[pyclass]
pub struct PyStdFileSystem {
    file_system: StdFileSystem,
}

#[pymethods]
impl PyStdFileSystem {
    #[new]
    fn new() -> Self {
        let file_system = StdFileSystem {};
        PyStdFileSystem { file_system }
    }

    fn path_exists(&self, path: &str) -> bool {
        self.file_system.path_exists(path)
    }

    fn getmtime(&self, path: &str) -> PyResult<f64> {
        match self.file_system.getmtime(path) {
            Ok(value) => Ok(value),
            Err(_) => Err(pyo3::exceptions::PyIOError::new_err("getmtime() failed")),
        }
    }
}

/// Network interface.
trait Network {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String>;
}

/// Network implementation, backed by the reqwest.
struct StdNetwork {}

impl Network for StdNetwork {
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String> {
        if !data.is_empty() {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(425))
                .build()?;
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

/// Time interface.
trait Time {
    /// Calculates the current Unix timestamp from GMT.
    fn now(&self) -> i64;

    /// Delay execution for a given number of seconds.
    fn sleep(&self, seconds: u64);
}

/// Time implementation, backed by the chrono.
struct StdTime {}

impl Time for StdTime {
    fn now(&self) -> i64 {
        let now = chrono::Local::now();
        now.naive_local().timestamp()
    }

    fn sleep(&self, seconds: u64) {
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }
}

#[pyclass]
pub struct PyStdTime {
    time: StdTime,
}

#[pymethods]
impl PyStdTime {
    #[new]
    fn new() -> Self {
        let time = StdTime {};
        PyStdTime { time }
    }

    fn now(&self) -> i64 {
        self.time.now()
    }

    fn sleep(&self, seconds: u64) {
        self.time.sleep(seconds)
    }
}

/// Subprocess interface.
trait Subprocess {
    /// Runs a commmand, capturing its output.
    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> anyhow::Result<String>;
}

/// Subprocess implementation, backed by the Rust stdlib.
struct StdSubprocess {}

impl Subprocess for StdSubprocess {
    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> anyhow::Result<String> {
        let (first, rest) = args.split_first().ok_or(anyhow!("option::NoneError"))?;
        let output = std::process::Command::new(first)
            .args(rest)
            .envs(&env)
            .output()?;
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    }
}

#[pyclass]
pub struct PyStdSubprocess {
    subprocess: StdSubprocess,
}

#[pymethods]
impl PyStdSubprocess {
    #[new]
    fn new() -> Self {
        let subprocess = StdSubprocess {};
        PyStdSubprocess { subprocess }
    }

    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> PyResult<String> {
        match self.subprocess.run(args, env) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "failed to run: {}",
                err.to_string()
            ))),
        }
    }
}

/// Unit testing interface.
trait Unit {
    /// Injects a fake error.
    fn make_error(&self) -> String;
}

/// Unit implementation, which intentionally does nothing.
struct StdUnit {}

impl Unit for StdUnit {
    fn make_error(&self) -> String {
        String::from("")
    }
}

#[pyclass]
pub struct PyStdUnit {
    unit: StdUnit,
}

#[pymethods]
impl PyStdUnit {
    #[new]
    fn new() -> Self {
        let unit = StdUnit {};
        PyStdUnit { unit }
    }

    fn make_error(&self) -> String {
        self.unit.make_error()
    }
}
