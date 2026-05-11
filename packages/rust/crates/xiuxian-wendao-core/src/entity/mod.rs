//! Entity graph records and typed relation labels for knowledge extraction.

mod records;
mod types;

pub use self::records::{Entity, GraphEntity, GraphEntityId, GraphRelation, GraphStats, Relation};
pub use self::types::{EntityType, RelationType};
