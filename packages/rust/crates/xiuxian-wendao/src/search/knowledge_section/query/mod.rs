#[path = "lookup/mod.rs"]
mod lookup;

#[cfg(test)]
#[path = "../../../../tests/unit/search/knowledge_section/query/mod.rs"]
mod tests;

pub use lookup::KnowledgeSectionSearchError;
pub(crate) use lookup::search_knowledge_sections;
