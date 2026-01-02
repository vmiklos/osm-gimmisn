/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the context module.

use super::*;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops::DerefMut;

/// Creates a Context instance for text purposes.
pub fn make_test_context() -> anyhow::Result<Context> {
    let mut ctx = Context::new("tests")?;

    let file_system = TestFileSystem::new();
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
    ctx.set_file_system(&file_system_rc);
    let time = TestTime::new(2020, 5, 10);
    let time_rc: Rc<dyn Time> = Rc::new(time);
    ctx.set_time(&time_rc);
    let network = TestNetwork::new(&[]);
    let network_rc: Rc<dyn Network> = Rc::new(network);
    ctx.set_network(network_rc);
    let subprocess = TestSubprocess::new(&HashMap::new());
    let subprocess_rc: Rc<dyn Subprocess> = Rc::new(subprocess);
    ctx.set_subprocess(&subprocess_rc);
    let database = TestDatabase {};
    let database_rc: Rc<dyn Database> = Rc::new(database);
    ctx.set_database(&database_rc);

    Ok(ctx)
}

/// Database implementation, for test purposes.
pub struct TestDatabase {}

impl Database for TestDatabase {
    fn open(&self) -> anyhow::Result<rusqlite::Connection> {
        Ok(rusqlite::Connection::open(":memory:")?)
    }
}

/// File system implementation, for test purposes.
pub struct TestFileSystem {
    hide_paths: Rc<RefCell<Vec<String>>>,
    mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>>,
    files: HashMap<String, Rc<RefCell<std::io::Cursor<Vec<u8>>>>>,
}

impl TestFileSystem {
    /// Creates a new TestFileSystem.
    pub fn new() -> Self {
        TestFileSystem {
            hide_paths: Rc::new(RefCell::new(Vec::new())),
            mtimes: HashMap::new(),
            files: HashMap::new(),
        }
    }

    /// Shorthand for new() + set_files() + cast to trait.
    pub fn from_files(
        files: &HashMap<String, Rc<RefCell<std::io::Cursor<Vec<u8>>>>>,
    ) -> Rc<dyn FileSystem> {
        let mut file_system = TestFileSystem::new();
        file_system.set_files(files);
        let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);
        file_system_rc
    }

    /// Creates a new test file, not yet inserting it to the filesystem.
    pub fn make_file() -> Rc<RefCell<std::io::Cursor<Vec<u8>>>> {
        Rc::new(RefCell::new(std::io::Cursor::new(Vec::new())))
    }

    /// Writes a json value to a test file & return that file.
    pub fn write_json_to_file(json: &serde_json::Value) -> Rc<RefCell<std::io::Cursor<Vec<u8>>>> {
        let file = TestFileSystem::make_file();
        {
            let mut guard = file.borrow_mut();
            let write = guard.deref_mut();
            serde_json::to_writer(write, json).unwrap();
        }
        file
    }

    /// Creates new test files, specifying their content via path-value pairs.
    pub fn make_files(
        ctx: &Context,
        files: &[(&str, &Rc<RefCell<Cursor<Vec<u8>>>>)],
    ) -> HashMap<String, Rc<RefCell<std::io::Cursor<Vec<u8>>>>> {
        let mut ret = HashMap::new();
        for file in files {
            let (path, content) = file;
            ret.insert(ctx.get_abspath(path), (*content).clone());
        }
        ret
    }

    /// Sets the hide paths.
    pub fn set_hide_paths(&mut self, hide_paths: &[String]) {
        self.hide_paths = Rc::new(RefCell::new(hide_paths.to_vec()));
    }

    /// Sets the mtimes.
    pub fn set_mtimes(&mut self, mtimes: &HashMap<String, Rc<RefCell<time::OffsetDateTime>>>) {
        self.mtimes = mtimes.clone();
    }

    /// Sets the files.
    pub fn set_files(&mut self, files: &HashMap<String, Rc<RefCell<std::io::Cursor<Vec<u8>>>>>) {
        self.files = files.clone()
    }
}

impl FileSystem for TestFileSystem {
    fn path_exists(&self, path: &str) -> bool {
        if self.hide_paths.borrow().contains(&path.to_string()) {
            return false;
        }

        if self.files.contains_key(path) {
            return true;
        }

        Path::new(path).exists()
    }

    fn getmtime(&self, path: &str) -> anyhow::Result<time::OffsetDateTime> {
        if let Some(value) = self.mtimes.get(path) {
            return Ok(*value.borrow());
        }

        let metadata =
            std::fs::metadata(path).context(format!("metadata() failed for '{path}'"))?;
        Ok(time::OffsetDateTime::try_from(metadata.modified()?)?)
    }

    fn open_read(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        if self.hide_paths.borrow().contains(&path.to_string()) {
            return Err(anyhow::anyhow!("'{}' is hidden", path));
        }

        if self.files.contains_key(path) {
            let ret = self.files[path].clone();
            ret.borrow_mut().seek(SeekFrom::Start(0))?;
            return Ok(ret);
        }
        let ret: Rc<RefCell<dyn Read>> = Rc::new(RefCell::new(
            std::fs::File::open(path).context(format!("failed to open '{path}'"))?,
        ));
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        if !self.files.contains_key(path) {
            return Err(anyhow::anyhow!(
                "open_write: self.files doesn't contain '{}'",
                path
            ));
        }

        let mut hide_paths = self.hide_paths.borrow_mut();
        if hide_paths.contains(&path.to_string()) {
            let position = hide_paths.iter().position(|i| *i == path).unwrap();
            hide_paths.remove(position);
        }

        assert!(self.mtimes.get(path).is_none());

        let ret = self.files[path].clone();
        ret.borrow_mut().seek(SeekFrom::Start(0))?;
        Ok(ret)
    }

    fn unlink(&self, path: &str) -> anyhow::Result<()> {
        let mut hide_paths = self.hide_paths.borrow_mut();
        if !self.files.contains_key(path) || hide_paths.contains(&path.to_string()) {
            return Err(anyhow::anyhow!("unlink: {}: no such file", path));
        }

        hide_paths.push(path.to_string());
        Ok(())
    }

    fn listdir(&self, path: &str) -> anyhow::Result<Vec<String>> {
        let mut contents: Vec<String> = Vec::new();
        for file in self.files.iter() {
            if self.hide_paths.borrow().contains(file.0) {
                continue;
            }

            if !file.0.starts_with(path) {
                continue;
            }

            contents.push(file.0.clone());
        }

        Ok(contents)
    }
}

/// Time implementation, for test purposes.
pub struct TestTime {
    now: time::OffsetDateTime,
    sleep: Rc<RefCell<u64>>,
}

impl TestTime {
    /// Creates a new TestTime.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        let date = time::Date::from_calendar_date(
            year,
            time::Month::try_from(month as u8).unwrap(),
            day as u8,
        )
        .unwrap()
        .midnight();
        let now = date.assume_utc();
        let sleep = Rc::new(RefCell::new(0_u64));
        TestTime { now, sleep }
    }

    /// Gets the duration of the last sleep.
    pub fn get_sleep(&self) -> u64 {
        *self.sleep.borrow_mut()
    }
}

impl Time for TestTime {
    fn now(&self) -> time::OffsetDateTime {
        self.now
    }

    fn sleep(&self, seconds: u64) {
        let mut guard = self.sleep.borrow_mut();
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
    /// Creates a new URLRoute.
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
    routes: Rc<RefCell<Vec<URLRoute>>>,
}

impl TestNetwork {
    /// Creates a new TestNetwork.
    pub fn new(routes: &[URLRoute]) -> Self {
        let routes = Rc::new(RefCell::new(routes.to_vec()));
        TestNetwork { routes }
    }
}

impl Network for TestNetwork {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String> {
        let mut ret: String = "".into();
        let mut remove: Option<usize> = None;
        let mut locked_routes = self.routes.borrow_mut();
        for (index, route) in locked_routes.iter().enumerate() {
            if url != route.url {
                continue;
            }

            if !route.data_path.is_empty() {
                let expected = std::fs::read_to_string(&route.data_path)?;
                assert_eq!(data, expected);
            }

            if route.result_path.is_empty() {
                locked_routes.remove(index);
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
    /// Creates a new TestUnit.
    pub fn new() -> Self {
        TestUnit {}
    }
}

impl Unit for TestUnit {
    fn make_error(&self) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("TestError"))
    }
}

/// Subprocess implementation for test purposes.
pub struct TestSubprocess {
    outputs: HashMap<String, String>,
    runs: Rc<RefCell<Vec<String>>>,
}

impl TestSubprocess {
    /// Creates a new TestSubprocess.
    pub fn new(outputs: &HashMap<String, String>) -> Self {
        let outputs = outputs.clone();
        let runs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        TestSubprocess { outputs, runs }
    }

    /// Gets a list of invoked commands.
    pub fn get_runs(&self) -> Vec<String> {
        self.runs.borrow_mut().clone()
    }
}

impl Subprocess for TestSubprocess {
    fn run(&self, args: Vec<String>) -> anyhow::Result<String> {
        let key = args.join(" ");
        self.runs.borrow_mut().push(key.clone());
        Ok(self.outputs[&key].clone())
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

/// Tests Ini.get_with_fallack().
#[test]
fn test_ini_get_with_fallback() {
    let ctx = make_test_context().unwrap();
    assert_eq!(
        ctx.get_ini()
            .get_with_fallback(&Some("/osm".to_string()), "myfallback"),
        "/osm"
    );
    assert_eq!(
        ctx.get_ini().get_with_fallback(&None, "myfallback"),
        "myfallback"
    );
}

/// Tests Ini.new().
#[test]
fn test_ini_new() {
    let ctx = make_test_context().unwrap();
    let wsgi_ini = TestFileSystem::make_file();
    let files = TestFileSystem::make_files(&ctx, &[("workdir/wsgi.ini", &wsgi_ini)]);
    let mut file_system = TestFileSystem::new();
    file_system.set_files(&files);
    file_system
        .write_from_string("[wsgi]\n=", &ctx.get_abspath("workdir/wsgi.ini"))
        .unwrap();
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);

    let ret = Ini::new(
        &file_system_rc,
        &ctx.get_abspath("workdir/wsgi.ini"),
        "tests",
    );

    assert_eq!(ret.is_err(), true);
}

/// Tests Ini.new() with a non-existing workdir/wsgi.ini.
#[test]
fn test_ini_new_no_config() {
    let ctx = make_test_context().unwrap();
    let mut file_system = TestFileSystem::new();
    file_system.set_hide_paths(&[ctx.get_abspath("workdir/wsgi.ini")]);
    let file_system_rc: Rc<dyn FileSystem> = Rc::new(file_system);

    let ret = Ini::new(&file_system_rc, &ctx.get_abspath("workdir/wsgi.ini"), "");

    assert_eq!(ret.is_err(), false);
}

/// Tests TestFileSystem::unlink().
#[test]
fn test_file_system_unlink() {
    let ctx = make_test_context().unwrap();

    let ret = ctx.get_file_system().unlink("no such file");

    assert_eq!(ret.is_err(), true);
}

/// Tests TestFileSystem::open_write().
#[test]
fn test_file_system_open_write() {
    let ctx = make_test_context().unwrap();

    let ret = ctx.get_file_system().open_write("no such file");

    assert_eq!(ret.is_err(), true);
}

/// Tests TestFileSystem::list_dir().
#[test]
fn test_file_system_list_dir() {
    let ctx = make_test_context().unwrap();
    let mut file_system = TestFileSystem::new();
    file_system.set_hide_paths(&[ctx.get_abspath("hidden file")]);
    let hidden_file = TestFileSystem::make_file();
    let visible_file = TestFileSystem::make_file();
    let files = TestFileSystem::make_files(
        &ctx,
        &[
            ("hidden file", &hidden_file),
            ("visible file", &visible_file),
        ],
    );
    file_system.set_files(&files);

    let ret = file_system.listdir("dir").unwrap();

    assert_eq!(ret.is_empty(), true);
}
