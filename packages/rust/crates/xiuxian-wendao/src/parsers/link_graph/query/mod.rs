//! `parsers::link_graph::query` owns Wendao parsers link graph query behavior.

mod api;
#[path = "helpers/mod.rs"]
mod helpers;
mod merge;
#[path = "scan/mod.rs"]
mod scan;
mod state;

pub use self::api::{ParsedLinkGraphQuery, parse_search_query};

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/link_graph/query.rs"]
mod tests;
