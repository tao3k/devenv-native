//! `search::knowledge_section::query::lookup` owns Wendao knowledge section query lookup behavior.

mod candidates;
mod error;
mod helpers;
mod route;

#[cfg(test)]
pub(crate) use candidates::{
    KnowledgeCandidate, candidate_path_key, compare_candidates, retained_window,
};
pub use error::KnowledgeSectionSearchError;
pub(crate) use route::search_knowledge_sections;
