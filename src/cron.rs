/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cron module allows doing nightly tasks.

use crate::context;
use anyhow::Context;
use pyo3::prelude::*;

/// Sets up logging.
fn setup_logging(ctx: &context::Context) -> anyhow::Result<()> {
    let config = simplelog::ConfigBuilder::new()
        .set_time_format("%Y-%m-%d %H:%M:%S".into())
        .set_time_to_local(true)
        .build();
    let logpath = ctx.get_abspath("workdir/cron.log")?;
    let file = std::fs::File::create(logpath)?;
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            simplelog::LevelFilter::Info,
            config.clone(),
            simplelog::TerminalMode::Stdout,
            simplelog::ColorChoice::Never,
        ),
        simplelog::WriteLogger::new(simplelog::LevelFilter::Info, config, file),
    ])?;

    Ok(())
}

#[pyfunction]
fn py_setup_logging(ctx: context::PyContext) -> PyResult<()> {
    match setup_logging(&ctx.context).context("setup_logging() failed") {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!("{:?}", err))),
    }
}

#[pyfunction]
fn py_info(msg: String) {
    log::info!("{}", msg);
}

#[pyfunction]
fn py_error(msg: String) {
    log::error!("{}", msg);
}

pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(py_setup_logging, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_info, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(py_error, module)?)?;
    Ok(())
}
