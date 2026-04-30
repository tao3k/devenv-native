//! Core markdown note parsing and Wendao-owned enrichments.

#[path = "markdown/api.rs"]
mod api;
#[path = "markdown/code_observation/mod.rs"]
pub mod code_observation;
#[path = "markdown/content.rs"]
mod content;
#[path = "markdown/links/mod.rs"]
mod links;
#[path = "markdown/paths.rs"]
mod paths;
#[path = "markdown/relations/mod.rs"]
mod relations;
#[path = "markdown/section_create/mod.rs"]
pub(crate) mod section_create;
#[path = "markdown/sections/mod.rs"]
mod sections;
#[path = "markdown/time.rs"]
mod time;
#[path = "markdown/types.rs"]
mod types;

pub use self::api::parse_note;
#[cfg(feature = "search-runtime")]
pub(crate) use self::api::{adapt_markdown_note, adapt_org_note};
pub use self::code_observation::{CodeObservation, extract_observations};
#[cfg(test)]
pub(crate) use self::links::ResolvedNoteReference;
#[cfg(any(test, feature = "studio"))]
pub(crate) use self::links::extract_resolved_note_references;
pub(crate) use self::paths::is_org_note;
pub use self::paths::{is_supported_note, normalize_alias};
pub use self::relations::{
    ExplicitRelationSource, ExplicitRelationTarget, ExplicitSectionRelation,
    extract_property_relations, parse_property_relations,
};
pub use self::sections::{LogbookEntry, ParsedSection};
pub use self::types::ParsedNote;

#[cfg(test)]
#[path = "../../tests/unit/parsers/markdown/document.rs"]
mod document_tests;
#[cfg(test)]
#[path = "../../tests/unit/parsers/markdown/namespace.rs"]
mod namespace_tests;
