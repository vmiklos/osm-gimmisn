/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The cache module accelerates some functions of the areas module.

use crate::areas;
use crate::context;

/// Decides if we have an up to date cache entry or not.
fn is_cache_current(
    ctx: &context::Context,
    cache_path: &str,
    dependencies: &[String],
) -> anyhow::Result<bool> {
    if !ctx.get_file_system().path_exists(cache_path) {
        return Ok(false);
    }

    let cache_mtime = ctx.get_file_system().getmtime(cache_path)?;

    for dependency in dependencies {
        if ctx.get_file_system().path_exists(dependency)
            && ctx.get_file_system().getmtime(dependency)? > cache_mtime
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Decides if we have an up to date json cache entry or not.
fn is_missing_housenumbers_json_cached(relation: &mut areas::Relation) -> anyhow::Result<bool> {
    let cache_path = relation.get_files().get_housenumbers_jsoncache_path();
    let datadir = relation.get_ctx().get_abspath("data");
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path,
    ];
    is_cache_current(relation.get_ctx(), &cache_path, &dependencies)
}

/// Gets the cached json of the missing housenumbers for a relation.
pub fn get_missing_housenumbers_json(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<String> {
    let output: String;
    if is_missing_housenumbers_json_cached(relation)? {
        let files = relation.get_files();
        output = ctx
            .get_file_system()
            .read_to_string(&files.get_housenumbers_jsoncache_path())?;
        return Ok(output);
    }

    let missing_housenumbers = relation.get_missing_housenumbers()?;
    output = serde_json::to_string(&missing_housenumbers)?;

    let files = relation.get_files();
    ctx.get_file_system()
        .write_from_string(&output, &files.get_housenumbers_jsoncache_path())?;
    Ok(output)
}

/// Decides if we have an up to date additional json cache entry or not.
fn is_additional_housenumbers_json_cached(
    ctx: &context::Context,
    relation: &areas::Relation,
) -> anyhow::Result<bool> {
    let cache_path = relation
        .get_files()
        .get_additional_housenumbers_jsoncache_path();
    let datadir = ctx.get_abspath("data");
    let relation_path = format!("{}/relation-{}.yaml", datadir, relation.get_name());
    let dependencies = vec![
        relation.get_files().get_osm_streets_path(),
        relation.get_files().get_osm_housenumbers_path(),
        relation.get_files().get_ref_housenumbers_path(),
        relation_path,
    ];
    is_cache_current(ctx, &cache_path, &dependencies)
}

/// Gets the cached json of the additional housenumbers for a relation.
pub fn get_additional_housenumbers_json(
    ctx: &context::Context,
    relation: &mut areas::Relation,
) -> anyhow::Result<String> {
    let output: String;
    if is_additional_housenumbers_json_cached(ctx, relation)? {
        let files = relation.get_files();
        output = ctx
            .get_file_system()
            .read_to_string(&files.get_additional_housenumbers_jsoncache_path())?;
        return Ok(output);
    }

    let additional_housenumbers = relation.get_additional_housenumbers()?;
    output = serde_json::to_string(&additional_housenumbers)?;

    let files = relation.get_files();
    ctx.get_file_system()
        .write_from_string(&output, &files.get_additional_housenumbers_jsoncache_path())?;
    Ok(output)
}

#[cfg(test)]
mod tests;
