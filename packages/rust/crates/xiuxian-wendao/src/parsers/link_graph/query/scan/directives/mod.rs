//! `parsers::link_graph::query::scan::directives` owns Wendao query scan directives behavior.

mod apply;
mod filters;
mod links;
mod search;
mod structure;

pub(in crate::parsers::link_graph::query::scan) use self::apply::apply_directive;
