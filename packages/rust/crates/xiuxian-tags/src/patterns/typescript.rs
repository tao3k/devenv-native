//! TypeScript ast-grep pattern table.

use super::TS_INTERFACE;

/// TypeScript symbol extraction patterns.
pub const PATTERNS: &[(&str, &str)] = &[("INTERFACE", TS_INTERFACE)];
