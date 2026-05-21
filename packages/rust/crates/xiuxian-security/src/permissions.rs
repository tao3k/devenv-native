//! `PermissionGatekeeper` zero-trust access control.
//!
//! Validates skill tool calls against declared permissions.
//!
//! Permission format:
//! - Exact: `"filesystem:read"` allows only `"filesystem:read"`.
//! - Wildcard category: `"filesystem:*"` allows any `"filesystem:*"` tool.
//! - Admin: `"*"` allows everything.

/// Zero-trust tool permission checker.
pub struct PermissionGatekeeper;

impl PermissionGatekeeper {
    /// Check if a tool execution is allowed by the given permissions.
    ///
    /// `tool_name`: Full tool name (e.g., `filesystem.read_file`).
    /// `permissions`: Permission patterns (e.g., [`filesystem:*`]).
    ///
    /// Returns `true` when at least one permission pattern matches the tool.
    #[must_use]
    pub fn check(tool_name: &str, permissions: &[String]) -> bool {
        permissions
            .iter()
            .any(|pattern| Self::matches_pattern(tool_name, pattern))
    }

    fn matches_pattern(tool: &str, pattern: &str) -> bool {
        // Admin permission allows everything.
        if pattern == "*" {
            return true;
        }

        // "filesystem:*" should match "filesystem.read_file".
        if let Some(prefix) = pattern.strip_suffix(":*") {
            let standardized_prefix = prefix.replace(':', ".");
            return tool.starts_with(&standardized_prefix);
        }

        if let Some(prefix) = pattern.strip_suffix(".*") {
            return tool.starts_with(prefix);
        }

        // Normalize separators for exact comparison.
        let normalized_tool = tool.replace(':', ".");
        let normalized_pattern = pattern.replace(':', ".");
        normalized_tool == normalized_pattern
    }
}
