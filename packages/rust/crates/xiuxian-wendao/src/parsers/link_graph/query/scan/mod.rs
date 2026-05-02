mod api;
#[path = "directives/mod.rs"]
mod directives;

pub(in crate::parsers::link_graph::query) use self::api::parse_terms;
