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
use std::io::Read;
use std::io::Write;
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
        let ret: Arc<Mutex<dyn Read + Send>> = Arc::new(Mutex::new(
            std::fs::File::open(path)
                .with_context(|| format!("failed to open {} for reading", path))?,
        ));
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Arc<Mutex<dyn Write + Send>>> {
        let ret: Arc<Mutex<dyn Write + Send>> = Arc::new(Mutex::new(
            std::fs::File::create(path)
                .with_context(|| format!("failed to open {} for writing", path))?,
        ));
        Ok(ret)
    }
}

/// Network interface.
pub trait Network: Send + Sync {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String>;
}

/// Network implementation, backed by a real HTTP library.
struct StdNetwork {}

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
pub trait Time: Send + Sync {
    /// Calculates the current Unix timestamp from GMT.
    fn now(&self) -> i64;

    /// Delay execution for a given number of seconds.
    fn sleep(&self, seconds: u64);

    /// Allows accessing the implementing struct.
    fn as_any(&self) -> &dyn std::any::Any;
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Subprocess interface.
pub trait Subprocess: Send + Sync {
    /// Runs a commmand, capturing its output.
    fn run(&self, args: Vec<String>) -> anyhow::Result<String>;

    /// Terminates the current process with the specified exit code.
    fn exit(&self, code: i32);

    /// Allows accessing the implementing struct.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Subprocess implementation, backed by the Rust stdlib.
struct StdSubprocess {}

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

    /// Gets the abs path of ref zipcounts.
    pub fn get_reference_zipcounts_path(&self) -> anyhow::Result<String> {
        let relpath = self
            .config
            .get("wsgi", "reference_zipcounts")
            .context("cannot get key reference_zipcounts")?;
        Ok(format!("{}/{}", self.root, relpath))
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
    pub fn get_cron_update_inactive(&self) -> bool {
        match self.config.get("wsgi", "cron_update_inactive") {
            Some(value) => value == "True",
            None => false,
        }
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
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Cursor;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::ops::DerefMut;

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

        pub fn make_file() -> Arc<Mutex<std::io::Cursor<Vec<u8>>>> {
            Arc::new(Mutex::new(std::io::Cursor::new(Vec::new())))
        }

        pub fn make_files(
            ctx: &Context,
            files: &[(&str, &Arc<Mutex<Cursor<Vec<u8>>>>)],
        ) -> HashMap<String, Arc<Mutex<std::io::Cursor<Vec<u8>>>>> {
            let mut ret = HashMap::new();
            for file in files {
                let (path, content) = file;
                ret.insert(ctx.get_abspath(path).unwrap(), (*content).clone());
            }
            ret
        }

        /// Sets the hide paths.
        pub fn set_hide_paths(&mut self, hide_paths: &[String]) {
            self.hide_paths = hide_paths.to_vec();
        }

        /// Sets the mtimes.
        pub fn set_mtimes(&mut self, mtimes: &HashMap<String, f64>) {
            self.mtimes = mtimes.clone();
        }

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

        /// Gets the duration of the last sleep.
        pub fn get_sleep(&self) -> u64 {
            *self.sleep.lock().unwrap()
        }
    }

    impl Time for TestTime {
        fn now(&self) -> i64 {
            self.now
        }

        fn sleep(&self, seconds: u64) {
            let mut guard = self.sleep.lock().unwrap();
            *guard.deref_mut() = seconds;
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    /// Contains info about how to patch out one URL.
    #[derive(Clone)]
    pub struct URLRoute {
        /// The request URL
        url: String,
        /// Path of expected POST data, empty for GET
        data_path: String,
        /// Path of expected result data
        result_path: String,
    }

    impl URLRoute {
        pub fn new(url: &str, data_path: &str, result_path: &str) -> Self {
            URLRoute {
                url: url.into(),
                data_path: data_path.into(),
                result_path: result_path.into(),
            }
        }
    }

    /// Network implementation, for test purposes.
    pub struct TestNetwork {
        routes: Arc<Mutex<Vec<URLRoute>>>,
    }

    impl TestNetwork {
        pub fn new(routes: &[URLRoute]) -> Self {
            let routes = Arc::new(Mutex::new(routes.to_vec()));
            TestNetwork { routes }
        }
    }

    impl Network for TestNetwork {
        /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
        fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String> {
            let mut ret: String = "".into();
            let mut remove: Option<usize> = None;
            let mut locked_routes = self.routes.lock().unwrap();
            for (index, route) in locked_routes.iter().enumerate() {
                if url != route.url {
                    continue;
                }

                if !route.data_path.is_empty() {
                    let expected = std::fs::read_to_string(&route.data_path)?;
                    assert_eq!(data, expected);
                }

                if route.result_path.is_empty() {
                    return Err(anyhow::anyhow!("empty result_path for url '{}'", url));
                }
                ret = std::fs::read_to_string(&route.result_path)?;
                remove = Some(index);
                break;
            }

            if ret.is_empty() {
                return Err(anyhow::anyhow!("url missing from route list: '{}'", url));
            }
            // Allow specifying multiple results for the same URL.
            locked_routes.remove(remove.unwrap());
            Ok(ret)
        }
    }

    /// Unit implementation, which intentionally fails.
    pub struct TestUnit {}

    impl TestUnit {
        pub fn new() -> Self {
            TestUnit {}
        }
    }

    impl Unit for TestUnit {
        fn make_error(&self) -> String {
            return "TestError".into();
        }
    }

    /// Subprocess implementation for test purposes.
    pub struct TestSubprocess {
        outputs: HashMap<String, String>,
        runs: Arc<Mutex<Vec<String>>>,
        exits: Arc<Mutex<Vec<i32>>>,
    }

    impl TestSubprocess {
        pub fn new(outputs: &HashMap<String, String>) -> Self {
            let outputs = outputs.clone();
            let runs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let exits: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(Vec::new()));
            TestSubprocess {
                outputs,
                runs,
                exits,
            }
        }

        /// Gets a list of invoked commands.
        pub fn get_runs(&self) -> Vec<String> {
            self.runs.lock().unwrap().clone()
        }

        /// Gets a list of exit codes.
        pub fn get_exits(&self) -> Vec<i32> {
            self.exits.lock().unwrap().clone()
        }
    }

    impl Subprocess for TestSubprocess {
        fn run(&self, args: Vec<String>) -> anyhow::Result<String> {
            let key = args.join(" ");
            self.runs.lock().unwrap().push(key.clone());
            Ok(self.outputs[&key].clone())
        }

        fn exit(&self, code: i32) {
            self.exits.lock().unwrap().push(code);
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    /// Tests Ini.get_tcp_port().
    #[test]
    fn test_ini_get_tcp_port() {
        let ctx = make_test_context().unwrap();
        assert_eq!(ctx.get_ini().get_tcp_port().unwrap(), 8000);
    }
}
