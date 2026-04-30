use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus, build_ocr_shard_result_batch,
    decode_ocr_shard_result_batches,
};

use super::key::{ocr_shard_cache_key, temporary_cache_path};
use super::prune::prune_ocr_shard_cache;
use super::types::{
    DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV, PdfOcrShardCache, PdfOcrShardCachePolicy,
    PdfOcrShardCachePruneReport, PdfOcrShardCacheResolution,
};
use crate::gateway::studio::router::handlers::analysis::document_extract::arrow_cache::{
    read_arrow_file, write_arrow_file,
};
use crate::gateway::studio::router::handlers::analysis::document_extract::pdf_ocr_order::validate_ocr_result_matches_input;

impl PdfOcrShardCache {
    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn from_environment()
    -> Self {
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
    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn new(
        root: PathBuf,
    ) -> Self {
        Self::new_with_policy(root, PdfOcrShardCachePolicy::default())
    }

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn new_with_policy(
        root: PathBuf,
        policy: PdfOcrShardCachePolicy,
    ) -> Self {
        Self {
            root,
            policy,
            last_sweep: Arc::new(Mutex::new(None)),
        }
    }

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn resolve(
        &self,
        inputs: &[PdfOcrShardInput],
    ) -> PdfOcrShardCacheResolution {
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

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn store_successful(
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

    pub(in crate::gateway::studio::router::handlers::analysis::document_extract) fn prune(
        &self,
    ) -> Result<PdfOcrShardCachePruneReport, String> {
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
