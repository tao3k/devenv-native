mod api;
mod normalize;
mod parse_target;
mod types;

#[cfg(any(test, feature = "search-runtime"))]
pub use api::extract_resolved_note_references;
pub(in crate::parsers::markdown) use api::{
    extract_link_targets_from_occurrences, extract_link_targets_from_occurrences_in_range,
};
#[cfg(any(test, feature = "search-runtime"))]
pub use types::ResolvedNoteReference;
