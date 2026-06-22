//! Render operation report assembly and persisted summaries.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::identity::PDF_RENDER_SHARD_PROFILE;
use super::types::{
    PdfPageRenderProfile, PdfPageRenderSelection, PdfPageRenderShardReport,
    PdfRenderRoutingDecision, PdfRenderStatus,
};

/// # Errors
///
/// Returns an error if reports cannot be written.
pub fn write_page_render_shard_reports(
    report_dir: &Path,
    records: &[PdfPageRenderShardReport],
) -> Result<(), String> {
    fs::create_dir_all(report_dir)
        .map_err(|error| format!("create report dir `{}`: {error}", report_dir.display()))?;
    let json_path = report_dir.join("pdf_page_render_shard_manifest.json");
    let report = serde_json::json!({
        "schema": "xiuxian_wendao.pdf_page_render_shard_manifest.v1",
        "profile": PDF_RENDER_SHARD_PROFILE,
        "totalInputs": records.len(),
        "totalRenderedShards": records.iter().map(|record| record.shard_count).sum::<u32>(),
        "renderedInputs": records.iter().filter(|record| record.status == "rendered").count(),
        "fallbackInputs": records.iter().filter(|record| record.status == "fallback").count(),
        "artifactCacheBackendCounts": artifact_cache_backend_counts(records),
        "artifactCacheHitCount": records.iter().map(|record| record.artifact_cache_hit_count).sum::<u64>(),
        "artifactCacheMissCount": records.iter().map(|record| record.artifact_cache_miss_count).sum::<u64>(),
        "artifactCacheThrottledCount": records.iter().map(|record| record.artifact_cache_throttled_count).sum::<u64>(),
        "artifactCacheByteCount": records.iter().map(|record| record.artifact_cache_byte_count).sum::<u64>(),
        "artifactCachePageRasterHitCount": records.iter().map(|record| record.artifact_cache_page_raster_hit_count).sum::<u64>(),
        "artifactCachePageRasterMissCount": records.iter().map(|record| record.artifact_cache_page_raster_miss_count).sum::<u64>(),
        "artifactCachePageRasterThrottledCount": records.iter().map(|record| record.artifact_cache_page_raster_throttled_count).sum::<u64>(),
        "artifactCachePageRasterByteCount": records.iter().map(|record| record.artifact_cache_page_raster_byte_count).sum::<u64>(),
        "artifactCacheRegionCropHitCount": records.iter().map(|record| record.artifact_cache_region_crop_hit_count).sum::<u64>(),
        "artifactCacheRegionCropMissCount": records.iter().map(|record| record.artifact_cache_region_crop_miss_count).sum::<u64>(),
        "artifactCacheRegionCropThrottledCount": records.iter().map(|record| record.artifact_cache_region_crop_throttled_count).sum::<u64>(),
        "artifactCacheRegionCropByteCount": records.iter().map(|record| record.artifact_cache_region_crop_byte_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionHitCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_hit_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionMissCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_miss_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionThrottledCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_throttled_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionByteCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_byte_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionRowHitCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_row_hit_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionRowMissCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_row_miss_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionRowThrottledCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_row_throttled_count).sum::<u64>(),
        "artifactCacheRegionManifestProjectionRowByteCount": records.iter().map(|record| record.artifact_cache_region_manifest_projection_row_byte_count).sum::<u64>(),
        "records": records,
    });
    fs::write(
        json_path.as_path(),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("write report `{}`: {error}", json_path.display()))?;

    let markdown_path = report_dir.join("pdf_page_render_shard_manifest.md");
    fs::write(markdown_path.as_path(), render_markdown_report(records))
        .map_err(|error| format!("write report `{}`: {error}", markdown_path.display()))?;
    Ok(())
}
pub(super) struct RenderShardContext<'a> {
    pub(super) path: &'a Path,
    output_dir: &'a Path,
    pub(super) profile: &'a PdfPageRenderProfile,
    selection: PdfPageRenderSelection,
    source_path: String,
    started: Instant,
}

impl<'a> RenderShardContext<'a> {
    pub(super) fn new(
        path: &'a Path,
        output_dir: &'a Path,
        profile: &'a PdfPageRenderProfile,
        selection: PdfPageRenderSelection,
    ) -> Self {
        Self {
            path,
            output_dir,
            profile,
            selection,
            source_path: path.to_string_lossy().to_string(),
            started: Instant::now(),
        }
    }

    pub(super) fn report(&self, parts: ReportParts) -> PdfPageRenderShardReport {
        let artifact_cache = parts.artifact_cache.as_deref();
        PdfPageRenderShardReport {
            source_path: self.source_path.clone(),
            output_dir: self.output_dir.to_string_lossy().to_string(),
            page_count: parts.page_count,
            shard_count: parts.shard_count,
            manifest_arrow_path: parts
                .manifest_arrow_path
                .map(|path| path.to_string_lossy().to_string()),
            ocr_input_arrow_path: parts
                .ocr_input_arrow_path
                .map(|path| path.to_string_lossy().to_string()),
            pending_resource_arrow_path: parts
                .pending_resource_arrow_path
                .map(|path| path.to_string_lossy().to_string()),
            render_profile: self.profile.profile_id.clone(),
            render_selection: self.selection.as_str().to_string(),
            status: parts.status.as_str().to_string(),
            routing_decision: parts.routing_decision.as_str().to_string(),
            elapsed_ms: self.started.elapsed().as_secs_f64() * 1000.0,
            error_message: parts.error_message,
            artifact_cache_backend: artifact_cache.map(|stats| stats.backend.clone()),
            artifact_cache_hit_count: artifact_cache.map_or(0, |stats| stats.hit_count),
            artifact_cache_miss_count: artifact_cache.map_or(0, |stats| stats.miss_count),
            artifact_cache_throttled_count: artifact_cache.map_or(0, |stats| stats.throttled_count),
            artifact_cache_byte_count: artifact_cache.map_or(0, |stats| stats.byte_count),
            artifact_cache_page_raster_hit_count: artifact_cache
                .map_or(0, |stats| stats.page_raster_hit_count),
            artifact_cache_page_raster_miss_count: artifact_cache
                .map_or(0, |stats| stats.page_raster_miss_count),
            artifact_cache_page_raster_throttled_count: artifact_cache
                .map_or(0, |stats| stats.page_raster_throttled_count),
            artifact_cache_page_raster_byte_count: artifact_cache
                .map_or(0, |stats| stats.page_raster_byte_count),
            artifact_cache_region_crop_hit_count: artifact_cache
                .map_or(0, |stats| stats.region_crop_hit_count),
            artifact_cache_region_crop_miss_count: artifact_cache
                .map_or(0, |stats| stats.region_crop_miss_count),
            artifact_cache_region_crop_throttled_count: artifact_cache
                .map_or(0, |stats| stats.region_crop_throttled_count),
            artifact_cache_region_crop_byte_count: artifact_cache
                .map_or(0, |stats| stats.region_crop_byte_count),
            artifact_cache_region_manifest_projection_hit_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_hit_count),
            artifact_cache_region_manifest_projection_miss_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_miss_count),
            artifact_cache_region_manifest_projection_throttled_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_throttled_count),
            artifact_cache_region_manifest_projection_byte_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_byte_count),
            artifact_cache_region_manifest_projection_row_hit_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_row_hit_count),
            artifact_cache_region_manifest_projection_row_miss_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_row_miss_count),
            artifact_cache_region_manifest_projection_row_throttled_count: artifact_cache
                .map_or(0, |stats| {
                    stats.region_manifest_projection_row_throttled_count
                }),
            artifact_cache_region_manifest_projection_row_byte_count: artifact_cache
                .map_or(0, |stats| stats.region_manifest_projection_row_byte_count),
        }
    }

    pub(super) fn shard_dir(&self, source_hash: &str) -> PathBuf {
        self.output_dir.join("ocr-shards").join(source_hash)
    }
}

pub(super) struct ReportParts {
    page_count: u32,
    shard_count: u32,
    manifest_arrow_path: Option<PathBuf>,
    ocr_input_arrow_path: Option<PathBuf>,
    pending_resource_arrow_path: Option<PathBuf>,
    status: PdfRenderStatus,
    routing_decision: PdfRenderRoutingDecision,
    error_message: Option<String>,
    artifact_cache: Option<Box<ArtifactCacheReportStats>>,
}

struct ArtifactCacheReportStats {
    backend: String,
    hit_count: u64,
    miss_count: u64,
    throttled_count: u64,
    byte_count: u64,
    page_raster_hit_count: u64,
    page_raster_miss_count: u64,
    page_raster_throttled_count: u64,
    page_raster_byte_count: u64,
    region_crop_hit_count: u64,
    region_crop_miss_count: u64,
    region_crop_throttled_count: u64,
    region_crop_byte_count: u64,
    region_manifest_projection_hit_count: u64,
    region_manifest_projection_miss_count: u64,
    region_manifest_projection_throttled_count: u64,
    region_manifest_projection_byte_count: u64,
    region_manifest_projection_row_hit_count: u64,
    region_manifest_projection_row_miss_count: u64,
    region_manifest_projection_row_throttled_count: u64,
    region_manifest_projection_row_byte_count: u64,
}

impl ReportParts {
    #[cfg(feature = "pdf-render")]
    pub(super) fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub(super) fn unsupported(error_message: &str) -> Self {
        Self {
            page_count: 0,
            shard_count: 0,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Unsupported,
            routing_decision: PdfRenderRoutingDecision::UnsupportedNonPdf,
            error_message: Some(error_message.to_string()),
            artifact_cache: None,
        }
    }

    pub(super) fn fallback(page_count: u32, shard_count: u32, error_message: String) -> Self {
        Self {
            page_count,
            shard_count,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Fallback,
            routing_decision: PdfRenderRoutingDecision::FullDoclingFallback,
            error_message: Some(error_message),
            artifact_cache: None,
        }
    }

    pub(super) fn preflight_failed(error_message: String) -> Self {
        Self {
            page_count: 0,
            shard_count: 0,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Fallback,
            routing_decision: PdfRenderRoutingDecision::PreflightFailed,
            error_message: Some(error_message),
            artifact_cache: None,
        }
    }

    pub(super) fn skipped(
        page_count: u32,
        routing_decision: PdfRenderRoutingDecision,
        error_message: String,
    ) -> Self {
        Self {
            page_count,
            shard_count: 0,
            manifest_arrow_path: None,
            ocr_input_arrow_path: None,
            pending_resource_arrow_path: None,
            status: PdfRenderStatus::Skipped,
            routing_decision,
            error_message: Some(error_message),
            artifact_cache: None,
        }
    }

    pub(super) fn rendered(
        page_count: u32,
        shard_count: u32,
        manifest_arrow_path: PathBuf,
        ocr_input_arrow_path: PathBuf,
        pending_resource_arrow_path: PathBuf,
    ) -> Self {
        Self {
            page_count,
            shard_count,
            manifest_arrow_path: Some(manifest_arrow_path),
            ocr_input_arrow_path: Some(ocr_input_arrow_path),
            pending_resource_arrow_path: Some(pending_resource_arrow_path),
            status: PdfRenderStatus::Rendered,
            routing_decision: PdfRenderRoutingDecision::HybridPageOcrCandidate,
            error_message: None,
            artifact_cache: None,
        }
    }

    #[cfg(feature = "pdf-render")]
    pub(super) fn with_artifact_cache_stats(
        mut self,
        stats: Option<super::artifact_cache::PdfRenderArtifactCacheStats>,
    ) -> Self {
        self.artifact_cache = stats.and_then(|stats| {
            let backend = stats.backend_name?;
            Some(Box::new(ArtifactCacheReportStats {
                backend,
                hit_count: stats.hit_count,
                miss_count: stats.miss_count,
                throttled_count: stats.throttled_count,
                byte_count: stats.byte_count,
                page_raster_hit_count: stats.page_raster_hit_count,
                page_raster_miss_count: stats.page_raster_miss_count,
                page_raster_throttled_count: stats.page_raster_throttled_count,
                page_raster_byte_count: stats.page_raster_byte_count,
                region_crop_hit_count: stats.region_crop_hit_count,
                region_crop_miss_count: stats.region_crop_miss_count,
                region_crop_throttled_count: stats.region_crop_throttled_count,
                region_crop_byte_count: stats.region_crop_byte_count,
                region_manifest_projection_hit_count: stats.region_manifest_projection_hit_count,
                region_manifest_projection_miss_count: stats.region_manifest_projection_miss_count,
                region_manifest_projection_throttled_count: stats
                    .region_manifest_projection_throttled_count,
                region_manifest_projection_byte_count: stats.region_manifest_projection_byte_count,
                region_manifest_projection_row_hit_count: stats
                    .region_manifest_projection_row_hit_count,
                region_manifest_projection_row_miss_count: stats
                    .region_manifest_projection_row_miss_count,
                region_manifest_projection_row_throttled_count: stats
                    .region_manifest_projection_row_throttled_count,
                region_manifest_projection_row_byte_count: stats
                    .region_manifest_projection_row_byte_count,
            }))
        });
        self
    }

    #[cfg(feature = "pdf-render")]
    pub(super) fn with_artifact_cache_stats_from(
        self,
        stats: Option<super::artifact_cache::PdfRenderArtifactCacheStats>,
    ) -> Self {
        let Some(stats) = stats else {
            return self;
        };
        self.with_artifact_cache_stats(Some(stats))
    }
}

fn artifact_cache_backend_counts(records: &[PdfPageRenderShardReport]) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for backend in records
        .iter()
        .filter_map(|record| record.artifact_cache_backend.as_deref())
    {
        *counts.entry(backend.to_string()).or_insert(0) += 1;
    }
    counts
}

fn render_markdown_report(records: &[PdfPageRenderShardReport]) -> String {
    let mut markdown = String::new();
    markdown.push_str("# PDF Page Render Shard Manifest Report\n\n");
    markdown.push_str(
        "| Source | Status | Decision | Pages | Shards | Artifact cache | Elapsed ms | Error |\n",
    );
    markdown.push_str(
        "| ------ | ------ | -------- | ----: | -----: | -------------- | ---------: | ----- |\n",
    );
    for record in records {
        let _ = writeln!(
            markdown,
            "| `{}` | `{}` | `{}` | {} | {} | {} | {:.3} | {} |",
            record.source_path,
            record.status,
            record.routing_decision,
            record.page_count,
            record.shard_count,
            artifact_cache_markdown_cell(record),
            record.elapsed_ms,
            record.error_message.as_deref().unwrap_or("")
        );
    }
    markdown
}

fn artifact_cache_markdown_cell(record: &PdfPageRenderShardReport) -> String {
    record
        .artifact_cache_backend
        .as_ref()
        .map_or_else(String::new, |backend| {
            format!(
                "`{backend}` H:{} M:{} T:{} B:{} | page:{}:{} crop:{}:{} proj:{}:{} row:{}:{}",
                record.artifact_cache_hit_count,
                record.artifact_cache_miss_count,
                record.artifact_cache_throttled_count,
                record.artifact_cache_byte_count,
                record.artifact_cache_page_raster_hit_count,
                record.artifact_cache_page_raster_miss_count,
                record.artifact_cache_region_crop_hit_count,
                record.artifact_cache_region_crop_miss_count,
                record.artifact_cache_region_manifest_projection_hit_count,
                record.artifact_cache_region_manifest_projection_miss_count,
                record.artifact_cache_region_manifest_projection_row_hit_count,
                record.artifact_cache_region_manifest_projection_row_miss_count
            )
        })
}
