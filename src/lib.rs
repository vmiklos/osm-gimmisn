/*
 * Copyright 2021 Miklos Vajna
 *
 * SPDX-License-Identifier: MIT
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

//! Finds objects missing from the OSM DB.

mod area_files;
mod areas;
mod cache;
pub mod cache_yamls;
pub mod context;
pub mod cron;
mod i18n;
pub mod missing_housenumbers;
mod overpass_query;
pub mod parse_access_log;
mod ranges;
mod serde;
mod sql;
mod stats;
pub mod sync_ref;
pub mod util;
pub mod validator;
mod webframe;
pub mod wsgi;
mod wsgi_additional;
mod wsgi_json;
mod yattag;
