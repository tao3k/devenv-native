//! `types::query` owns Wendao types query behavior.

use super::KnowledgeCategory;

/// Search query for knowledge entries.
#[derive(Debug, Clone, Default)]
pub struct KnowledgeSearchQuery {
    /// Search query text
    pub query: String,
    /// Optional category filter
    pub category: Option<KnowledgeCategory>,
    /// Tags to filter by (entries matching ANY tag)
    pub tags: Vec<String>,
    /// Maximum results to return
    pub limit: i32,
}

impl KnowledgeSearchQuery {
    /// Create a new search query.
    #[must_use]
    pub fn new(query: String) -> Self {
        Self {
            query,
            category: None,
            tags: Vec::new(),
            limit: 5,
        }
    }

    /// Set category filter.
    #[must_use]
    pub fn with_category(mut self, category: KnowledgeCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set tags filter.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set result limit.
    #[must_use]
    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = limit;
        self
    }
}
