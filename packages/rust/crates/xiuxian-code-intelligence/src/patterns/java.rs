//! Java ast-grep pattern table.

use super::{JAVA_CLASS, JAVA_METHOD};

/// Java symbol extraction patterns.
pub const JAVA_PATTERNS: &[(&str, &str)] = &[("CLASS", JAVA_CLASS), ("METHOD", JAVA_METHOD)];
