//! Link-graph sorting terms for stable query result ordering.

use super::enums::{LinkGraphSortField, LinkGraphSortOrder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Result sorting term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSortTerm {
    /// Sort field.
    pub field: LinkGraphSortField,
    /// Sort order for the field.
    pub order: LinkGraphSortOrder,
}

impl Default for LinkGraphSortTerm {
    fn default() -> Self {
        Self {
            field: LinkGraphSortField::Score,
            order: LinkGraphSortOrder::Desc,
        }
    }
}
