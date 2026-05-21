//! `entity::query` owns Wendao entity query behavior.

use super::{EntityType, RelationType};

/// Search query for entities.
#[derive(Debug, Clone)]
pub struct EntitySearchQuery {
    /// Query text
    pub query: String,
    /// Entity type filter
    pub entity_type: Option<EntityType>,
    /// Maximum results
    pub limit: i32,
}

impl Default for EntitySearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            entity_type: None,
            limit: 10,
        }
    }
}

impl EntitySearchQuery {
    /// Create new query.
    #[must_use]
    pub fn new(query: String) -> Self {
        Self {
            query,
            ..Default::default()
        }
    }

    /// Set entity type filter.
    #[must_use]
    pub fn with_entity_type(mut self, entity_type: EntityType) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Set limit.
    #[must_use]
    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = limit;
        self
    }
}

/// Multi-hop search options.
#[derive(Debug, Clone)]
pub struct MultiHopOptions {
    /// Starting entity names
    pub start_entities: Vec<String>,
    /// Relation types to follow
    pub relation_types: Vec<RelationType>,
    /// Maximum hops
    pub max_hops: usize,
    /// Maximum results
    pub limit: i32,
}

impl Default for MultiHopOptions {
    fn default() -> Self {
        Self {
            start_entities: Vec::new(),
            relation_types: Vec::new(),
            max_hops: 2,
            limit: 20,
        }
    }
}
