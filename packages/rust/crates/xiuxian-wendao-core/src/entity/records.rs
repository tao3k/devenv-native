//! Entity and relation records shared by Wendao knowledge graph APIs.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{EntityType, RelationType};

/// Stable identifier for one extracted graph entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEntityId(String);

impl GraphEntityId {
    /// Consumes this id into its serialized representation.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for GraphEntityId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for GraphEntityId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Represents an entity extracted from text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEntity {
    /// Unique identifier.
    pub id: String,
    /// Entity name.
    pub name: String,
    /// Entity type.
    pub entity_type: EntityType,
    /// Brief description.
    pub description: String,
    /// Source document.
    pub source: Option<String>,
    /// Alternative names.
    pub aliases: Vec<String>,
    /// Confidence score.
    pub confidence: f32,
    /// Vector embedding.
    pub vector: Option<Vec<f32>>,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Public compatibility alias for graph entities.
pub type Entity = GraphEntity;

impl GraphEntity {
    /// Create a new entity.
    #[must_use]
    pub fn new(
        id: impl Into<GraphEntityId>,
        name: String,
        entity_type: EntityType,
        description: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into().into_string(),
            name,
            entity_type,
            description,
            source: None,
            aliases: Vec::new(),
            confidence: 1.0,
            vector: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set source.
    #[must_use]
    pub fn with_source(mut self, source: Option<String>) -> Self {
        self.source = source;
        self.updated_at = Utc::now();
        self
    }

    /// Set aliases.
    #[must_use]
    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self.updated_at = Utc::now();
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
        self
    }

    /// Set vector embedding.
    #[must_use]
    pub fn with_vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self.updated_at = Utc::now();
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
        self
    }
}

/// Represents a relation between two entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRelation {
    /// Unique identifier.
    pub id: String,
    /// Source entity name.
    pub source: String,
    /// Target entity name.
    pub target: String,
    /// Relation type.
    pub relation_type: RelationType,
    /// Brief description.
    pub description: String,
    /// Source document.
    pub source_doc: Option<String>,
    /// Confidence score.
    pub confidence: f32,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Public compatibility alias for graph relations.
pub type Relation = GraphRelation;

impl GraphRelation {
    /// Create a new relation.
    #[must_use]
    pub fn new(
        source: String,
        target: String,
        relation_type: RelationType,
        description: String,
    ) -> Self {
        let id = format!(
            "{}|{}|{}",
            source.to_lowercase().replace(' ', "_"),
            relation_type.to_string().to_lowercase().replace(' ', "_"),
            target.to_lowercase().replace(' ', "_")
        );
        Self {
            id,
            source,
            target,
            relation_type,
            description,
            source_doc: None,
            confidence: 1.0,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Set source document.
    #[must_use]
    pub fn with_source_doc(mut self, source_doc: Option<String>) -> Self {
        self.source_doc = source_doc;
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Knowledge graph statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphStats {
    /// Total entities.
    pub total_entities: i64,
    /// Total relations.
    pub total_relations: i64,
    /// Entities by type.
    pub entities_by_type: HashMap<String, i64>,
    /// Relations by type.
    pub relations_by_type: HashMap<String, i64>,
    /// Last update.
    pub last_updated: Option<DateTime<Utc>>,
}
