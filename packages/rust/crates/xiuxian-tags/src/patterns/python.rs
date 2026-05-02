//! Python ast-grep pattern table.

use super::{PYTHON_ASYNC_DEF, PYTHON_CLASS, PYTHON_DEF};

/// Python symbol extraction patterns.
pub const PATTERNS: &[(&str, &str)] = &[
    ("CLASS", PYTHON_CLASS),
    ("DEF", PYTHON_DEF),
    ("ASYNC_DEF", PYTHON_ASYNC_DEF),
];
