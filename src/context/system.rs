/*
 * Copyright 2022 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Trait implementations using the real file system, network, time, etc.

use super::*;
use isahc::ReadResponseExt as _;
use isahc::RequestExt as _;
use isahc::config::Configurable as _;
use lazy_static::lazy_static;

lazy_static! {
    static ref TZ_OFFSET: time::UtcOffset = time::UtcOffset::current_local_offset().unwrap();
}

/// Gets the current timezone offset. Has to be first called when there are no threads yet.
pub fn get_tz_offset() -> time::UtcOffset {
    *TZ_OFFSET
}

/// Database implementation, backed by sqlite on the filesystem.
pub struct StdDatabase {}

impl Database for StdDatabase {
    fn open(&self) -> anyhow::Result<rusqlite::Connection> {
        // https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html
        Ok(rusqlite::Connection::open("workdir/state.db")?)
    }
}

/// File system implementation, backed by the Rust stdlib.
pub struct StdFileSystem {}

// Real file-system is intentionally mocked.
impl FileSystem for StdFileSystem {
    fn path_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn getmtime(&self, path: &str) -> anyhow::Result<time::OffsetDateTime> {
        let metadata = std::fs::metadata(path)?;
        let modified = time::OffsetDateTime::from(metadata.modified()?);
        Ok(modified.to_offset(get_tz_offset()))
    }

    fn open_read(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Read>>> {
        let ret: Rc<RefCell<dyn Read>> = Rc::new(RefCell::new(
            std::fs::File::open(path)
                .with_context(|| format!("failed to open {path} for reading"))?,
        ));
        Ok(ret)
    }

    fn open_write(&self, path: &str) -> anyhow::Result<Rc<RefCell<dyn Write>>> {
        // Create containing directory if needed.
        let path_obj = std::path::Path::new(path);
        let dir_obj = path_obj.parent().context("failed to get parent dir")?;
        let dir = dir_obj.to_str().context("failed to get dir as string")?;
        std::fs::create_dir_all(dir)?;

        let ret: Rc<RefCell<dyn Write>> = Rc::new(RefCell::new(
            std::fs::File::create(path)
                .with_context(|| format!("failed to open {path} for writing"))?,
        ));
        Ok(ret)
    }

    fn unlink(&self, path: &str) -> anyhow::Result<()> {
        Ok(std::fs::remove_file(path)?)
    }

    fn listdir(&self, path: &str) -> anyhow::Result<Vec<String>> {
        let mut contents: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.into_os_string().into_string().unwrap();
            contents.push(file_name);
        }
        Ok(contents)
    }
}

/// Network implementation, backed by a real HTTP library.
pub struct StdNetwork {}

// Real network is intentionally mocked.
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

/// Time implementation, backed by the the actual time.
pub struct StdTime {}

// Real time is intentionally mocked.
impl Time for StdTime {
    fn now(&self) -> time::OffsetDateTime {
        let now = time::OffsetDateTime::now_utc();
        now.to_offset(get_tz_offset())
    }

    fn sleep(&self, seconds: u64) {
        std::thread::sleep(std::time::Duration::from_secs(seconds));
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Subprocess implementation, backed by the Rust stdlib.
pub struct StdSubprocess {}

// Real processes are intentionally mocked.
impl Subprocess for StdSubprocess {
    fn run(&self, args: Vec<String>) -> anyhow::Result<String> {
        let (first, rest) = args
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("args is an empty list"))?;
        let output = std::process::Command::new(first).args(rest).output()?;
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Unit implementation, which intentionally does nothing.
pub struct StdUnit {}

impl Unit for StdUnit {
    fn make_error(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
