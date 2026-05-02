//! File event topic grouping.

use super::{FILE_CHANGED, FILE_CREATED, FILE_DELETED, FILE_RENAMED};

/// File-related topics.
pub const TOPICS: &[(&str, &str)] = &[
    ("CHANGED", FILE_CHANGED),
    ("CREATED", FILE_CREATED),
    ("DELETED", FILE_DELETED),
    ("RENAMED", FILE_RENAMED),
];
