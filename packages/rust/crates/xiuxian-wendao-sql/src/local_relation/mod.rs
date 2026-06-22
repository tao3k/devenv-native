//! Local relation engine abstraction and implementations.

#[cfg(not(feature = "duckdb"))]
mod disabled;
#[cfg(feature = "duckdb")]
mod duckdb;
mod types;

#[cfg(not(feature = "duckdb"))]
pub use disabled::FeatureDisabledDuckDbLocalRelationEngine as DuckDbLocalRelationEngine;
#[cfg(feature = "duckdb")]
pub use duckdb::DuckDbLocalRelationEngine;
pub use types::{
    LocalRelationEngine, LocalRelationEngineKind, LocalRelationMaterializationState,
    LocalRelationRegistrationHint,
};
