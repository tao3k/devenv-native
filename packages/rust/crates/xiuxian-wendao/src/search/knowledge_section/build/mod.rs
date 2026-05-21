//! `search::knowledge_section::build` owns Wendao search knowledge section build behavior.

mod orchestration;
mod paths;
mod publish;
mod rows;
mod types;
mod write;

#[cfg(test)]
#[path = "../../../../tests/unit/search/knowledge_section/build/mod.rs"]
mod tests;

#[cfg(any(test, feature = "test-support"))]
pub(crate) use orchestration::ensure_knowledge_section_index_started;
pub(crate) use orchestration::ensure_knowledge_section_index_started_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use publish::publish_knowledge_sections_from_projects;
#[cfg(any(test, feature = "test-support"))]
pub use types::KnowledgeSectionBuildError;
