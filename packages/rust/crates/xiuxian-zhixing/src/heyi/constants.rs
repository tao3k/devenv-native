//! Metadata attribute constants used by Zhixing-Heyi task synchronization.

/// Unified Journaling Syntax key for carryover count.
pub const ATTR_JOURNAL_CARRYOVER: &str = "journal:carryover";
/// Key for scheduled time metadata in RFC3339 format.
pub const ATTR_TIMER_SCHEDULED: &str = "timer:scheduled";
/// Key for reminder status metadata (boolean).
pub const ATTR_TIMER_REMINDED: &str = "timer:reminded";
/// Key for reminder delivery target (for example `telegram:1304799691`).
pub const ATTR_TIMER_RECIPIENT: &str = "timer:recipient";
