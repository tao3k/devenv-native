//! Projection labels for deterministic repository page families.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Deterministic projected page family derived from stage-1 repository truth.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPageKind {
    /// Reference-oriented projected page.
    #[default]
    Reference,
    /// How-to oriented projected page.
    HowTo,
    /// Tutorial-oriented projected page.
    Tutorial,
    /// Explanation-oriented projected page.
    Explanation,
}
