//! JavaScript ast-grep pattern table.

use super::{JS_CLASS, JS_FN};

/// JavaScript symbol extraction patterns.
pub const JAVASCRIPT_PATTERNS: &[(&str, &str)] = &[("CLASS", JS_CLASS), ("FN", JS_FN)];
