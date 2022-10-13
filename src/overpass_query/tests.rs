/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the overpass_query module.

use super::*;
use std::sync::Arc;

/// Tests overpass_query_need_sleep().
#[test]
fn test_overpass_query_need_sleep() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-happy.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);

    assert_eq!(overpass_query_need_sleep(&ctx), 0);
}

/// Tests overpass_query_need_sleep(): the wait path.
#[test]
fn test_overpass_query_need_sleep_wait() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-wait.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);

    assert_eq!(overpass_query_need_sleep(&ctx), 12);
}

/// Tests overpass_query_need_sleep(): the wait for negative amount path.
#[test]
fn test_overpass_query_need_sleep_wait_negative() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "src/fixtures/network/overpass-status-wait-negative.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);

    assert_eq!(overpass_query_need_sleep(&ctx), 1);
}

/// Tests overpass_query().
#[test]
fn test_overpass_query() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/interpreter",
        /*data_path=*/ "src/fixtures/network/overpass-happy.expected-data",
        /*result_path=*/ "src/fixtures/network/overpass-happy.csv",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let query = ctx
        .get_file_system()
        .read_to_string("src/fixtures/network/overpass-happy.expected-data")
        .unwrap();

    let buf = overpass_query(&ctx, query).unwrap();

    assert_eq!(buf.starts_with("@id"), true);
}
