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
use pyo3::class::PyIterProtocol;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBool;
use pyo3::types::PyBytes;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;
use pyo3::types::PyString;
use pyo3::types::PyType;
use pyo3::types::PyUnicode;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

/// File system interface.
pub trait FileSystem: Send + Sync {
    /// Test whether a path exists.
    fn path_exists(&self, path: &str) -> bool;

    /// Return the last modification time of a file.
    fn getmtime(&self, path: &str) -> anyhow::Result<f64>;

    /// Opens a file for reading in binary mode.
    fn open_read(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>>;

    /// Opens a file for writing in binary mode.
    fn open_write(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>>;
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

    fn open_read(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        let ret: Arc<Mutex<dyn Read + Send>> = Arc::new(Mutex::new(std::fs::File::open(path)?));
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        use anyhow::Context;
        let ret: Arc<Mutex<dyn Write + Send>> = Arc::new(Mutex::new(
            std::fs::File::create(path)
                .with_context(|| format!("failed to open {} for writing", path))?,
        ));
        Ok(ret)
    }
}

/// Iterator for PyRead.
#[pyclass]
pub struct PyReadIter {
    inner: std::vec::IntoIter<String>,
}

#[pyproto]
impl PyIterProtocol for PyReadIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        let string: String;
        match slf.inner.next() {
            Some(value) => string = value,
            None => {
                return None;
            }
        };
        let buf: Vec<u8> = string.into_bytes();
        Python::with_gil(|py| Some(PyBytes::new(py, &buf).into()))
    }
}

/// File-like object, wrapping a Read.
#[pyclass]
pub struct PyRead {
    /// The underlying Rust Read.
    pub read: Arc<Mutex<dyn Read + Send>>,
}

#[pymethods]
impl PyRead {
    fn read(&mut self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let mut buf: Vec<u8> = Vec::new();
            self.read.lock().unwrap().read_to_end(&mut buf)?;
            Ok(PyBytes::new(py, &buf).into())
        })
    }

    fn close(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __enter__(&self) -> Self {
        let read = self.read.clone();
        PyRead { read }
    }

    fn __exit__(
        &mut self,
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> bool {
        let gil = Python::acquire_gil();
        ty == Some(gil.python().get_type::<PyValueError>())
    }
}

#[pyproto]
impl PyIterProtocol for PyRead {
    fn __iter__(slf: PyRef<Self>) -> PyResult<Py<PyReadIter>> {
        let mut guard = slf.read.lock().unwrap();
        let mut reader = std::io::BufReader::new(guard.deref_mut());
        let mut lines: Vec<String> = Vec::new();
        loop {
            let mut line = String::new();
            if let Ok(len) = reader.read_line(&mut line) {
                if len == 0 {
                    break;
                }
                lines.push(line);
                continue;
            }
            break;
        }
        let iter = PyReadIter {
            inner: lines.into_iter(),
        };
        Py::new(slf.py(), iter)
    }
}

/// File-like object, wrapping a Write.
#[pyclass]
pub struct PyWrite {
    /// The underlying Rust Write.
    pub write: Arc<Mutex<dyn Write + Send>>,
}

#[pymethods]
impl PyWrite {
    fn write(&mut self, buf: &[u8]) -> PyResult<usize> {
        let mut guard = self.write.lock().unwrap();
        match guard.write_all(buf) {
            Ok(_) => Ok(buf.len()),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "write() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn close(&mut self) -> PyResult<()> {
        Ok(())
    }

    fn __enter__(&self) -> Self {
        let write = self.write.clone();
        PyWrite { write }
    }

    fn __exit__(
        &mut self,
        ty: Option<&PyType>,
        _value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> bool {
        let gil = Python::acquire_gil();
        ty == Some(gil.python().get_type::<PyValueError>())
    }
}

/// Write implementation, backed by Python.
pub struct PyAnyWrite {
    /// The underlying Python object.
    pub write: Py<PyAny>,
}

impl Write for PyAnyWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Python::with_gil(|py| {
            let any = match self.write.call_method1(py, "write", (buf,)) {
                Ok(value) => value,
                Err(err) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("failed to write: {}", err.to_string()),
                    ));
                }
            };
            let size = match any.as_ref(py).downcast::<PyInt>() {
                Ok(value) => value,
                Err(err) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("failed to downcast to PyInt: {}", err.to_string()),
                    ));
                }
            };
            let ret: usize = size.extract().unwrap();
            Ok(ret)
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for PyAnyWrite {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.write.call_method0(py, "close").unwrap();
        })
    }
}

/// Read implementation, backed by Python.
struct PyAnyRead {
    cursor: std::io::Cursor<Vec<u8>>,
}

impl Read for PyAnyRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

/// Python wrapper around a FileSystem.
#[pyclass]
pub struct PyFileSystem {
    file_system: Arc<dyn FileSystem>,
}

#[pymethods]
impl PyFileSystem {
    fn path_exists(&self, path: &str) -> bool {
        self.file_system.path_exists(path)
    }

    fn getmtime(&self, path: &str) -> PyResult<f64> {
        match self.file_system.getmtime(path) {
            Ok(value) => Ok(value),
            Err(_) => Err(pyo3::exceptions::PyIOError::new_err("getmtime() failed")),
        }
    }

    fn open_read(&self, path: &str) -> PyResult<PyRead> {
        match self.file_system.open_read(path) {
            Ok(value) => Ok(PyRead { read: value }),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "open_read() failed: {}",
                err.to_string()
            ))),
        }
    }

    fn open_write(&self, path: &str) -> PyResult<PyWrite> {
        match self.file_system.open_write(path) {
            Ok(value) => Ok(PyWrite { write: value }),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
                "open_write() failed: {}",
                err.to_string()
            ))),
        }
    }
}

/// FileSystem implementation, backed by Python code.
struct PyAnyFileSystem {
    file_system: Py<PyAny>,
}

impl PyAnyFileSystem {
    fn new(file_system: Py<PyAny>) -> Self {
        PyAnyFileSystem { file_system }
    }
}

impl FileSystem for PyAnyFileSystem {
    fn path_exists(&self, path: &str) -> bool {
        Python::with_gil(|py| {
            let any = match self.file_system.call_method1(py, "path_exists", (path,)) {
                Ok(value) => value,
                _ => {
                    return false;
                }
            };
            let boolean = match any.as_ref(py).downcast::<PyBool>() {
                Ok(value) => value,
                _ => {
                    return false;
                }
            };
            let ret: bool = boolean.extract().unwrap();
            ret
        })
    }

    fn getmtime(&self, path: &str) -> anyhow::Result<f64> {
        Python::with_gil(|py| {
            let any = match self.file_system.call_method1(py, "getmtime", (path,)) {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!("failed to call getmtime(): {}", err.to_string()));
                }
            };
            let float = match any.as_ref(py).downcast::<PyFloat>() {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!(
                        "failed to downcast to PyFloat: {}",
                        err.to_string()
                    ));
                }
            };
            let ret: f64 = float.extract().unwrap();
            Ok(ret)
        })
    }

    fn open_read(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
        Python::with_gil(|py| {
            let binaryio = match self.file_system.call_method1(py, "open_read", (path,)) {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!(
                        "failed to call open_read('{}'): {}",
                        path,
                        err.to_string()
                    ));
                }
            };
            let any = match binaryio.call_method0(py, "read") {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!("failed to call read(): {}", err.to_string()));
                }
            };
            let bytes = match any.as_ref(py).downcast::<PyBytes>() {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!(
                        "failed to downcast to PyBytes: {}",
                        err.to_string()
                    ));
                }
            };
            let cursor: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(bytes.extract().unwrap());
            binaryio.call_method0(py, "close").unwrap();
            let inner = PyAnyRead { cursor };
            let ret: Arc<Mutex<dyn Read + Send>> = Arc::new(Mutex::new(inner));
            Ok(ret)
        })
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        Python::with_gil(|py| {
            let write = match self.file_system.call_method1(py, "open_write", (path,)) {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!("failed to call open_write(): {}", err.to_string()));
                }
            };
            let inner = PyAnyWrite { write };
            let ret: Arc<Mutex<dyn Write + Send>> = Arc::new(Mutex::new(inner));
            Ok(ret)
        })
    }
}

/// Network interface.
pub trait Network: Send + Sync {
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
    fn urlopen(&self, url: &str, data: &str) -> PyResult<String> {
        match self.network.urlopen(url, data) {
            Ok(value) => Ok(value),
            Err(err) => Err(pyo3::exceptions::PyOSError::new_err(err.to_string())),
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
            let data = match any.as_ref(py).downcast::<PyString>() {
                Ok(value) => value,
                _ => {
                    return Err(anyhow!("urlopen() didn't return a PyString"));
                }
            };

            Ok(data.to_string())
        })
    }
}

/// Time interface.
pub trait Time: Send + Sync {
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
pub trait Subprocess: Send + Sync {
    /// Runs a commmand, capturing its output.
    fn run(&self, args: Vec<String>, env: HashMap<String, String>) -> anyhow::Result<String>;

    /// Terminates the current process with the specified exit code.
    fn exit(&self, code: i32);
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

    fn exit(&self, code: i32) {
        std::process::exit(code);
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

    fn exit(&self, code: i32) {
        self.subprocess.exit(code)
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
                Err(err) => {
                    return Err(anyhow!("failed to call run(): {}", err.to_string()));
                }
            };
            let string = match any.as_ref(py).downcast::<PyUnicode>() {
                Ok(value) => value,
                Err(err) => {
                    return Err(anyhow!(
                        "failed to downcast to PyUnicode: {}",
                        err.to_string()
                    ));
                }
            };
            Ok(string.extract().unwrap())
        })
    }

    fn exit(&self, code: i32) {
        let gil = Python::acquire_gil();
        self.subprocess
            .call_method1(gil.python(), "exit", (code,))
            .unwrap();
    }
}

/// Unit testing interface.
pub trait Unit: Send + Sync {
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

/// Python wrapper around a Unit.
#[pyclass]
pub struct PyUnit {
    unit: Arc<dyn Unit>,
}

#[pymethods]
impl PyUnit {
    fn make_error(&self) -> String {
        self.unit.make_error()
    }
}

/// Unit implementation, backed by Python code.
struct PyAnyUnit {
    unit: Py<PyAny>,
}

impl PyAnyUnit {
    fn new(unit: Py<PyAny>) -> Self {
        PyAnyUnit { unit }
    }
}

impl Unit for PyAnyUnit {
    fn make_error(&self) -> String {
        Python::with_gil(|py| {
            let any = match self.unit.call_method0(py, "make_error") {
                Ok(value) => value,
                Err(_) => {
                    return String::from("");
                }
            };
            let string = match any.as_ref(py).downcast::<PyUnicode>() {
                Ok(value) => value,
                Err(_) => {
                    return String::from("");
                }
            };
            string.extract().unwrap()
        })
    }
}

/// Configuration file reader.
#[derive(Clone)]
pub struct Ini {
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
    pub fn get_workdir(&self) -> anyhow::Result<String> {
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
    pub fn get_reference_housenumber_paths(&self) -> anyhow::Result<Vec<String>> {
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
    pub fn get_reference_street_path(&self) -> anyhow::Result<String> {
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
    pub fn get_reference_citycounts_path(&self) -> anyhow::Result<String> {
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
    pub fn get_uri_prefix(&self) -> anyhow::Result<String> {
        self.config
            .get("wsgi", "uri_prefix")
            .ok_or_else(|| anyhow!("cannot get key uri_prefix"))
    }

    /// Gets the TCP port to be used.
    pub fn get_tcp_port(&self) -> anyhow::Result<i64> {
        match self.config.get("wsgi", "tcp_port") {
            Some(value) => Ok(value.parse::<i64>()?),
            None => Ok(8000),
        }
    }

    /// Gets the URI of the overpass instance to be used.
    pub fn get_overpass_uri(&self) -> String {
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

/// Python wrapper around a Rust Ini.
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

    fn get_overpass_uri(&self) -> String {
        self.ini.get_overpass_uri()
    }

    fn get_cron_update_inactive(&self) -> bool {
        self.ini.get_cron_update_inactive()
    }
}

/// Context owns global state which is set up once and then read everywhere.
#[derive(Clone)]
pub struct Context {
    root: String,
    ini: Ini,
    network: Arc<dyn Network>,
    time: Arc<dyn Time>,
    subprocess: Arc<dyn Subprocess>,
    unit: Arc<dyn Unit>,
    file_system: Arc<dyn FileSystem>,
}

impl Context {
    /// Creates a new Context.
    pub fn new(prefix: &str) -> anyhow::Result<Self> {
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
        let unit = Arc::new(StdUnit {});
        let file_system = Arc::new(StdFileSystem {});
        Ok(Context {
            root,
            ini,
            network,
            time,
            subprocess,
            unit,
            file_system,
        })
    }

    /// Make a path absolute, taking the repo root as a base dir.
    pub fn get_abspath(&self, rel_path: &str) -> anyhow::Result<String> {
        Ok(Path::new(&self.root)
            .join(rel_path)
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?
            .to_string())
    }

    /// Gets the ini file.
    pub fn get_ini(&self) -> &Ini {
        &self.ini
    }

    /// Gets the network implementation.
    pub fn get_network(&self) -> &Arc<dyn Network> {
        &self.network
    }

    fn set_network(&mut self, network: &Arc<dyn Network>) {
        self.network = network.clone();
    }

    /// Gets the time implementation.
    pub fn get_time(&self) -> &Arc<dyn Time> {
        &self.time
    }

    /// Sets the time implementation.
    pub fn set_time(&mut self, time: &Arc<dyn Time>) {
        self.time = time.clone();
    }

    /// Gets the subprocess implementation.
    pub fn get_subprocess(&self) -> &Arc<dyn Subprocess> {
        &self.subprocess
    }

    fn set_subprocess(&mut self, subprocess: &Arc<dyn Subprocess>) {
        self.subprocess = subprocess.clone();
    }

    /// Gets the testing interface.
    pub fn get_unit(&self) -> &Arc<dyn Unit> {
        &self.unit
    }

    fn set_unit(&mut self, unit: &Arc<dyn Unit>) {
        self.unit = unit.clone();
    }

    /// Gets the file system implementation.
    pub fn get_file_system(&self) -> &Arc<dyn FileSystem> {
        &self.file_system
    }

    /// Sets the file system implementation.
    pub fn set_file_system(&mut self, file_system: &Arc<dyn FileSystem>) {
        self.file_system = file_system.clone();
    }
}

#[pyclass]
#[derive(Clone)]
/// Python wrapper around a Rust Context.
pub struct PyContext {
    /// The underlying Rust Context.
    pub context: Context,
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

    fn get_unit(&self) -> PyUnit {
        PyUnit {
            unit: self.context.get_unit().clone(),
        }
    }

    fn set_unit(&mut self, unit: &PyAny) {
        let unit: Arc<dyn Unit> = Arc::new(PyAnyUnit::new(unit.into()));
        self.context.set_unit(&unit);
    }

    fn get_file_system(&self) -> PyFileSystem {
        PyFileSystem {
            file_system: self.context.get_file_system().clone(),
        }
    }

    fn set_file_system(&mut self, file_system: &PyAny) {
        let file_system: Arc<dyn FileSystem> = Arc::new(PyAnyFileSystem::new(file_system.into()));
        self.context.set_file_system(&file_system);
    }
}

/// Registers Python wrappers of Rust structs into the Python module.
pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_class::<PyIni>()?;
    module.add_class::<PyContext>()?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io::Seek;
    use std::io::SeekFrom;

    /// Creates a Context instance for text purposes.
    pub fn make_test_context() -> anyhow::Result<Context> {
        Ok(Context::new("tests")?)
    }

    /// File system implementation, for test purposes.
    pub struct TestFileSystem {
        hide_paths: Vec<String>,
        mtimes: HashMap<String, f64>,
        files: HashMap<String, Arc<Mutex<std::io::Cursor<Vec<u8>>>>>,
    }

    impl TestFileSystem {
        pub fn new() -> Self {
            TestFileSystem {
                hide_paths: Vec::new(),
                mtimes: HashMap::new(),
                files: HashMap::new(),
            }
        }

        /// Sets the hide paths.
        pub fn set_hide_paths(&mut self, hide_paths: &[String]) {
            self.hide_paths = hide_paths.to_vec();
        }

        /*/// Sets the mtimes.
        fn set_mtimes(&mut self, mtimes: &HashMap<String, f64>) {
            self.mtimes = mtimes.clone();
        }*/

        /// Sets the files.
        pub fn set_files(&mut self, files: &HashMap<String, Arc<Mutex<std::io::Cursor<Vec<u8>>>>>) {
            self.files = files.clone()
        }
    }

    impl FileSystem for TestFileSystem {
        fn path_exists(&self, path: &str) -> bool {
            if self.hide_paths.contains(&path.to_string()) {
                return false;
            }

            if self.files.contains_key(path) {
                return true;
            }

            Path::new(path).exists()
        }

        fn getmtime(&self, path: &str) -> anyhow::Result<f64> {
            if self.mtimes.contains_key(path) {
                return Ok(self.mtimes[path]);
            }

            let metadata = std::fs::metadata(path)?;
            let modified = metadata.modified()?;
            let mtime = modified.duration_since(std::time::SystemTime::UNIX_EPOCH)?;
            Ok(mtime.as_secs_f64())
        }

        fn open_read(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Read + Send>>> {
            if self.files.contains_key(path) {
                let ret = self.files[path].clone();
                ret.lock().unwrap().seek(SeekFrom::Start(0))?;
                return Ok(ret);
            }
            let ret: Arc<Mutex<dyn Read + Send>> = Arc::new(Mutex::new(std::fs::File::open(path)?));
            Ok(ret)
        }

        fn open_write(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
            if self.files.contains_key(path) {
                let ret = self.files[path].clone();
                ret.lock().unwrap().seek(SeekFrom::Start(0))?;
                return Ok(ret);
            }

            use anyhow::Context;
            let ret: Arc<Mutex<dyn Write + Send>> = Arc::new(Mutex::new(
                std::fs::File::create(path)
                    .with_context(|| format!("failed to open {} for writing", path))?,
            ));
            Ok(ret)
        }
    }

    /// Generates unix timestamp for 2020-05-10.
    pub fn make_test_time() -> TestTime {
        TestTime::new(2020, 5, 10)
    }

    /// Time implementation, for test purposes.
    pub struct TestTime {
        now: i64,
        sleep: Arc<Mutex<u64>>,
    }

    impl TestTime {
        pub fn new(year: i32, month: u32, day: u32) -> Self {
            let now = chrono::NaiveDate::from_ymd(year, month, day)
                .and_hms(0, 0, 0)
                .timestamp();
            let sleep = Arc::new(Mutex::new(0_u64));
            TestTime { now, sleep }
        }

        /*/// Gets the duration of the last sleep.
        fn get_sleep(&self) -> u64 {
            *self.sleep.lock().unwrap()
        }*/
    }

    impl Time for TestTime {
        fn now(&self) -> i64 {
            self.now
        }

        fn sleep(&self, seconds: u64) {
            let mut guard = self.sleep.lock().unwrap();
            *guard.deref_mut() = seconds;
        }
    }

    /// Tests Ini.get_tcp_port().
    #[test]
    fn test_ini_get_tcp_port() {
        let ctx = make_test_context().unwrap();
        assert_eq!(ctx.get_ini().get_tcp_port().unwrap(), 8000);
    }
}
