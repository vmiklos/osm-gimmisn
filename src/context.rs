/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Abstractions to help writing unit tests: filesystem, network, etc.

use anyhow::anyhow;
use pyo3::prelude::*;
use pyo3::types::PyInt;
use pyo3::types::PyString;
use pyo3::types::PyTuple;
use pyo3::types::PyUnicode;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

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
trait Network: Send + Sync {
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

/// Python wrapper around a Network.
#[pyclass]
pub struct PyNetwork {
    network: Arc<dyn Network>,
}

#[pymethods]
impl PyNetwork {
    fn urlopen(&self, url: &str, data: &str) -> (String, String) {
        match self.network.urlopen(url, data) {
            Ok(value) => (value, String::from("")),
            Err(err) => (String::from(""), err.to_string()),
        }
    }
}

/// Network implementation, backed by Python code.
struct PyAnyNetwork {
    network: Py<PyAny>,
}

impl PyAnyNetwork {
    fn new(network: Py<PyAny>) -> Self {
        PyAnyNetwork { network }
    }
}

impl Network for PyAnyNetwork {
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String> {
        Python::with_gil(|py| {
            let any = self.network.call_method1(py, "urlopen", (url, data))?;
            let tuple = match any.as_ref(py).downcast::<PyTuple>() {
                Ok(value) => value,
                _ => {
                    return Err(anyhow!("urlopen() didn't return a PyTuple"));
                }
            };

            let data = match tuple.get_item(0).downcast::<PyString>() {
                Ok(value) => value,
                _ => {
                    return Err(anyhow!("urlopen() didn't return a PyTuple(PyString, ...)"));
                }
            };

            let err = match tuple.get_item(1).downcast::<PyString>() {
                Ok(value) => value,
                _ => {
                    return Err(anyhow!("urlopen() didn't return a PyTuple(..., PyString)"));
                }
            };

            if err.len().unwrap() > 0 {
                return Err(anyhow!("urlopen() failed: {}", err));
            }

            Ok(data.to_string())
        })
    }
}

/// Time interface.
trait Time: Send + Sync {
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

/// Python wrapper around a Time.
#[pyclass]
pub struct PyTime {
    time: Arc<dyn Time>,
}

#[pymethods]
impl PyTime {
    fn now(&self) -> i64 {
        self.time.now()
    }

    fn sleep(&self, seconds: u64) {
        self.time.sleep(seconds)
    }
}

/// Time implementation, backed by Python code.
struct PyAnyTime {
    time: Py<PyAny>,
}

impl PyAnyTime {
    fn new(time: Py<PyAny>) -> Self {
        PyAnyTime { time }
    }
}

impl Time for PyAnyTime {
    fn now(&self) -> i64 {
        Python::with_gil(|py| {
            let any = match self.time.call_method0(py, "now") {
                Ok(value) => value,
                _ => {
                    return 0;
                }
            };
            let int = match any.as_ref(py).downcast::<PyInt>() {
                Ok(value) => value,
                _ => {
                    return 0;
                }
            };

            let ret: i64 = int.extract().unwrap();
            ret
        })
    }

    fn sleep(&self, seconds: u64) {
        Python::with_gil(|py| {
            self.time.call_method1(py, "sleep", (seconds,)).unwrap();
        })
    }
}

/// Subprocess interface.
trait Subprocess: Send + Sync {
    /// Runs a commmand, capturing its output.
    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> anyhow::Result<String>;
}

/// Subprocess implementation, backed by the Rust stdlib.
struct StdSubprocess {}

impl Subprocess for StdSubprocess {
    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> anyhow::Result<String> {
        let (first, rest) = args
            .split_first()
            .ok_or_else(|| anyhow!("args is an empty list"))?;
        let output = std::process::Command::new(first)
            .args(rest)
            .envs(&env)
            .output()?;
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    }
}

/// Python wrapper around a Subprocess.
#[pyclass]
pub struct PySubprocess {
    subprocess: Arc<dyn Subprocess>,
}

#[pymethods]
impl PySubprocess {
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

/// Subprocess implementation, backed by Python code.
struct PyAnySubprocess {
    subprocess: Py<PyAny>,
}

impl PyAnySubprocess {
    fn new(subprocess: Py<PyAny>) -> Self {
        PyAnySubprocess { subprocess }
    }
}

impl Subprocess for PyAnySubprocess {
    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> anyhow::Result<String> {
        Python::with_gil(|py| {
            let any = match self.subprocess.call_method1(py, "run", (args, env)) {
                Ok(value) => value,
                Err(err) => { return Err(anyhow!("failed to call run(): {}", err.to_string())); },
            };
            let string = match any.as_ref(py).downcast::<PyUnicode>() {
                Ok(value) => value,
                Err(err) => { return Err(anyhow!("failed to downcast to PyUnicode: {}", err.to_string())); },
            };
            Ok(string.extract().unwrap())
        })
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

/// Configuration file reader.
#[derive(Clone)]
struct Ini {
    config: configparser::ini::Ini,
    root: String,
}

impl Ini {
    fn new(config_path: &str, root: &str) -> anyhow::Result<Self> {
        let mut config = configparser::ini::Ini::new();
        let _ret = config.load(config_path);
        Ok(Ini {
            config,
            root: String::from(root),
        })
    }

    /// Gets the directory which is writable.
    fn get_workdir(&self) -> anyhow::Result<String> {
        let workdir = self
            .config
            .get("wsgi", "workdir")
            .ok_or_else(|| anyhow!("cannot get key workdir"))?;
        Ok(Path::new(&self.root)
            .join(&workdir)
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .to_string())
    }

    /// Gets the abs paths of ref housenumbers.
    fn get_reference_housenumber_paths(&self) -> anyhow::Result<Vec<String>> {
        let value = self
            .config
            .get("wsgi", "reference_housenumbers")
            .ok_or_else(|| anyhow!("cannot get key reference_housenumbers"))?;
        let relpaths = value.split(' ');
        relpaths
            .map(|relpath| -> anyhow::Result<String> {
                Ok(Path::new(&self.root)
                    .join(&relpath)
                    .to_str()
                    .ok_or_else(|| anyhow!("cannot convert path to string"))?
                    .to_string())
            })
            .collect::<anyhow::Result<Vec<String>>>()
    }

    /// Gets the abs path of ref streets.
    fn get_reference_street_path(&self) -> anyhow::Result<String> {
        let relpath = self
            .config
            .get("wsgi", "reference_street")
            .ok_or_else(|| anyhow!("cannot get key reference_street"))?;
        Ok(Path::new(&self.root)
            .join(&relpath)
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .to_string())
    }

    /// Gets the abs path of ref citycounts.
    fn get_reference_citycounts_path(&self) -> anyhow::Result<String> {
        let relpath = self
            .config
            .get("wsgi", "reference_citycounts")
            .ok_or_else(|| anyhow!("cannot get key reference_citycounts"))?;
        Ok(Path::new(&self.root)
            .join(&relpath)
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .to_string())
    }

    /// Gets the global URI prefix.
    fn get_uri_prefix(&self) -> anyhow::Result<String> {
        self.config
            .get("wsgi", "uri_prefix")
            .ok_or_else(|| anyhow!("cannot get key uri_prefix"))
    }

    /// Gets the TCP port to be used.
    fn get_tcp_port(&self) -> anyhow::Result<i64> {
        match self.config.get("wsgi", "tcp_port") {
            Some(value) => Ok(value.parse::<i64>()?),
            None => Ok(8000),
        }
    }

    /// Gets the URI of the overpass instance to be used.
    fn get_overpass_uri(&self) -> String {
        match self.config.get("wsgi", "overpass_uri") {
            Some(value) => value,
            None => String::from("https://overpass-api.de"),
        }
    }

    /// Should cron.py update inactive relations?
    fn get_cron_update_inactive(&self) -> bool {
        match self.config.get("wsgi", "cron_update_inactive") {
            Some(value) => value == "True",
            None => false,
        }
    }
}

#[pyclass]
pub struct PyIni {
    ini: Ini,
}

#[pymethods]
impl PyIni {
    #[new]
    fn new(config_path: &str, root: &str) -> PyResult<Self> {
        match Ini::new(config_path, root) {
            Ok(value) => Ok(PyIni { ini: value }),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::new() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_workdir(&self) -> PyResult<String> {
        match self.ini.get_workdir() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::get_workdir() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_reference_housenumber_paths(&self) -> PyResult<Vec<String>> {
        match self.ini.get_reference_housenumber_paths() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::get_reference_housenumber_paths() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_reference_street_path(&self) -> PyResult<String> {
        match self.ini.get_reference_street_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::get_reference_street_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_reference_citycounts_path(&self) -> PyResult<String> {
        match self.ini.get_reference_citycounts_path() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::get_reference_citycounts_path() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_uri_prefix(&self) -> PyResult<String> {
        match self.ini.get_uri_prefix() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::get_uri_prefix() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_tcp_port(&self) -> PyResult<i64> {
        match self.ini.get_tcp_port() {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Ini::get_tcp_port() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_overpass_uri(&self) -> String {
        self.ini.get_overpass_uri()
    }

    fn get_cron_update_inactive(&self) -> bool {
        self.ini.get_cron_update_inactive()
    }
}

/// Context owns global state which is set up once and then read everywhere.
struct Context {
    root: String,
    ini: Ini,
    network: Arc<dyn Network>,
    time: Arc<dyn Time>,
    subprocess: Arc<dyn Subprocess>,
}

impl Context {
    fn new(prefix: &str) -> anyhow::Result<Self> {
        let root_dir = env!("CARGO_MANIFEST_DIR");
        let root = Path::new(&root_dir)
            .join(&prefix)
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .to_string();
        let ini = Ini::new(
            Path::new(&root)
                .join("wsgi.ini")
                .to_str()
                .ok_or_else(|| anyhow!("cannot convert path to string"))?,
            &root,
        )?;
        let network = Arc::new(StdNetwork {});
        let time = Arc::new(StdTime {});
        let subprocess = Arc::new(StdSubprocess {});
        Ok(Context {
            root,
            ini,
            network,
            time,
            subprocess,
        })
    }

    /// Make a path absolute, taking the repo root as a base dir.
    fn get_abspath(&self, rel_path: &str) -> anyhow::Result<String> {
        Ok(Path::new(&self.root)
            .join(rel_path)
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .to_string())
    }

    fn get_ini(&self) -> &Ini {
        &self.ini
    }

    fn get_network(&self) -> &Arc<dyn Network> {
        &self.network
    }

    fn set_network(&mut self, network: &Arc<dyn Network>) {
        self.network = network.clone();
    }

    fn get_time(&self) -> &Arc<dyn Time> {
        &self.time
    }

    fn set_time(&mut self, time: &Arc<dyn Time>) {
        self.time = time.clone();
    }

    fn get_subprocess(&self) -> &Arc<dyn Subprocess> {
        &self.subprocess
    }

    fn set_subprocess(&mut self, subprocess: &Arc<dyn Subprocess>) {
        self.subprocess = subprocess.clone();
    }
}

#[pyclass]
pub struct PyContext {
    context: Context,
}

#[pymethods]
impl PyContext {
    #[new]
    fn new(prefix: &str) -> PyResult<Self> {
        match Context::new(prefix) {
            Ok(value) => Ok(PyContext { context: value }),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "Context::new() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_abspath(&self, rel_path: &str) -> PyResult<String> {
        match self.context.get_abspath(rel_path) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "context.get_abspath() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn get_ini(&self) -> PyIni {
        PyIni {
            ini: self.context.get_ini().clone(),
        }
    }

    fn get_network(&self) -> PyNetwork {
        PyNetwork {
            network: self.context.get_network().clone(),
        }
    }

    fn set_network(&mut self, network: &PyAny) {
        let network: Arc<dyn Network> = Arc::new(PyAnyNetwork::new(network.into()));
        self.context.set_network(&network);
    }

    fn get_time(&self) -> PyTime {
        PyTime {
            time: self.context.get_time().clone(),
        }
    }

    fn set_time(&mut self, time: &PyAny) {
        let time: Arc<dyn Time> = Arc::new(PyAnyTime::new(time.into()));
        self.context.set_time(&time);
    }

    fn get_subprocess(&self) -> PySubprocess {
        PySubprocess {
            subprocess: self.context.get_subprocess().clone(),
        }
    }

    fn set_subprocess(&mut self, subprocess: &PyAny) {
        let subprocess: Arc<dyn Subprocess> = Arc::new(PyAnySubprocess::new(subprocess.into()));
        self.context.set_subprocess(&subprocess);
    }
}
