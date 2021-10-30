/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Compares reference house numbers with OSM ones and shows the diff.

use crate::areas;
use crate::context;
use crate::util;
use pyo3::prelude::*;
use std::io::Write;

/// Commandline interface.
pub fn main(argv: &[String], stream: &mut dyn Write, ctx: &context::Context) -> anyhow::Result<()> {
    let relation_name = argv[1].clone();

    let mut relations = areas::Relations::new(ctx)?;
    let mut relation = relations.get_relation(&relation_name)?;
    let (ongoing_streets, _done_streets) = relation.get_missing_housenumbers()?;

    for result in ongoing_streets {
        // House number, # of only_in_reference items.
        let range_list = util::get_housenumber_ranges(&result.1);
        let mut range_strings: Vec<&String> = range_list.iter().map(|i| i.get_number()).collect();
        range_strings.sort_by_key(|i| util::split_house_number(i));
        stream.write_all(
            format!("{}\t{}\n", result.0.get_osm_name(), range_strings.len()).as_bytes(),
        )?;
        // only_in_reference items.
        stream.write_all(format!("{:?}\n", range_strings).as_bytes())?;
    }

    Ok(())
}

/// Commandline interface.
#[pyfunction]
pub fn py_missing_housenumbers_main(
    argv: Vec<String>,
    stdout: PyObject,
    ctx: &context::PyContext,
) -> PyResult<()> {
    let mut stream = context::PyAnyWrite { write: stdout };
    match main(&argv, &mut stream, &ctx.context) {
        Ok(value) => Ok(value),
        Err(err) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "main() failed: {}",
            err.to_string()
        ))),
    }
}

/// Registers Python wrappers of Rust structs into the Python module.
pub fn register_python_symbols(module: &PyModule) -> PyResult<()> {
    module.add_function(pyo3::wrap_pyfunction!(
        py_missing_housenumbers_main,
        module
    )?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::io::Seek;
    use std::io::SeekFrom;

    /// Tests main().
    #[test]
    fn test_main() {
        let argv = vec!["".to_string(), "gh195".to_string()];
        let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        let mut ctx = context::tests::make_test_context().unwrap();

        main(&argv, &mut buf, &mut ctx).unwrap();

        buf.seek(SeekFrom::Start(0)).unwrap();
        let mut actual: Vec<u8> = Vec::new();
        buf.read_to_end(&mut actual).unwrap();
        assert_eq!(
            actual,
            b"Kalotaszeg utca\t3\n[\"25\", \"27-37\", \"31*\"]\n"
        );
    }
}
