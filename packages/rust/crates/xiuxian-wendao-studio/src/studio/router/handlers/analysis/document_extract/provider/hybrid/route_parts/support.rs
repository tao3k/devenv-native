use super::Instant;
use super::{
    BTreeMap, DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, FAILED_PAGE_RECOVERY_HOSTED_VLM_PAGE_MODE,
    HybridDocumentResourceBatch, HybridPdfFailedPageRecoveryMode, HybridPdfOcr2RegionPlanner,
    HybridPdfOcrProfilePlanner, Path, PdfOcrShardInput, PdfPageRenderShardReport,
    PdfRenderRoutingDecision, PdfRenderStatus, hybrid_page_ocr_profile_planner_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
use super::{
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD_ENV, PdfPageRegionRenderRequest,
    page_region_render_request_chunks_all, page_region_render_request_chunks_by_page,
    page_region_render_request_chunks_by_page_area_desc,
    page_region_render_request_chunks_by_page_max_area_desc,
    page_region_render_request_chunks_by_region,
    page_region_render_request_chunks_by_region_seed_page,
};
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::{
    HybridPdfOcr2RegionPipelineMode, hybrid_page_ocr2_region_pipeline_mode_with_lookup,
};
#[cfg(feature = "document-extract-pdf-render")]
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::{
    HybridPdfOcr2RegionRenderChunkMode, hybrid_page_ocr2_region_render_chunk_mode_with_lookup,
};

pub(super) fn record_phase_elapsed(
    phase_elapsed_ms: &mut BTreeMap<String, f64>,
    phase: &str,
    started: Instant,
) {
    phase_elapsed_ms.insert(phase.to_string(), started.elapsed().as_secs_f64() * 1000.0);
}

pub(super) fn record_ocr_scheduler_or_docling_fallback_phase(
    phase_elapsed_ms: &mut BTreeMap<String, f64>,
    resource_batch: &HybridDocumentResourceBatch,
    started: Instant,
) {
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
    if !resource_batch.page_range_docling_fallback_pages.is_empty() {
        phase_elapsed_ms.insert(
            "ocrScheduler".to_string(),
            resource_batch
                .ocr_metrics
                .iter()
                .filter_map(|metric| metric.rust_scheduler_elapsed_ms)
                .fold(0.0_f64, f64::max),
        );
        phase_elapsed_ms.insert("doclingPageRangeFallback".to_string(), elapsed_ms);
        return;
    }
    phase_elapsed_ms.insert("ocrScheduler".to_string(), elapsed_ms);
}

pub(super) fn direct_docling_structure_recovery_page_range_enabled() -> bool {
    direct_docling_structure_recovery_page_range_enabled_with_lookup(&|key| std::env::var(key).ok())
}

pub(super) fn direct_docling_structure_recovery_page_range_enabled_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    hybrid_page_ocr_profile_planner_with_lookup(lookup)
        == HybridPdfOcrProfilePlanner::DoclingStructureRecovery
        && lookup(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).is_none()
        && hybrid_page_ocr2_region_planner_with_lookup(lookup)
            == HybridPdfOcr2RegionPlanner::Disabled
        && hybrid_page_ocr2_region_pipeline_mode_with_lookup(lookup)
            != HybridPdfOcr2RegionPipelineMode::RenderDispatch
}

pub(super) fn direct_docling_structure_recovery_render_report(
    source: &Path,
    output: &Path,
    page_count: u32,
) -> PdfPageRenderShardReport {
    PdfPageRenderShardReport {
        source_path: source.to_string_lossy().to_string(),
        output_dir: output.to_string_lossy().to_string(),
        page_count,
        shard_count: 0,
        manifest_arrow_path: None,
        ocr_input_arrow_path: None,
        pending_resource_arrow_path: None,
        render_profile: "source-pdf-page-range-shards-v1".to_string(),
        render_selection: "docling-page-range-direct".to_string(),
        status: PdfRenderStatus::Rendered.as_str().to_string(),
        routing_decision: PdfRenderRoutingDecision::HybridPageOcrCandidate
            .as_str()
            .to_string(),
        elapsed_ms: 0.0,
        error_message: None,
        artifact_cache_backend: None,
        artifact_cache_hit_count: 0,
        artifact_cache_miss_count: 0,
        artifact_cache_throttled_count: 0,
        artifact_cache_byte_count: 0,
        artifact_cache_page_raster_hit_count: 0,
        artifact_cache_page_raster_miss_count: 0,
        artifact_cache_page_raster_throttled_count: 0,
        artifact_cache_page_raster_byte_count: 0,
        artifact_cache_region_crop_hit_count: 0,
        artifact_cache_region_crop_miss_count: 0,
        artifact_cache_region_crop_throttled_count: 0,
        artifact_cache_region_crop_byte_count: 0,
        artifact_cache_region_manifest_projection_hit_count: 0,
        artifact_cache_region_manifest_projection_miss_count: 0,
        artifact_cache_region_manifest_projection_throttled_count: 0,
        artifact_cache_region_manifest_projection_byte_count: 0,
        artifact_cache_region_manifest_projection_row_hit_count: 0,
        artifact_cache_region_manifest_projection_row_miss_count: 0,
        artifact_cache_region_manifest_projection_row_throttled_count: 0,
        artifact_cache_region_manifest_projection_row_byte_count: 0,
    }
}

pub(super) fn ocr2_region_pipeline_enabled() -> bool {
    #[cfg(feature = "document-extract-pdf-render")]
    {
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|key| std::env::var(key).ok())
            == HybridPdfOcr2RegionPipelineMode::RenderDispatch
    }
    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        false
    }
}

pub(super) fn ocr2_region_pipeline_mode_label() -> &'static str {
    #[cfg(feature = "document-extract-pdf-render")]
    {
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|key| std::env::var(key).ok()).as_str()
    }
    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        "disabled"
    }
}

pub(super) fn ocr2_region_render_chunk_mode_label() -> &'static str {
    #[cfg(feature = "document-extract-pdf-render")]
    {
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| std::env::var(key).ok())
            .as_str()
    }
    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        "page"
    }
}

pub(super) fn failed_page_recovery_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfFailedPageRecoveryMode {
    match lookup(DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        FAILED_PAGE_RECOVERY_HOSTED_VLM_PAGE_MODE => HybridPdfFailedPageRecoveryMode::HostedVlmPage,
        _ => HybridPdfFailedPageRecoveryMode::Disabled,
    }
}

pub(super) fn failed_page_recovery_mode() -> HybridPdfFailedPageRecoveryMode {
    failed_page_recovery_mode_with_lookup(&|key| std::env::var(key).ok())
}

pub(super) fn failed_page_recovery_mode_label() -> &'static str {
    failed_page_recovery_mode().as_str()
}

#[cfg(feature = "document-extract-pdf-render")]
pub(super) fn ocr2_region_render_request_chunks_with_lookup(
    regions: &[PdfPageRegionRenderRequest],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Vec<Vec<PdfPageRegionRenderRequest>> {
    match hybrid_page_ocr2_region_render_chunk_mode_with_lookup(lookup) {
        HybridPdfOcr2RegionRenderChunkMode::All => page_region_render_request_chunks_all(regions),
        HybridPdfOcr2RegionRenderChunkMode::PageAreaDesc => {
            page_region_render_request_chunks_by_page_area_desc(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::PageMaxAreaDesc => {
            page_region_render_request_chunks_by_page_max_area_desc(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::Region => {
            page_region_render_request_chunks_by_region(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::RegionSeedPage => {
            page_region_render_request_chunks_by_region_seed_page(regions)
        }
        HybridPdfOcr2RegionRenderChunkMode::Page => {
            page_region_render_request_chunks_by_page(regions)
        }
    }
}

#[cfg(feature = "document-extract-pdf-render")]
pub(super) fn ocr2_region_render_ahead_limit_for_capacity_with_lookup(
    chunk_count: usize,
    endpoint_count: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> usize {
    let endpoint_window = endpoint_count.saturating_sub(1).max(1);
    let requested = lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD_ENV)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(endpoint_window);
    requested.clamp(1, chunk_count.max(1).min(endpoint_window))
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Ocr2RegionMaterializationStats {
    pub(super) requested_region_count: usize,
    pub(super) rendered_region_count: usize,
    pub(super) render_cache_hit_count: usize,
    pub(super) render_cache_miss_count: usize,
    pub(super) render_artifact_cache_hit_count: u64,
    pub(super) render_artifact_cache_miss_count: u64,
    pub(super) render_artifact_cache_throttled_count: u64,
    pub(super) render_artifact_cache_byte_count: u64,
    pub(super) render_artifact_cache_page_raster_hit_count: u64,
    pub(super) render_artifact_cache_page_raster_miss_count: u64,
    pub(super) render_artifact_cache_page_raster_throttled_count: u64,
    pub(super) render_artifact_cache_page_raster_byte_count: u64,
    pub(super) render_artifact_cache_region_crop_hit_count: u64,
    pub(super) render_artifact_cache_region_crop_miss_count: u64,
    pub(super) render_artifact_cache_region_crop_throttled_count: u64,
    pub(super) render_artifact_cache_region_crop_byte_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_hit_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_miss_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_throttled_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_byte_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_row_hit_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_row_miss_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_row_throttled_count: u64,
    pub(super) render_artifact_cache_region_manifest_projection_row_byte_count: u64,
    pub(super) render_reported_elapsed_ms: f64,
    pub(super) pipeline_planned_render_chunk_count: usize,
    pub(super) pipeline_endpoint_count: usize,
    pub(super) pipeline_render_ahead_limit: usize,
    pub(super) pipeline_render_spawn_count: usize,
    pub(super) pipeline_render_chunk_count: usize,
    pub(super) pipeline_region_dispatch_count: usize,
    pub(super) pipeline_base_result_count: usize,
    pub(super) pipeline_base_result_shard_count: usize,
    pub(super) pipeline_region_result_count: usize,
    pub(super) pipeline_region_result_shard_count: usize,
}

impl Ocr2RegionMaterializationStats {
    pub(super) fn record_render_artifact_cache_report(
        &mut self,
        report: &PdfPageRenderShardReport,
    ) {
        self.render_artifact_cache_hit_count = self
            .render_artifact_cache_hit_count
            .saturating_add(report.artifact_cache_hit_count);
        self.render_artifact_cache_miss_count = self
            .render_artifact_cache_miss_count
            .saturating_add(report.artifact_cache_miss_count);
        self.render_artifact_cache_throttled_count = self
            .render_artifact_cache_throttled_count
            .saturating_add(report.artifact_cache_throttled_count);
        self.render_artifact_cache_byte_count = self
            .render_artifact_cache_byte_count
            .saturating_add(report.artifact_cache_byte_count);
        self.render_artifact_cache_page_raster_hit_count = self
            .render_artifact_cache_page_raster_hit_count
            .saturating_add(report.artifact_cache_page_raster_hit_count);
        self.render_artifact_cache_page_raster_miss_count = self
            .render_artifact_cache_page_raster_miss_count
            .saturating_add(report.artifact_cache_page_raster_miss_count);
        self.render_artifact_cache_page_raster_throttled_count = self
            .render_artifact_cache_page_raster_throttled_count
            .saturating_add(report.artifact_cache_page_raster_throttled_count);
        self.render_artifact_cache_page_raster_byte_count = self
            .render_artifact_cache_page_raster_byte_count
            .saturating_add(report.artifact_cache_page_raster_byte_count);
        self.render_artifact_cache_region_crop_hit_count = self
            .render_artifact_cache_region_crop_hit_count
            .saturating_add(report.artifact_cache_region_crop_hit_count);
        self.render_artifact_cache_region_crop_miss_count = self
            .render_artifact_cache_region_crop_miss_count
            .saturating_add(report.artifact_cache_region_crop_miss_count);
        self.render_artifact_cache_region_crop_throttled_count = self
            .render_artifact_cache_region_crop_throttled_count
            .saturating_add(report.artifact_cache_region_crop_throttled_count);
        self.render_artifact_cache_region_crop_byte_count = self
            .render_artifact_cache_region_crop_byte_count
            .saturating_add(report.artifact_cache_region_crop_byte_count);
        self.render_artifact_cache_region_manifest_projection_hit_count = self
            .render_artifact_cache_region_manifest_projection_hit_count
            .saturating_add(report.artifact_cache_region_manifest_projection_hit_count);
        self.render_artifact_cache_region_manifest_projection_miss_count = self
            .render_artifact_cache_region_manifest_projection_miss_count
            .saturating_add(report.artifact_cache_region_manifest_projection_miss_count);
        self.render_artifact_cache_region_manifest_projection_throttled_count = self
            .render_artifact_cache_region_manifest_projection_throttled_count
            .saturating_add(report.artifact_cache_region_manifest_projection_throttled_count);
        self.render_artifact_cache_region_manifest_projection_byte_count = self
            .render_artifact_cache_region_manifest_projection_byte_count
            .saturating_add(report.artifact_cache_region_manifest_projection_byte_count);
        self.render_artifact_cache_region_manifest_projection_row_hit_count = self
            .render_artifact_cache_region_manifest_projection_row_hit_count
            .saturating_add(report.artifact_cache_region_manifest_projection_row_hit_count);
        self.render_artifact_cache_region_manifest_projection_row_miss_count = self
            .render_artifact_cache_region_manifest_projection_row_miss_count
            .saturating_add(report.artifact_cache_region_manifest_projection_row_miss_count);
        self.render_artifact_cache_region_manifest_projection_row_throttled_count = self
            .render_artifact_cache_region_manifest_projection_row_throttled_count
            .saturating_add(report.artifact_cache_region_manifest_projection_row_throttled_count);
        self.render_artifact_cache_region_manifest_projection_row_byte_count = self
            .render_artifact_cache_region_manifest_projection_row_byte_count
            .saturating_add(report.artifact_cache_region_manifest_projection_row_byte_count);
    }
}

#[derive(Debug)]
pub(super) struct Ocr2RegionMaterialization {
    pub(super) inputs: Vec<PdfOcrShardInput>,
    pub(super) stats: Ocr2RegionMaterializationStats,
    pub(super) phase_elapsed_ms: BTreeMap<String, f64>,
}

impl Ocr2RegionMaterialization {
    #[cfg(feature = "document-extract-pdf-render")]
    pub(super) fn new(inputs: Vec<PdfOcrShardInput>) -> Self {
        Self {
            inputs,
            stats: Ocr2RegionMaterializationStats::default(),
            phase_elapsed_ms: BTreeMap::new(),
        }
    }

    #[cfg(feature = "document-extract-pdf-render")]
    pub(super) fn record_phase_elapsed(&mut self, phase: &str, started: Instant) {
        record_phase_elapsed(&mut self.phase_elapsed_ms, phase, started);
    }
}
