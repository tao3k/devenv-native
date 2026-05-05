//! Go ast-grep pattern table.

use super::{GO_FN, GO_STRUCT};

/// Go symbol extraction patterns.
pub const GO_PATTERNS: &[(&str, &str)] = &[("STRUCT", GO_STRUCT), ("FN", GO_FN)];
