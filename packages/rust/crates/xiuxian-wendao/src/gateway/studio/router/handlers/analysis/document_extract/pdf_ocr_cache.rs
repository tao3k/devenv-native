use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus, build_ocr_shard_result_batch,
    decode_ocr_shard_result_batches,
};

use super::arrow_cache::{read_arrow_file, write_arrow_file};
use super::pdf_ocr_order::{order_ocr_results_by_inputs, validate_ocr_result_matches_input};

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

const DEFAULT_OCR_SHARD_CACHE_MAX_BYTES: u64 = 10 * 1024 * 1024 * 1024;
const DEFAULT_OCR_SHARD_CACHE_SWEEP_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub(super) struct PdfOcrShardCache {
    root: PathBuf,
    policy: PdfOcrShardCachePolicy,
    last_sweep: Arc<Mutex<Option<Instant>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PdfOcrShardCachePolicy {
    max_bytes: Option<u64>,
    max_entries: Option<usize>,
    max_age: Option<Duration>,
    sweep_interval: Duration,
}

#[derive(Debug, Clone)]
struct PdfOcrShardCacheEntry {
    path: PathBuf,
    bytes: u64,
    modified: SystemTime,
}

#[derive(Debug)]
pub(super) struct PdfOcrShardCacheResolution {
    slots: Vec<Option<PdfOcrShardResult>>,
    misses: Vec<PdfOcrShardInput>,
    miss_positions: Vec<usize>,
    hit_count: usize,
}

impl PdfOcrShardCache {
    pub(super) fn from_environment() -> Self {
        if let Some(root) = std::env::var_os(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV) {
            return Self::new_with_policy(
                PathBuf::from(root),
                PdfOcrShardCachePolicy::from_environment(),
            );
        }
        let cache_root = std::env::var_os("PRJ_CACHE_HOME")
            .map_or_else(|| PathBuf::from(".cache"), PathBuf::from);
        Self::new_with_policy(
            cache_root.join("wendao-document-extract/ocr-shards"),
            PdfOcrShardCachePolicy::from_environment(),
        )
    }

    #[cfg(test)]
    pub(super) fn new(root: PathBuf) -> Self {
        Self::new_with_policy(root, PdfOcrShardCachePolicy::default())
    }

    pub(super) fn new_with_policy(root: PathBuf, policy: PdfOcrShardCachePolicy) -> Self {
        Self {
            root,
            policy,
            last_sweep: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn resolve(&self, inputs: &[PdfOcrShardInput]) -> PdfOcrShardCacheResolution {
        let mut slots = vec![None; inputs.len()];
        let mut misses = Vec::new();
        let mut miss_positions = Vec::new();
        let mut hit_count = 0;

        for (position, input) in inputs.iter().enumerate() {
            if let Some(result) = self.read(input) {
                slots[position] = Some(result);
                hit_count += 1;
            } else {
                misses.push(input.clone());
                miss_positions.push(position);
            }
        }

        PdfOcrShardCacheResolution {
            slots,
            misses,
            miss_positions,
            hit_count,
        }
    }

    pub(super) fn store_successful(
        &self,
        input: &PdfOcrShardInput,
        result: &PdfOcrShardResult,
    ) -> Result<bool, String> {
        if result.status != PdfOcrShardResultStatus::Succeeded {
            return Ok(false);
        }
        validate_ocr_result_matches_input(input, result)?;
        let path = self.cache_path(input);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "create OCR shard cache directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        let batch = build_ocr_shard_result_batch(std::slice::from_ref(result))?;
        let temporary = temporary_cache_path(path.as_path());
        write_arrow_file(temporary.as_path(), std::slice::from_ref(&batch))?;
        fs::rename(temporary.as_path(), path.as_path()).map_err(|error| {
            let _ = fs::remove_file(temporary.as_path());
            format!(
                "publish OCR shard cache `{}` to `{}`: {error}",
                temporary.display(),
                path.display()
            )
        })?;
        self.prune_if_due()?;
        Ok(true)
    }

    pub(super) fn prune(&self) -> Result<PdfOcrShardCachePruneReport, String> {
        prune_ocr_shard_cache(self.root.as_path(), &self.policy)
    }

    fn prune_if_due(&self) -> Result<Option<PdfOcrShardCachePruneReport>, String> {
        if !self.policy.has_limits() {
            return Ok(None);
        }
        let now = Instant::now();
        {
            let last_sweep = self
                .last_sweep
                .lock()
                .map_err(|error| format!("lock OCR shard cache sweep state: {error}"))?;
            if let Some(last_sweep) = *last_sweep
                && now.duration_since(last_sweep) < self.policy.sweep_interval
            {
                return Ok(None);
            }
        }
        let report = self.prune()?;
        let mut last_sweep = self
            .last_sweep
            .lock()
            .map_err(|error| format!("lock OCR shard cache sweep state: {error}"))?;
        *last_sweep = Some(now);
        Ok(Some(report))
    }

    fn read(&self, input: &PdfOcrShardInput) -> Option<PdfOcrShardResult> {
        let path = self.cache_path(input);
        if !path.exists() {
            return None;
        }
        let result = read_existing_result(input, path.as_path());
        if result.is_none() {
            let _ = fs::remove_file(path.as_path());
        }
        result
    }

    fn cache_path(&self, input: &PdfOcrShardInput) -> PathBuf {
        let key = ocr_shard_cache_key(input);
        self.root.join(&key[0..2]).join(format!("{key}.arrow"))
    }
}

impl PdfOcrShardCachePolicy {
    fn from_environment() -> Self {
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

    fn has_limits(&self) -> bool {
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct PdfOcrShardCachePruneReport {
    pub(super) scanned_entries: usize,
    pub(super) scanned_bytes: u64,
    pub(super) removed_entries: usize,
    pub(super) removed_bytes: u64,
    pub(super) retained_entries: usize,
    pub(super) retained_bytes: u64,
}

fn read_existing_result(input: &PdfOcrShardInput, path: &Path) -> Option<PdfOcrShardResult> {
    let batches = read_arrow_file(path).ok()?;
    let mut results = decode_ocr_shard_result_batches(batches.as_slice()).ok()?;
    if results.len() != 1 {
        return None;
    }
    let result = results.pop()?;
    validate_ocr_result_matches_input(input, &result).ok()?;
    (result.status == PdfOcrShardResultStatus::Succeeded).then_some(result)
}

fn prune_ocr_shard_cache(
    root: &Path,
    policy: &PdfOcrShardCachePolicy,
) -> Result<PdfOcrShardCachePruneReport, String> {
    let mut entries = collect_ocr_shard_cache_entries(root)?;
    let scanned_entries = entries.len();
    let scanned_bytes = entries.iter().map(|entry| entry.bytes).sum::<u64>();
    let mut report = PdfOcrShardCachePruneReport {
        scanned_entries,
        scanned_bytes,
        ..PdfOcrShardCachePruneReport::default()
    };

    if let Some(max_age) = policy.max_age {
        let now = SystemTime::now();
        entries.retain(|entry| {
            let is_expired = now
                .duration_since(entry.modified)
                .is_ok_and(|age| age > max_age);
            if is_expired && remove_cache_entry(entry, &mut report).is_err() {
                return true;
            }
            !is_expired
        });
    }

    entries.sort_by(|left, right| {
        left.modified
            .cmp(&right.modified)
            .then(left.path.cmp(&right.path))
    });
    enforce_entry_limit(&mut entries, policy.max_entries, &mut report);
    enforce_byte_limit(&mut entries, policy.max_bytes, &mut report);

    report.retained_entries = entries.len();
    report.retained_bytes = entries.iter().map(|entry| entry.bytes).sum();
    Ok(report)
}

fn enforce_entry_limit(
    entries: &mut Vec<PdfOcrShardCacheEntry>,
    max_entries: Option<usize>,
    report: &mut PdfOcrShardCachePruneReport,
) {
    let Some(max_entries) = max_entries else {
        return;
    };
    while entries.len() > max_entries {
        let entry = entries.remove(0);
        if remove_cache_entry(&entry, report).is_err() {
            break;
        }
    }
}

fn enforce_byte_limit(
    entries: &mut Vec<PdfOcrShardCacheEntry>,
    max_bytes: Option<u64>,
    report: &mut PdfOcrShardCachePruneReport,
) {
    let Some(max_bytes) = max_bytes else {
        return;
    };
    let mut retained_bytes = entries.iter().map(|entry| entry.bytes).sum::<u64>();
    while retained_bytes > max_bytes && !entries.is_empty() {
        let entry = entries.remove(0);
        retained_bytes = retained_bytes.saturating_sub(entry.bytes);
        if remove_cache_entry(&entry, report).is_err() {
            break;
        }
    }
}

fn collect_ocr_shard_cache_entries(root: &Path) -> Result<Vec<PdfOcrShardCacheEntry>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let read_dir = match fs::read_dir(directory.as_path()) {
            Ok(read_dir) => read_dir,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "read OCR shard cache directory `{}`: {error}",
                    directory.display()
                ));
            }
        };
        for entry in read_dir {
            let entry = entry.map_err(|error| {
                format!(
                    "read OCR shard cache entry `{}`: {error}",
                    directory.display()
                )
            })?;
            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(format!(
                        "read OCR shard cache metadata `{}`: {error}",
                        path.display()
                    ));
                }
            };
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("arrow") {
                continue;
            }
            entries.push(PdfOcrShardCacheEntry {
                path,
                bytes: metadata.len(),
                modified: metadata.modified().unwrap_or(UNIX_EPOCH),
            });
        }
    }
    Ok(entries)
}

fn remove_cache_entry(
    entry: &PdfOcrShardCacheEntry,
    report: &mut PdfOcrShardCachePruneReport,
) -> Result<(), String> {
    match fs::remove_file(entry.path.as_path()) {
        Ok(()) => {
            report.removed_entries += 1;
            report.removed_bytes += entry.bytes;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "remove OCR shard cache entry `{}`: {error}",
            entry.path.display()
        )),
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

impl PdfOcrShardCacheResolution {
    pub(super) fn misses(&self) -> &[PdfOcrShardInput] {
        self.misses.as_slice()
    }

    pub(super) fn hit_count(&self) -> usize {
        self.hit_count
    }

    pub(super) fn merge(
        mut self,
        live_results: Vec<PdfOcrShardResult>,
    ) -> Result<Vec<PdfOcrShardResult>, String> {
        let ordered_live = order_ocr_results_by_inputs(self.misses.as_slice(), live_results)?;
        for (position, result) in self.miss_positions.into_iter().zip(ordered_live) {
            self.slots[position] = Some(result);
        }
        self.slots
            .into_iter()
            .enumerate()
            .map(|(position, result)| {
                result.ok_or_else(|| {
                    format!("OCR shard cache merge left input position {position} unresolved")
                })
            })
            .collect()
    }
}

pub(super) fn ocr_shard_cache_key(input: &PdfOcrShardInput) -> String {
    let mut hasher = Sha256::new();
    for fragment in ocr_shard_cache_fragments(input) {
        hasher.update(fragment.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn ocr_shard_cache_fragments(input: &PdfOcrShardInput) -> Vec<String> {
    vec![
        input.contract_version.clone(),
        input.source_path.clone(),
        input.source_content_hash.clone(),
        input.page_index.to_string(),
        input.shard_type.clone(),
        input.region_index.to_string(),
        input.parent_shard_element_id.clone(),
        input.reading_order_key.clone(),
        input.image_mime_type.clone(),
        input.raster_sha256.clone(),
        input.render_profile.clone(),
        input.ocr_profile.clone(),
        input.ocr_engine.clone(),
        input.preferred_languages.join("\u{1f}"),
        f64_bits(input.min_confidence),
        input.preserve_layout.to_string(),
        input.raster_width_px.to_string(),
        input.raster_height_px.to_string(),
        input.render_dpi.to_string(),
        input.rotation_degrees.to_string(),
        f64_bits(input.crop_left),
        f64_bits(input.crop_bottom),
        f64_bits(input.crop_right),
        f64_bits(input.crop_top),
        f64_bits(input.point_to_pixel_scale_x),
        f64_bits(input.point_to_pixel_scale_y),
        input.source_page_pixel_left.to_string(),
        input.source_page_pixel_top.to_string(),
        input.source_page_pixel_right.to_string(),
        input.source_page_pixel_bottom.to_string(),
        input.shard_element_id.clone(),
    ]
}

fn f64_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn temporary_cache_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ocr-shard.arrow");
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nanos))
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/pdf_ocr_cache.rs"]
mod tests;
