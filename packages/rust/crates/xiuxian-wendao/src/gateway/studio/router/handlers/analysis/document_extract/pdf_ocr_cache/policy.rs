use std::time::Duration;

use super::types::{
    DEFAULT_OCR_SHARD_CACHE_MAX_BYTES, DEFAULT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS,
    DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS_ENV,
    DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES_ENV,
    DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES_ENV,
    DOCUMENT_EXTRACT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS_ENV, PdfOcrShardCachePolicy,
};

impl PdfOcrShardCachePolicy {
    pub(super) fn from_environment() -> Self {
        Self {
            max_bytes: optional_u64_env(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES_ENV)
                .or(Some(DEFAULT_OCR_SHARD_CACHE_MAX_BYTES)),
            max_entries: optional_usize_env(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES_ENV),
            max_age: optional_u64_env(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS_ENV)
                .map(Duration::from_secs),
            sweep_interval: optional_u64_env(
                DOCUMENT_EXTRACT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS_ENV,
            )
            .map_or(
                Duration::from_secs(DEFAULT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS),
                Duration::from_secs,
            ),
        }
    }

    pub(super) fn has_limits(&self) -> bool {
        self.max_bytes.is_some() || self.max_entries.is_some() || self.max_age.is_some()
    }
}

impl Default for PdfOcrShardCachePolicy {
    fn default() -> Self {
        Self {
            max_bytes: Some(DEFAULT_OCR_SHARD_CACHE_MAX_BYTES),
            max_entries: None,
            max_age: None,
            sweep_interval: Duration::from_secs(DEFAULT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS),
        }
    }
}

fn optional_u64_env(key: &str) -> Option<u64> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
}

fn optional_usize_env(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
}
