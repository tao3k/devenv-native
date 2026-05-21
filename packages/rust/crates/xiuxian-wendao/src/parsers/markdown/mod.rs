//! Core markdown note parsing and Wendao-owned enrichments.

#[path = "api.rs"]
mod api;
/// Public Wendao boundary.
#[path = "code_observation/mod.rs"]
pub mod code_observation;
#[path = "content.rs"]
mod content;
#[path = "links/mod.rs"]
mod links;
#[path = "paths.rs"]
mod paths;
#[path = "relations/mod.rs"]
mod relations;
#[path = "section_create/mod.rs"]
pub(crate) mod section_create;
#[path = "sections/mod.rs"]
mod sections;
#[path = "time.rs"]
mod time;
#[path = "types.rs"]
mod types;

pub use self::api::parse_note;
#[cfg(feature = "search-runtime")]
pub(crate) use self::api::{adapt_markdown_note, adapt_org_note};
pub use self::code_observation::{CodeObservation, extract_observations};
#[cfg(all(test, not(feature = "search-runtime")))]
pub(crate) use self::links::ResolvedNoteReference;
#[cfg(feature = "search-runtime")]
pub use self::links::{ResolvedNoteReference, extract_resolved_note_references};
#[cfg(feature = "search-runtime")]
pub(crate) use self::paths::is_org_note;
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
