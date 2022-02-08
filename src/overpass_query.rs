/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! The overpass_query module allows getting data out of the OSM DB without a full download.

use crate::context;

/// Posts the query string to the overpass API and returns the result string.
pub fn overpass_query(ctx: &context::Context, query: String) -> anyhow::Result<String> {
    let url = ctx.get_ini().get_overpass_uri() + "/api/interpreter";

    ctx.get_network().urlopen(&url, &query)
}

/// Checks if we need to sleep before executing an overpass query.
pub fn overpass_query_need_sleep(ctx: &context::Context) -> i32 {
    let url = ctx.get_ini().get_overpass_uri() + "/api/status";
    let status = match ctx.get_network().urlopen(&url, "") {
        Ok(value) => value,
        _ => {
            return 0;
        }
    };
    let mut sleep = 0;
    let mut available = false;
    for line in status.lines() {
        if line.starts_with("Slot available after:") {
            let re = regex::Regex::new(r".*in (-?\d+) seconds.*").unwrap();
            for cap in re.captures_iter(line) {
                // This should neve fail since the regex only allows numbers.
                sleep = cap[1].parse::<i32>().expect("parse() to i32 failed");
                // Wait one more second just to be safe.
                sleep += 1;
                if sleep <= 0 {
                    sleep = 1;
                }
            }
            break;
        }
        if line.contains("available now") {
            available = true;
        }
    }
    if available {
        return 0;
    }
    sleep
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Tests overpass_query_need_sleep().
    #[test]
    fn test_overpass_query_need_sleep() {
        let mut ctx = context::tests::make_test_context().unwrap();
        let routes = vec![context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
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
            /*result_path=*/ "tests/network/overpass-status-wait.txt",
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
            /*result_path=*/ "tests/network/overpass-status-wait-negative.txt",
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
            /*data_path=*/ "tests/network/overpass-happy.expected-data",
            /*result_path=*/ "tests/network/overpass-happy.csv",
        )];
        let network = context::tests::TestNetwork::new(&routes);
        let network_arc: Arc<dyn context::Network> = Arc::new(network);
        ctx.set_network(&network_arc);
        let query = ctx
            .get_file_system()
            .read_to_string("tests/network/overpass-happy.expected-data")
            .unwrap();

        let buf = overpass_query(&ctx, query).unwrap();

        assert_eq!(buf.starts_with("@id"), true);
    }
}
