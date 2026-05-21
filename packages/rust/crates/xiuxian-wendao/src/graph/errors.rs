//! `graph::errors` owns Wendao graph errors behavior.

use thiserror::Error;

/// Graph errors.
#[derive(Debug, Error)]
pub enum GraphError {
    /// The requested entity was not found.
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    /// A relation with this ID already exists.
    #[error("Relation already exists: {0}")]
    RelationExists(String),
    /// The relation references invalid source/target entities.
    #[error("Invalid relation: source={0}, target={1}")]
    /// Tuple payload boundary: this public variant mirrors an existing relation payload shape.
    InvalidRelation(String, String),
}
