//! Rust ast-grep pattern table.

use super::{RUST_ENUM, RUST_FN, RUST_IMPL, RUST_STRUCT, RUST_TRAIT};

/// Rust symbol extraction patterns.
pub const PATTERNS: &[(&str, &str)] = &[
    ("STRUCT", RUST_STRUCT),
    ("FN", RUST_FN),
    ("ENUM", RUST_ENUM),
    ("TRAIT", RUST_TRAIT),
    ("IMPL", RUST_IMPL),
];
