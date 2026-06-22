use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobCacheBackend, ArtifactBlobCacheBackendConfig, ArtifactBlobWrite,
};
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus, build_ocr_shard_result_batch,
    decode_ocr_shard_result_batches,
};

use super::key::{ocr_shard_artifact_key, ocr_shard_cache_key, temporary_cache_path};
use super::prune::prune_ocr_shard_cache;
use super::types::{
    PdfOcrShardCache, PdfOcrShardCachePolicy, PdfOcrShardCachePruneReport,
    PdfOcrShardCacheResolution,
};
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    read_arrow_bytes, read_arrow_file, write_arrow_bytes, write_arrow_file,
};
use crate::studio::router::handlers::analysis::document_extract::pdf_ocr_order::validate_ocr_result_matches_input;

impl PdfOcrShardCache {
    pub(crate) fn from_environment() -> Self {
        Self::artifact_blob_from_environment().unwrap_or_else(|reason| {
            panic!("configure PDF OCR shard ArtifactBlobCache: {reason}");
        })
    }

    fn artifact_blob_from_environment() -> Result<Self, String> {
        let config = ArtifactBlobCacheBackendConfig::from_lookup(&artifact_cache_env_lookup)
            .map_err(|error| {
                format!("resolve ArtifactBlobCache backend for PDF OCR shard cache: {error}")
            })?;
        let root = config.root().to_path_buf();
        let backend = config.build().map_err(|error| {
            format!("build ArtifactBlobCache backend for PDF OCR shard cache: {error}")
        })?;
        Ok(Self::new_with_artifact_cache_backend(
            root,
            PdfOcrShardCachePolicy::from_environment(),
            Arc::new(backend),
        ))
    }

    #[cfg(test)]
    pub(crate) fn new(root: PathBuf) -> Self {
        Self::new_with_policy(root, PdfOcrShardCachePolicy::default())
    }

    #[cfg(test)]
    pub(crate) fn new_with_policy(root: PathBuf, policy: PdfOcrShardCachePolicy) -> Self {
        Self {
            root,
            policy,
            last_sweep: Arc::new(Mutex::new(None)),
            artifact_cache: None,
        }
    }

    pub(crate) fn new_with_artifact_cache_backend(
        root: PathBuf,
        policy: PdfOcrShardCachePolicy,
        artifact_cache: Arc<ArtifactBlobCacheBackend>,
    ) -> Self {
        Self {
            root,
            policy,
            last_sweep: Arc::new(Mutex::new(None)),
            artifact_cache: Some(artifact_cache),
        }
    }

    pub(crate) fn resolve(&self, inputs: &[PdfOcrShardInput]) -> PdfOcrShardCacheResolution {
        inputs
            .iter()
            .enumerate()
            .fold(
                PdfOcrShardCacheResolutionBuilder::new(inputs.len()),
                |mut builder, (position, input)| {
                    builder.push(position, input, self.read(input));
                    builder
                },
            )
            .finish()
    }

    pub(crate) fn store_successful(
        &self,
        input: &PdfOcrShardInput,
        result: &PdfOcrShardResult,
    ) -> Result<bool, String> {
        if result.status != PdfOcrShardResultStatus::Succeeded {
            return Ok(false);
        }
        validate_ocr_result_matches_input(input, result)?;
        let batch = build_ocr_shard_result_batch(std::slice::from_ref(result))?;
        if let Some(cache) = &self.artifact_cache {
            let bytes = write_arrow_bytes(std::slice::from_ref(&batch))?;
            let key = ocr_shard_artifact_key(input)?;
            cache
                .write(&key, ArtifactBlobWrite::new(bytes.as_slice()))
                .map_err(|error| format!("write PDF OCR shard ArtifactBlobCache entry: {error}"))?;
            return Ok(true);
        }

        let path = self.cache_path(input);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "create OCR shard cache directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
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

    pub(crate) fn prune(&self) -> Result<PdfOcrShardCachePruneReport, String> {
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
        if let Some(cache) = &self.artifact_cache {
            return Self::read_artifact_blob(input, cache);
        }
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

    fn read_artifact_blob(
        input: &PdfOcrShardInput,
        cache: &ArtifactBlobCacheBackend,
    ) -> Option<PdfOcrShardResult> {
        let key = ocr_shard_artifact_key(input).ok()?;
        let bytes = cache.read(&key).ok()??;
        let batches = read_arrow_bytes(bytes.bytes()).ok()?;
        let result = result_from_batches(input, batches.as_slice());
        if result.is_none() {
            let _ = cache.remove(&key);
        }
        result
    }
}

fn artifact_cache_env_lookup(key: &str) -> Option<String> {
    std::env::var(key).ok().or_else(|| {
        (key == "PRJ_CACHE_HOME").then(|| {
            std::env::current_dir().map_or_else(
                |_| ".cache".to_string(),
                |root| root.join(".cache").to_string_lossy().into_owned(),
            )
        })
    })
}

struct PdfOcrShardCacheResolutionBuilder {
    slots: Vec<Option<PdfOcrShardResult>>,
    misses: Vec<PdfOcrShardInput>,
    miss_positions: Vec<usize>,
    hit_count: usize,
}

impl PdfOcrShardCacheResolutionBuilder {
    fn new(input_count: usize) -> Self {
        Self {
            slots: vec![None; input_count],
            misses: Vec::new(),
            miss_positions: Vec::new(),
            hit_count: 0,
        }
    }

    fn push(
        &mut self,
        position: usize,
        input: &PdfOcrShardInput,
        result: Option<PdfOcrShardResult>,
    ) {
        if let Some(result) = result {
            self.slots[position] = Some(result);
            self.hit_count += 1;
        } else {
            self.misses.push(input.clone());
            self.miss_positions.push(position);
        }
    }

    fn finish(self) -> PdfOcrShardCacheResolution {
        PdfOcrShardCacheResolution {
            slots: self.slots,
            misses: self.misses,
            miss_positions: self.miss_positions,
            hit_count: self.hit_count,
        }
    }
}

fn read_existing_result(input: &PdfOcrShardInput, path: &Path) -> Option<PdfOcrShardResult> {
    let batches = read_arrow_file(path).ok()?;
    result_from_batches(input, batches.as_slice())
}

fn result_from_batches(
    input: &PdfOcrShardInput,
    batches: &[arrow::record_batch::RecordBatch],
) -> Option<PdfOcrShardResult> {
    let mut results = decode_ocr_shard_result_batches(batches).ok()?;
    if results.len() != 1 {
        return None;
    }
    let result = results.pop()?;
    validate_ocr_result_matches_input(input, &result).ok()?;
    (result.status == PdfOcrShardResultStatus::Succeeded).then_some(result)
}
