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
use anyhow::Context as AnyhowContext;
use isahc::config::Configurable;
use isahc::ReadResponseExt;
use isahc::RequestExt;
use std::cell::RefCell;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

/// File system interface.
pub trait FileSystem {
    /// Test whether a path exists.
    fn path_exists(&self, path: &str) -> bool;

    /// Return the last modification time of a file.
    fn getmtime(&self, path: &str) -> anyhow::Result<f64>;

    /// Opens a file for reading in binary mode.
    fn open_read(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Read>>>;

    /// Opens a file for writing in binary mode.
    fn open_write(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Write>>>;

    /// Read the entire contents of a file into a string.
    fn read_to_string(&self, path: &str) -> anyhow::Result<String> {
        let stream = self.open_read(path)?;
        let mut guard = stream.borrow_mut();
        let mut bytes: Vec<u8> = Vec::new();
        guard.read_to_end(&mut bytes).unwrap();
        Ok(String::from_utf8(bytes)?)
    }

    /// Write the entire string to a file.
    fn write_from_string(&self, string: &str, path: &str) -> anyhow::Result<()> {
        let stream = self.open_write(path)?;
        let mut guard = stream.borrow_mut();
        Ok(guard.write_all(string.as_bytes())?)
    }
}

/// File system implementation, backed by the Rust stdlib.
struct StdFileSystem {}

// Real file-system is intentionally mocked.
#[cfg(not(tarpaulin_include))]
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

    fn open_read(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let ret: Rc<RefCell<dyn Read>> = Rc::new(RefCell::new(
            std::fs::File::open(path)
                .with_context(|| format!("failed to open {} for reading", path))?,
        ));
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        let ret: Rc<RefCell<dyn Write>> = Rc::new(RefCell::new(
            std::fs::File::create(path)
                .with_context(|| format!("failed to open {} for writing", path))?,
        ));
        Ok(ret)
    }
}

/// Network interface.
pub trait Network {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String>;
}

/// Network implementation, backed by a real HTTP library.
struct StdNetwork {}

// Real network is intentionally mocked.
#[cfg(not(tarpaulin_include))]
impl Network for StdNetwork {
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String> {
        if !data.is_empty() {
            let mut buf = isahc::Request::post(url)
                .redirect_policy(isahc::config::RedirectPolicy::Limit(1))
                .timeout(Duration::from_secs(425))
                .body(data)?
                .send()?;
            let ret = buf.text()?;
            return Ok(ret);
        }

        let mut buf = isahc::Request::get(url)
            .redirect_policy(isahc::config::RedirectPolicy::Limit(1))
            .timeout(Duration::from_secs(425))
            .body(())?
            .send()?;
        let ret = buf.text()?;
        Ok(ret)
    }
}

/// Time interface.
pub trait Time {
    /// Calculates the current Unix timestamp from GMT.
    fn now(&self) -> i64;

    /// Delay execution for a given number of seconds.
    fn sleep(&self, seconds: u64);

    /// Allows accessing the implementing struct.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Time implementation, backed by the chrono.
struct StdTime {}

// Real time is intentionally mocked.
#[cfg(not(tarpaulin_include))]
impl Time for StdTime {
    fn now(&self) -> i64 {
        let now = chrono::Local::now();
        now.naive_local().timestamp()
    }

    fn sleep(&self, seconds: u64) {
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Subprocess interface.
pub trait Subprocess {
    /// Runs a commmand, capturing its output.
    fn run(&self, args: Vec<String>) -> anyhow::Result<String>;

    /// Terminates the current process with the specified exit code.
    fn exit(&self, code: i32);

    /// Allows accessing the implementing struct.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Subprocess implementation, backed by the Rust stdlib.
struct StdSubprocess {}

// Real processes are intentionally mocked.
#[cfg(not(tarpaulin_include))]
impl Subprocess for StdSubprocess {
    fn run(&self, args: Vec<String>) -> anyhow::Result<String> {
        let (first, rest) = args
            .split_first()
            .ok_or_else(|| anyhow!("args is an empty list"))?;
        let output = std::process::Command::new(first).args(rest).output()?;
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    }

    fn exit(&self, code: i32) {
        std::process::exit(code);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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

/// Configuration file reader.
#[derive(Clone)]
pub struct Ini {
    config: configparser::ini::Ini,
    root: String,
}

impl Ini {
    fn new(config_path: &str, root: &str) -> anyhow::Result<Self> {
        let mut config = configparser::ini::Ini::new();
        // TODO error handling?
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
            .context("no wsgi.workdir in config")?;
        Ok(format!("{}/{}", self.root, workdir))
    }

    /// Gets the abs paths of ref housenumbers.
    pub fn get_reference_housenumber_paths(&self) -> anyhow::Result<Vec<String>> {
        let value = self
            .config
            .get("wsgi", "reference_housenumbers")
            .context("no wsgi.reference_housenumbers in config")?;
        let relpaths = value.split(' ');
        Ok(relpaths
            .map(|relpath| format!("{}/{}", self.root, relpath))
            .collect())
    }

    /// Gets the abs path of ref streets.
    pub fn get_reference_street_path(&self) -> anyhow::Result<String> {
        let relpath = self
            .config
            .get("wsgi", "reference_street")
            .context("no wsgi.reference_street in config")?;
        Ok(format!("{}/{}", self.root, relpath))
    }

    /// Gets the abs path of ref citycounts.
    pub fn get_reference_citycounts_path(&self) -> anyhow::Result<String> {
        let relpath = self
            .config
            .get("wsgi", "reference_citycounts")
            .context("no wsgi.reference_citycounts in config")?;
        Ok(format!("{}/{}", self.root, relpath))
    }

    /// Gets the abs path of ref zipcounts.
    pub fn get_reference_zipcounts_path(&self) -> anyhow::Result<String> {
        let relpath = self
            .config
            .get("wsgi", "reference_zipcounts")
            .context("no wsgi.reference_zipcounts in config")?;
        Ok(format!("{}/{}", self.root, relpath))
    }

    /// Gets the global URI prefix.
    pub fn get_uri_prefix(&self) -> anyhow::Result<String> {
        self.config
            .get("wsgi", "uri_prefix")
            .context("no wsgi.uri_prefix in config")
    }

    fn get_with_fallback(&self, key: &str, fallback: &str) -> String {
        match self.config.get("wsgi", key) {
            Some(value) => value,
            None => String::from(fallback),
        }
    }

    /// Gets the TCP port to be used.
    pub fn get_tcp_port(&self) -> anyhow::Result<i64> {
        Ok(self.get_with_fallback("tcp_port", "8000").parse::<i64>()?)
    }

    /// Gets the URI of the overpass instance to be used.
    pub fn get_overpass_uri(&self) -> String {
        self.get_with_fallback("overpass_uri", "https://overpass-api.de")
    }

    /// Should the cron job update inactive relations?
    pub fn get_cron_update_inactive(&self) -> bool {
        let value = self.get_with_fallback("cron_update_inactive", "False");
        value == "True"
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
        let root = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), prefix);
        let ini = Ini::new(&format!("{}/wsgi.ini", root), &root)?;
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
    pub fn get_abspath(&self, rel_path: &str) -> String {
        format!("{}/{}", self.root, rel_path)
    }

    /// Gets the ini file.
    pub fn get_ini(&self) -> &Ini {
        &self.ini
    }

    /// Gets the network implementation.
    pub fn get_network(&self) -> &Arc<dyn Network> {
        &self.network
    }

    /// Sets the network implementation.
    pub fn set_network(&mut self, network: &Arc<dyn Network>) {
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

    /// Sets the subprocess implementation.
    pub fn set_subprocess(&mut self, subprocess: &Arc<dyn Subprocess>) {
        self.subprocess = subprocess.clone();
    }

    /// Gets the testing interface.
    pub fn get_unit(&self) -> &Arc<dyn Unit> {
        &self.unit
    }

    /// Sets the unit implementation.
    pub fn set_unit(&mut self, unit: &Arc<dyn Unit>) {
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

#[cfg(test)]
pub mod tests;
