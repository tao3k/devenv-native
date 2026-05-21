//! `parsers::link_graph::query::scan` owns Wendao link graph query scan behavior.

mod api;
#[path = "directives/mod.rs"]
mod directives;

pub(in crate::parsers::link_graph::query) use self::api::parse_terms;
