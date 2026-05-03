mod orchestration;
mod paths;
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
pub use types::KnowledgeSectionBuildError;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use write::publish_knowledge_sections_from_projects;
