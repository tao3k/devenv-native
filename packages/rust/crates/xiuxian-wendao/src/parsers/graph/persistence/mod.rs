//! `parsers::graph::persistence` owns Wendao parsers graph persistence behavior.

mod api;
mod entity;
mod relation;

pub use self::api::{entity_from_dict, relation_from_dict};

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/graph/persistence.rs"]
mod tests;
