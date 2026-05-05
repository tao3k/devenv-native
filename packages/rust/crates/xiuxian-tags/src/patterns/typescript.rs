//! TypeScript ast-grep pattern table.

use super::TS_INTERFACE;

/// TypeScript symbol extraction patterns.
pub const TYPESCRIPT_PATTERNS: &[(&str, &str)] = &[("INTERFACE", TS_INTERFACE)];
