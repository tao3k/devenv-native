//! Local relation engine abstraction and `DataFusion` implementation.

mod datafusion;
mod types;

pub use datafusion::DataFusionLocalRelationEngine;
pub use types::{
    LocalRelationEngine, LocalRelationEngineKind, LocalRelationMaterializationState,
    LocalRelationRegistrationHint,
};
