/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Abstractions to help writing unit tests: filesystem, network, etc.

use anyhow::Context as _;
use once_cell::unsync::OnceCell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use crate::sql;

/// File system interface.
pub trait FileSystem {
    /// Test whether a path exists.
    fn path_exists(&self, path: &str) -> bool;

    /// Return the last modification time of a file.
    fn getmtime(&self, path: &str) -> anyhow::Result<time::OffsetDateTime>;

    /// Opens a file for reading in binary mode.
    fn open_read(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Read>>>;

    /// Opens a file for writing in binary mode.
    fn open_write(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Write>>>;

    /// Removes a file.
    fn unlink(&self, path: &str) -> anyhow::Result<()>;

    /// Return a list containing the names of the files in the directory.
    fn listdir(&self, path: &str) -> anyhow::Result<Vec<String>>;

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

pub use system::StdFileSystem;

/// Database interface.
pub trait Database {
    /// Opens the connection.
    fn open(&self) -> anyhow::Result<rusqlite::Connection>;

    /// Opens and initializes a new database connection.
    fn create(&self) -> anyhow::Result<rusqlite::Connection> {
        let mut conn = self.open()?;
        sql::init(&mut conn)?;
        Ok(conn)
    }
}

pub use system::StdDatabase;

/// Network interface.
pub trait Network {
    /// Opens an URL. Empty data means HTTP GET, otherwise it means a HTTP POST.
    fn urlopen(&self, url: &str, data: &str) -> anyhow::Result<String>;
}

pub use system::StdNetwork;

/// Time interface.
pub trait Time {
    /// Calculates the current time.
    fn now(&self) -> time::OffsetDateTime;

    /// The current time as a unix timestamp nanosec string.
    fn now_string(&self) -> String {
        let now = self.now();
        now.unix_timestamp_nanos().to_string()
    }

    /// Delay execution for a given number of seconds.
    fn sleep(&self, seconds: u64);

    /// Allows accessing the implementing struct.
    fn as_any(&self) -> &dyn std::any::Any;
}

pub use system::StdTime;

/// Subprocess interface.
pub trait Subprocess {
    /// Runs a commmand, capturing its output.
    fn run(&self, args: Vec<String>) -> anyhow::Result<String>;

    /// Terminates the current process with the specified exit code.
    fn exit(&self, code: i32);

    /// Allows accessing the implementing struct.
    fn as_any(&self) -> &dyn std::any::Any;
}

pub use system::StdSubprocess;

/// Unit testing interface.
pub trait Unit {
    /// Injects a fake error.
    fn make_error(&self) -> anyhow::Result<()>;
}

pub use system::StdUnit;

/// The root of the workdir/wsgi.ini config file.
#[derive(Default, serde::Deserialize)]
pub struct IniConfig {
    /// The wsgi section in the config file.
    pub wsgi: WsgiConfig,
}

/// The wsgi section in the config file.
#[derive(Default, serde::Deserialize)]
pub struct WsgiConfig {
    /// Space-separated list of housenumber references.
    pub reference_housenumbers: String,
    /// Street reference file path.
    pub reference_street: String,
    /// City counts reference file path.
    pub reference_citycounts: String,
    /// ZIP counts reference file path.
    pub reference_zipcounts: String,
    uri_prefix: Option<String>,
    tcp_port: Option<String>,
    overpass_uri: Option<String>,
    cron_update_inactive: Option<String>,
}

/// Configuration file reader.
pub struct Ini {
    config: IniConfig,
    root: String,
}

impl Ini {
    fn new(
        file_system: &Rc<dyn FileSystem>,
        config_path: &str,
        root: &str,
    ) -> anyhow::Result<Self> {
        let mut config = IniConfig::default();
        if let Ok(data) = file_system.read_to_string(config_path) {
            config = toml::from_str(&data)?;
        }
        Ok(Ini {
            config,
            root: String::from(root),
        })
    }

    /// Gets the directory which is writable.
    pub fn get_workdir(&self) -> String {
        format!("{}/workdir", self.root)
    }

    /// Gets the abs paths of ref housenumbers.
    pub fn get_reference_housenumber_paths(&self) -> anyhow::Result<Vec<String>> {
        let value = &self.config.wsgi.reference_housenumbers;
        let relpaths = value.split(' ');
        Ok(relpaths
            .map(|relpath| format!("{}/{}", self.root, relpath))
            .collect())
    }

    /// Gets the abs path of ref streets.
    pub fn get_reference_street_path(&self) -> anyhow::Result<String> {
        let relpath = &self.config.wsgi.reference_street;
        Ok(format!("{}/{}", self.root, relpath))
    }

    /// Gets the abs path of ref citycounts.
    pub fn get_reference_citycounts_path(&self) -> anyhow::Result<String> {
        let relpath = &self.config.wsgi.reference_citycounts;
        Ok(format!("{}/{}", self.root, relpath))
    }

    /// Gets the abs path of ref zipcounts.
    pub fn get_reference_zipcounts_path(&self) -> anyhow::Result<String> {
        let relpath = &self.config.wsgi.reference_zipcounts;
        Ok(format!("{}/{}", self.root, relpath))
    }

    /// Gets the global URI prefix.
    pub fn get_uri_prefix(&self) -> String {
        self.get_with_fallback(&self.config.wsgi.uri_prefix, "/osm")
    }

    fn get_with_fallback(&self, option: &Option<String>, fallback: &str) -> String {
        match option {
            Some(value) => value.to_string(),
            None => String::from(fallback),
        }
    }

    /// Gets the TCP port to be used.
    pub fn get_tcp_port(&self) -> anyhow::Result<i64> {
        Ok(self
            .get_with_fallback(&self.config.wsgi.tcp_port, "8000")
            .parse::<i64>()?)
    }

    /// Gets the URI of the overpass instance to be used.
    pub fn get_overpass_uri(&self) -> String {
        self.get_with_fallback(&self.config.wsgi.overpass_uri, "https://overpass-api.de")
    }

    /// Should the cron job update inactive relations?
    pub fn get_cron_update_inactive(&self) -> bool {
        let value = self.get_with_fallback(&self.config.wsgi.cron_update_inactive, "False");
        value == "True"
    }
}

/// Context owns global state which is set up once and then read everywhere.
pub struct Context {
    root: String,
    ini: Ini,
    network: Rc<dyn Network>,
    time: Rc<dyn Time>,
    subprocess: Rc<dyn Subprocess>,
    unit: Rc<dyn Unit>,
    file_system: Rc<dyn FileSystem>,
    database: Rc<dyn Database>,
    connection: OnceCell<Rc<RefCell<rusqlite::Connection>>>,
}

impl Context {
    /// Creates a new Context.
    pub fn new(prefix: &str) -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let current_dir_str = current_dir.to_str().context("current_dir() failed")?;
        let root = format!("{current_dir_str}/{prefix}");
        let network = Rc::new(StdNetwork {});
        let time = Rc::new(StdTime {});
        let subprocess = Rc::new(StdSubprocess {});
        let unit = Rc::new(StdUnit {});
        let file_system: Rc<dyn FileSystem> = Rc::new(StdFileSystem {});
        let database: Rc<dyn Database> = Rc::new(StdDatabase {});
        let ini = Ini::new(&file_system, &format!("{root}/workdir/wsgi.ini"), &root)?;
        let connection = OnceCell::new();
        Ok(Context {
            root,
            ini,
            network,
            time,
            subprocess,
            unit,
            file_system,
            database,
            connection,
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
    pub fn get_network(&self) -> &Rc<dyn Network> {
        &self.network
    }

    /// Sets the network implementation.
    pub fn set_network(&mut self, network: Rc<dyn Network>) {
        self.network = network;
    }

    /// Gets the time implementation.
    pub fn get_time(&self) -> &Rc<dyn Time> {
        &self.time
    }

    /// Sets the time implementation.
    pub fn set_time(&mut self, time: &Rc<dyn Time>) {
        self.time = time.clone();
    }

    /// Gets the subprocess implementation.
    pub fn get_subprocess(&self) -> &Rc<dyn Subprocess> {
        &self.subprocess
    }

    /// Sets the subprocess implementation.
    pub fn set_subprocess(&mut self, subprocess: &Rc<dyn Subprocess>) {
        self.subprocess = subprocess.clone();
    }

    /// Gets the testing interface.
    pub fn get_unit(&self) -> &Rc<dyn Unit> {
        &self.unit
    }

    /// Sets the unit implementation.
    pub fn set_unit(&mut self, unit: &Rc<dyn Unit>) {
        self.unit = unit.clone();
    }

    /// Gets the file system implementation.
    pub fn get_file_system(&self) -> &Rc<dyn FileSystem> {
        &self.file_system
    }

    /// Sets the file system implementation.
    pub fn set_file_system(&mut self, file_system: &Rc<dyn FileSystem>) {
        self.file_system = file_system.clone();
    }

    /// Sets the database implementation.
    pub fn set_database(&mut self, database: &Rc<dyn Database>) {
        self.database = database.clone();
    }

    /// Gets the database connection.
    pub fn get_database_connection(&self) -> anyhow::Result<RefMut<'_, rusqlite::Connection>> {
        let connection: &Rc<RefCell<rusqlite::Connection>> = self.connection.get_or_try_init(
            || -> anyhow::Result<Rc<RefCell<rusqlite::Connection>>> {
                Ok(Rc::new(RefCell::new(self.database.create()?)))
            },
        )?;
        Ok(connection.borrow_mut())
    }
}

pub mod system;
#[cfg(test)]
pub mod tests;
