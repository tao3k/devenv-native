//! `types::stats` owns Wendao types stats behavior.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Knowledge base statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeStats {
    /// Total number of entries
    pub total_entries: i64,
    /// Count per category
    pub entries_by_category: HashMap<String, i64>,
    /// Total unique tags
    pub total_tags: i64,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
}
