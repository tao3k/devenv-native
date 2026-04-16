//! Core markdown note parsing and Wendao-owned enrichments.

#[path = "api.rs"]
mod api;
pub mod code_observation;
mod content;
mod links;
mod paths;
mod relations;
#[cfg(any(feature = "zhenfa-router", test))]
pub(crate) mod section_create;
mod sections;
mod time;
mod types;

#[cfg(feature = "search-runtime")]
pub(crate) use self::api::adapt_markdown_note;
pub use self::api::parse_note;
pub use self::code_observation::{CodeObservation, extract_observations};
pub use self::paths::{is_supported_note, normalize_alias};
pub use self::relations::{
    ExplicitRelationSource, ExplicitRelationTarget, ExplicitSectionRelation,
    extract_property_relations, parse_property_relations,
};
pub use self::sections::{LogbookEntry, ParsedSection};
pub use self::types::ParsedNote;

#[cfg(test)]
#[path = "../../../tests/unit/parsers/markdown/document.rs"]
mod document_tests;
#[cfg(test)]
#[path = "../../../tests/unit/parsers/markdown/namespace.rs"]
mod namespace_tests;
