#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(test)]
pub(crate) use build::ensure_knowledge_section_index_started;
pub(crate) use build::ensure_knowledge_section_index_started_with_scanned_files;
#[cfg(test)]
pub(crate) use build::{KnowledgeSectionBuildError, publish_knowledge_sections_from_projects};
pub(crate) use query::{KnowledgeSectionSearchError, search_knowledge_sections};
