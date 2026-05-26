use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use arrow::array::{Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use serde_json::json;
use sha2::Digest;
use xiuxian_db_store::artifact_cache::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV, ARTIFACT_CACHE_FLUSHERS_ENV,
    ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
    ARTIFACT_CACHE_RECLAIMERS_ENV, ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_RUNTIME_WORKERS_ENV, ARTIFACT_CACHE_STORAGE_BYTES_ENV, ArtifactBlobCacheBackend,
    ArtifactBlobCacheBackendConfig, ArtifactCacheError, ArtifactKind, AttachmentArtifactKeyParts,
    attachment_artifact_key, fetch_through_artifact_bytes,
};
use xiuxian_db_store::{decode_record_batches_ipc, encode_record_batches_ipc};
use xiuxian_wendao_attachments::legacy_office::{
    LegacyOfficeExtraction, LegacyOfficeFormat, LegacyOfficeQualityMetrics, extract_legacy_office,
    is_supported_legacy_office_path, legacy_office_format, legacy_office_quality_metrics,
};
use xiuxian_wendao_server::transport::DocumentExtractFlightRouteResponse;

use super::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, build_native_text_resource_batch, write_arrow_file,
};

const LEGACY_OFFICE_PROJECTION_SCHEMA: &str = "xiuxian_wendao.legacy_office_projection.v1";
const LEGACY_OFFICE_PROJECTION_PROFILE: &str = "legacy-office-projection-v1-litchi-f9ca012";
const LEGACY_OFFICE_PROJECTION_REPORT_NAME: &str = "_legacy_office_projection_report.json";

#[derive(Debug, Clone)]
pub(super) struct LegacyOfficeProjection {
    pub(super) format: LegacyOfficeFormat,
    pub(super) text: String,
    pub(super) markdown: String,
    pub(super) quality_metrics: LegacyOfficeQualityMetrics,
}

#[derive(Debug, Clone)]
struct LegacyOfficeProjectionRead {
    projection: LegacyOfficeProjection,
    source_digest: String,
    cache_backend: String,
    cache_status: String,
    cache_byte_len: usize,
}

static LEGACY_OFFICE_ARTIFACT_CACHE_BACKENDS: OnceLock<
    Mutex<BTreeMap<String, Arc<ArtifactBlobCacheBackend>>>,
> = OnceLock::new();

impl StudioDocumentExtractFlightRouteProvider {
    pub(super) async fn sync_legacy_office_document_extract_batch(
        &self,
        source: &Path,
        output: &Path,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let batches = write_legacy_office_document_extract_output(source, output).await?;
        write_arrow_file(
            output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).as_path(),
            batches.as_slice(),
        )?;
        tokio::fs::File::create(output.join("_complete.marker"))
            .await
            .map_err(|error| format!("touch legacy Office document extract marker: {error}"))?;
        if let Err(error) = self.persist_sync_output_artifact(source, output).await {
            log::warn!("failed to persist legacy Office document extract artifact: {error}");
        }
        Ok(DocumentExtractFlightRouteResponse::from_batches(batches))
    }
}

pub(super) fn is_legacy_office_source(path: &Path) -> bool {
    is_supported_legacy_office_path(path)
}

pub(super) async fn write_legacy_office_document_extract_output(
    source: &Path,
    output: &Path,
) -> Result<Vec<RecordBatch>, String> {
    tokio::fs::create_dir_all(output).await.map_err(|error| {
        format!(
            "create legacy Office document extract output `{}`: {error}",
            output.display()
        )
    })?;
    let projection_read = tokio::task::spawn_blocking({
        let source = source.to_path_buf();
        move || extract_or_restore_legacy_office_projection(source.as_path())
    })
    .await
    .map_err(|error| format!("join legacy Office extraction task: {error}"))??;
    let projection = &projection_read.projection;
    let markdown_path = output
        .join(
            source
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("legacy-office"),
        )
        .with_extension("md");
    tokio::fs::write(markdown_path.as_path(), projection.markdown.as_bytes())
        .await
        .map_err(|error| {
            format!(
                "write legacy Office markdown resource `{}`: {error}",
                markdown_path.display()
            )
        })?;
    write_legacy_office_projection_report(output, &projection_read).await?;
    let source_path = source.to_string_lossy();
    let markdown_path = markdown_path.to_string_lossy();
    Ok(vec![build_native_text_resource_batch(
        source_path.as_ref(),
        "legacy-office-document",
        markdown_path.as_ref(),
        projection.format.extension(),
        projection.markdown.as_str(),
        "text/markdown",
        "_legacy_office_document",
    )?])
}

fn extract_or_restore_legacy_office_projection(
    source: &Path,
) -> Result<LegacyOfficeProjectionRead, String> {
    let format = legacy_office_format(source)
        .ok_or_else(|| format!("unsupported legacy Office source `{}`", source.display()))?;
    let source_digest = source_sha256(source)?;
    let cache = legacy_office_artifact_cache_from_environment()?;
    if let Some(cache) = cache.as_deref() {
        let key = legacy_office_projection_artifact_key(&source_digest, format)?;
        let source = source.to_path_buf();
        let build_source_digest = source_digest.clone();
        let artifact = fetch_through_artifact_bytes(cache, &key, move || {
            let projection = legacy_office_projection_from_extraction(
                extract_legacy_office(source.as_path()).map_err(|error| {
                    ArtifactCacheError::backend(
                        "legacy-office",
                        "extracting Arrow projection",
                        error,
                    )
                })?,
            );
            let batch = build_legacy_office_projection_batch(&build_source_digest, &projection)
                .map_err(|error| {
                    ArtifactCacheError::backend("legacy-office", "building Arrow projection", error)
                })?;
            encode_record_batches_ipc(&[batch]).map_err(|error| {
                ArtifactCacheError::backend(
                    "legacy-office",
                    "encoding Arrow projection IPC",
                    error.to_string(),
                )
            })
        })
        .map_err(|error| {
            format!("fetch through legacy Office Arrow projection artifact: {error}")
        })?;
        let batches = decode_record_batches_ipc(artifact.bytes()).map_err(|error| {
            format!("decode legacy Office Arrow projection artifact IPC: {error}")
        })?;
        let projection =
            legacy_office_projection_from_batches(batches.as_slice(), &source_digest, format)?;
        return Ok(LegacyOfficeProjectionRead {
            projection,
            source_digest,
            cache_backend: artifact.backend_name().to_string(),
            cache_status: format!("{:?}", artifact.status()),
            cache_byte_len: artifact.byte_len(),
        });
    }
    Ok(LegacyOfficeProjectionRead {
        projection: legacy_office_projection_from_extraction(extract_legacy_office(source)?),
        source_digest,
        cache_backend: "disabled".to_string(),
        cache_status: "Disabled".to_string(),
        cache_byte_len: 0,
    })
}

fn legacy_office_projection_from_extraction(
    extraction: LegacyOfficeExtraction,
) -> LegacyOfficeProjection {
    LegacyOfficeProjection {
        format: extraction.format,
        text: extraction.text,
        markdown: extraction.markdown,
        quality_metrics: extraction.quality_metrics,
    }
}

fn legacy_office_artifact_cache_from_environment()
-> Result<Option<Arc<ArtifactBlobCacheBackend>>, String> {
    if !artifact_cache_env_present() {
        return Ok(None);
    }
    let config = ArtifactBlobCacheBackendConfig::from_env()
        .map_err(|error| format!("resolve legacy Office ArtifactBlobCache backend: {error}"))?;
    let key = artifact_cache_backend_config_key(&config);
    let backends =
        LEGACY_OFFICE_ARTIFACT_CACHE_BACKENDS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut backends = backends.lock().map_err(|_| {
        "legacy Office ArtifactBlobCache backend registry lock poisoned".to_string()
    })?;
    if let Some(backend) = backends.get(&key) {
        return Ok(Some(Arc::clone(backend)));
    }
    let backend = Arc::new(config.build().map_err(|error| {
        format!(
            "build legacy Office ArtifactBlobCache backend `{}` at `{}`: {error}",
            config.kind().as_str(),
            config.root().display()
        )
    })?);
    backends.insert(key, Arc::clone(&backend));
    Ok(Some(backend))
}

fn legacy_office_projection_artifact_key(
    source_digest: &str,
    format: LegacyOfficeFormat,
) -> Result<xiuxian_db_store::artifact_cache::ArtifactKey, String> {
    attachment_artifact_key(AttachmentArtifactKeyParts {
        kind: ArtifactKind::ArrowIpcBatch,
        source_digest: source_digest.to_string(),
        profile_digest: digest_component([LEGACY_OFFICE_PROJECTION_PROFILE, format.extension()]),
        shard_digest: "document".to_string(),
    })
    .map_err(|error| format!("build legacy Office Arrow projection artifact key: {error}"))
}

pub(super) fn build_legacy_office_projection_batch(
    source_digest: &str,
    projection: &LegacyOfficeProjection,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("schema", DataType::Utf8, false),
            Field::new("sourceSha256", DataType::Utf8, false),
            Field::new("format", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new("markdown", DataType::Utf8, false),
        ])),
        vec![
            string_array([LEGACY_OFFICE_PROJECTION_SCHEMA]),
            string_array([source_digest]),
            string_array([projection.format.extension()]),
            string_array([projection.text.as_str()]),
            string_array([projection.markdown.as_str()]),
        ],
    )
    .map_err(|error| format!("build legacy Office Arrow projection batch: {error}"))
}

pub(super) fn legacy_office_projection_from_batches(
    batches: &[RecordBatch],
    expected_source_digest: &str,
    expected_format: LegacyOfficeFormat,
) -> Result<LegacyOfficeProjection, String> {
    let batch = batches
        .first()
        .ok_or_else(|| "legacy Office Arrow projection artifact is empty".to_string())?;
    if batch.num_rows() != 1 {
        return Err(format!(
            "legacy Office Arrow projection expected 1 row, found {}",
            batch.num_rows()
        ));
    }
    let schema = string_value(batch, "schema")?;
    if schema != LEGACY_OFFICE_PROJECTION_SCHEMA {
        return Err(format!(
            "legacy Office Arrow projection schema mismatch: expected `{LEGACY_OFFICE_PROJECTION_SCHEMA}`, found `{schema}`"
        ));
    }
    let source_digest = string_value(batch, "sourceSha256")?;
    if source_digest != expected_source_digest {
        return Err(format!(
            "legacy Office Arrow projection sourceSha256 mismatch: expected `{expected_source_digest}`, found `{source_digest}`"
        ));
    }
    let format = match string_value(batch, "format")?.as_str() {
        "doc" => LegacyOfficeFormat::Doc,
        "xls" => LegacyOfficeFormat::Xls,
        "ppt" => LegacyOfficeFormat::Ppt,
        value => {
            return Err(format!(
                "legacy Office Arrow projection has invalid format `{value}`"
            ));
        }
    };
    if format != expected_format {
        return Err(format!(
            "legacy Office Arrow projection format mismatch: expected `{}`, found `{}`",
            expected_format.extension(),
            format.extension()
        ));
    }
    let text = string_value(batch, "text")?;
    let markdown = string_value(batch, "markdown")?;
    if text.trim().is_empty() || markdown.trim().is_empty() {
        return Err("legacy Office Arrow projection contains empty text".to_string());
    }
    Ok(LegacyOfficeProjection {
        format,
        quality_metrics: legacy_office_quality_metrics(format, text.as_str(), markdown.as_str()),
        text,
        markdown,
    })
}

async fn write_legacy_office_projection_report(
    output: &Path,
    projection_read: &LegacyOfficeProjectionRead,
) -> Result<(), String> {
    let projection = &projection_read.projection;
    let report = json!({
        "schema": "xiuxian_wendao.legacy_office_projection_report.v1",
        "format": projection.format.extension(),
        "sourceSha256": projection_read.source_digest.as_str(),
        "parserProfile": LEGACY_OFFICE_PROJECTION_PROFILE,
        "textChars": projection.quality_metrics.text_char_count,
        "markdownChars": projection.quality_metrics.markdown_char_count,
        "lineCount": projection.quality_metrics.line_count,
        "nonEmptyLineCount": projection.quality_metrics.non_empty_line_count,
        "tabDelimitedRowCount": projection.quality_metrics.tab_delimited_row_count,
        "maxColumnCount": projection.quality_metrics.max_column_count,
        "markdownFencedBlockCount": projection.quality_metrics.markdown_fenced_block_count,
        "tabularBoundarySignal": projection
            .quality_metrics
            .has_tabular_boundary_signal(projection.format),
        "cacheBackend": projection_read.cache_backend.as_str(),
        "cacheStatus": projection_read.cache_status.as_str(),
        "cacheByteLen": projection_read.cache_byte_len,
        "precisionGatePassed": true,
    });
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("serialize legacy Office projection report: {error}"))?;
    tokio::fs::write(output.join(LEGACY_OFFICE_PROJECTION_REPORT_NAME), bytes)
        .await
        .map_err(|error| format!("write legacy Office projection report: {error}"))
}

fn string_value(batch: &RecordBatch, name: &str) -> Result<String, String> {
    let index = batch.schema().index_of(name).map_err(|error| {
        format!("legacy Office Arrow projection missing column `{name}`: {error}")
    })?;
    let column = batch
        .column(index)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("legacy Office Arrow projection column `{name}` is not Utf8"))?;
    if column.is_null(0) {
        return Err(format!(
            "legacy Office Arrow projection column `{name}` is null"
        ));
    }
    Ok(column.value(0).to_string())
}

fn string_array<'a>(values: impl IntoIterator<Item = &'a str>) -> arrow::array::ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as arrow::array::ArrayRef
}

fn source_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("read legacy Office source `{}`: {error}", path.display()))?;
    Ok(digest_component([bytes.as_slice()]))
}

fn digest_component<T>(fragments: impl IntoIterator<Item = T>) -> String
where
    T: AsRef<[u8]>,
{
    let mut hasher = sha2::Sha256::new();
    for fragment in fragments {
        hasher.update(fragment.as_ref());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn artifact_cache_backend_config_key(config: &ArtifactBlobCacheBackendConfig) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        config.kind().as_str(),
        config.root().display(),
        config.memory_capacity_bytes(),
        config.storage_capacity_bytes(),
        config.runtime_worker_threads(),
        config.memory_shards(),
        config.block_size_bytes(),
        config.recover_concurrency(),
        config.flushers(),
        config.reclaimers()
    )
}

fn artifact_cache_env_present() -> bool {
    [
        ARTIFACT_CACHE_BACKEND_ENV,
        ARTIFACT_CACHE_ROOT_ENV,
        ARTIFACT_CACHE_MEMORY_BYTES_ENV,
        ARTIFACT_CACHE_STORAGE_BYTES_ENV,
        ARTIFACT_CACHE_RUNTIME_WORKERS_ENV,
        ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
        ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV,
        ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV,
        ARTIFACT_CACHE_FLUSHERS_ENV,
        ARTIFACT_CACHE_RECLAIMERS_ENV,
        "PRJ_CACHE_HOME",
    ]
    .iter()
    .any(|key| std::env::var(key).is_ok_and(|value| !value.trim().is_empty()))
}
