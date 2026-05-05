#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(any(test, feature = "test-support"))]
pub use build::KnowledgeSectionBuildError;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::ensure_knowledge_section_index_started;
pub(crate) use build::ensure_knowledge_section_index_started_with_scanned_files;
#[cfg(any(test, feature = "test-support"))]
pub(crate) use build::publish_knowledge_sections_from_projects;
pub use query::KnowledgeSectionSearchError;
pub(crate) use query::search_knowledge_sections;
