/// Metadata attribute for journal carryover state.
pub const ATTR_JOURNAL_CARRYOVER: &str = "journal:carryover";
/// Metadata attribute for scheduled timer timestamp.
pub const ATTR_TIMER_SCHEDULED: &str = "timer:scheduled";
/// Metadata attribute for reminded timer state.
pub const ATTR_TIMER_REMINDED: &str = "timer:reminded";

/// Specialized error type for Zhixing domain indexing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Internal indexing failure.
    #[error("{0}")]
    Internal(String),
}

/// Result type for Zhixing domain indexing.
pub type Result<T> = std::result::Result<T, Error>;
