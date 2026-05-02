//! Standard property drawer attribute keys.

/// Explicit node identifier - takes precedence over `structural_path`.
pub const ID: &str = "ID";
/// Node status: STABLE | DRAFT | DEPRECATED.
pub const STATUS: &str = "STATUS";
/// Semantic contract constraint, such as `must_contain("Rust", "Lock")`.
pub const CONTRACT: &str = "CONTRACT";
