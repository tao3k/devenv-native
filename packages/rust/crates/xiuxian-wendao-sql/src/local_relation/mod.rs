//! Local relation engine abstraction and implementations.

mod duckdb;
mod types;

pub use duckdb::DuckDbLocalRelationEngine;
pub use types::{
    LocalRelationEngine, LocalRelationEngineKind, LocalRelationMaterializationState,
    LocalRelationRegistrationHint,
};
