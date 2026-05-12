//! `parsers::markdown::relations` owns Wendao parsers markdown relations behavior.

mod api;
mod keys;
mod targets;
mod types;

pub use api::{extract_property_relations, parse_property_relations};
pub use types::{ExplicitRelationSource, ExplicitRelationTarget, ExplicitSectionRelation};

#[cfg(test)]
pub(crate) use targets::parse_relation_targets;

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/markdown/relations.rs"]
mod tests;
