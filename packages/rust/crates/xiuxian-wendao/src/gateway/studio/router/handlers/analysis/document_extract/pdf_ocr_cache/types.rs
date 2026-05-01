use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};

pub(super) const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT";
pub(super) const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_BYTES";
pub(super) const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_ENTRIES";
pub(super) const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_MAX_AGE_SECS";
pub(super) const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS";

pub(super) const DEFAULT_OCR_SHARD_CACHE_MAX_BYTES: u64 = 10 * 1024 * 1024 * 1024;
pub(super) const DEFAULT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub(crate) struct PdfOcrShardCache {
    pub(super) root: PathBuf,
    pub(super) policy: PdfOcrShardCachePolicy,
    pub(super) last_sweep: Arc<Mutex<Option<Instant>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PdfOcrShardCachePolicy {
    pub(super) max_bytes: Option<u64>,
    pub(super) max_entries: Option<usize>,
    pub(super) max_age: Option<Duration>,
    pub(super) sweep_interval: Duration,
}

#[derive(Debug, Clone)]
pub(super) struct PdfOcrShardCacheEntry {
    pub(super) path: PathBuf,
    pub(super) bytes: u64,
    pub(super) modified: SystemTime,
}

#[derive(Debug)]
pub(crate) struct PdfOcrShardCacheResolution {
    pub(super) slots: Vec<Option<PdfOcrShardResult>>,
    pub(super) misses: Vec<PdfOcrShardInput>,
    pub(super) miss_positions: Vec<usize>,
    pub(super) hit_count: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct PdfOcrShardCachePruneReport {
    pub(super) scanned_entries: usize,
    pub(super) scanned_bytes: u64,
    pub(super) removed_entries: usize,
    pub(super) removed_bytes: u64,
    pub(super) retained_entries: usize,
    pub(super) retained_bytes: u64,
}
