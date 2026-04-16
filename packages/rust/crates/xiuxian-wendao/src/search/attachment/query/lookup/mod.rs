mod decode;
mod helpers;
mod route;
mod scan;
mod scoring;
mod types;

pub(crate) use route::search_attachment_hits;
#[cfg(test)]
pub(crate) use scoring::{compare_candidates, retained_window};
#[cfg(test)]
pub(crate) use types::AttachmentCandidate;
pub(crate) use types::AttachmentSearchError;
