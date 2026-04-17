mod api;
mod normalize;
mod parse_target;
mod types;

pub(crate) use api::extract_resolved_note_references;
pub(in crate::parsers::markdown) use api::{
    extract_link_targets_from_occurrences, extract_link_targets_from_occurrences_in_range,
};
#[cfg(test)]
pub(crate) use types::ResolvedNoteReference;
