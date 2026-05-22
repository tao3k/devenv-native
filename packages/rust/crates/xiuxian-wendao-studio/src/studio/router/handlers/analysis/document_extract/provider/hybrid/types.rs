use std::collections::BTreeMap;
#[cfg(feature = "document-extract-pdf-render")]
use std::path::PathBuf;

#[cfg(test)]
use arrow::array::{Array, Int32Array};
use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric;
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::PdfPageRegionRenderRequest;

pub(crate) const DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION";
pub(crate) const DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON";
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI";
#[cfg(feature = "document-extract-pdf-render")]
pub(crate) const DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO";
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER";
#[cfg(feature = "document-extract-pdf-render")]
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS";
#[cfg(feature = "document-extract-pdf-render")]
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES";
#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE";
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE";
#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK";

#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_SCAFFOLD_REGION_TABLE_JSON_MODE: &str = "region-table-json";
const OCR2_REGION_PIPELINE_RENDER_DISPATCH_MODE: &str = "render-dispatch";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_RENDER_CHUNK_ALL_MODE: &str = "all";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_RENDER_CHUNK_REGION_MODE: &str = "region";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_RENDER_CHUNK_REGION_SEED_PAGE_MODE: &str = "region-seed-page";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_RENDER_CHUNK_PAGE_AREA_DESC_MODE: &str = "page-area-desc";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_RENDER_CHUNK_PAGE_MAX_AREA_DESC_MODE: &str = "page-max-area-desc";

#[cfg(any(feature = "document-extract-pdf-render", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcr2ScaffoldMode {
    Disabled,
    RegionTableJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcr2RegionPipelineMode {
    Disabled,
    RenderDispatch,
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcr2RegionRenderChunkMode {
    Page,
    All,
    PageAreaDesc,
    PageMaxAreaDesc,
    Region,
    RegionSeedPage,
}

#[cfg(feature = "document-extract-pdf-render")]
impl HybridPdfOcr2RegionPipelineMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::RenderDispatch => OCR2_REGION_PIPELINE_RENDER_DISPATCH_MODE,
        }
    }
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
impl HybridPdfOcr2RegionRenderChunkMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::All => OCR2_REGION_RENDER_CHUNK_ALL_MODE,
            Self::PageAreaDesc => OCR2_REGION_RENDER_CHUNK_PAGE_AREA_DESC_MODE,
            Self::PageMaxAreaDesc => OCR2_REGION_RENDER_CHUNK_PAGE_MAX_AREA_DESC_MODE,
            Self::Region => OCR2_REGION_RENDER_CHUNK_REGION_MODE,
            Self::RegionSeedPage => OCR2_REGION_RENDER_CHUNK_REGION_SEED_PAGE_MODE,
        }
    }
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) fn hybrid_page_ocr2_scaffold_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfOcr2ScaffoldMode {
    match lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        OCR2_SCAFFOLD_REGION_TABLE_JSON_MODE => HybridPdfOcr2ScaffoldMode::RegionTableJson,
        _ => HybridPdfOcr2ScaffoldMode::Disabled,
    }
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) fn hybrid_page_ocr2_region_render_chunk_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfOcr2RegionRenderChunkMode {
    match lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        OCR2_REGION_RENDER_CHUNK_ALL_MODE => HybridPdfOcr2RegionRenderChunkMode::All,
        OCR2_REGION_RENDER_CHUNK_REGION_MODE => HybridPdfOcr2RegionRenderChunkMode::Region,
        OCR2_REGION_RENDER_CHUNK_REGION_SEED_PAGE_MODE => {
            HybridPdfOcr2RegionRenderChunkMode::RegionSeedPage
        }
        OCR2_REGION_RENDER_CHUNK_PAGE_AREA_DESC_MODE => {
            HybridPdfOcr2RegionRenderChunkMode::PageAreaDesc
        }
        OCR2_REGION_RENDER_CHUNK_PAGE_MAX_AREA_DESC_MODE => {
            HybridPdfOcr2RegionRenderChunkMode::PageMaxAreaDesc
        }
        _ => HybridPdfOcr2RegionRenderChunkMode::Page,
    }
}

pub(crate) fn hybrid_page_ocr2_region_pipeline_mode_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> HybridPdfOcr2RegionPipelineMode {
    match lookup(DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE_ENV)
        .unwrap_or_default()
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .as_str()
    {
        OCR2_REGION_PIPELINE_RENDER_DISPATCH_MODE => {
            HybridPdfOcr2RegionPipelineMode::RenderDispatch
        }
        _ => HybridPdfOcr2RegionPipelineMode::Disabled,
    }
}

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/hybrid/types.rs"]
mod tests;

#[cfg(feature = "document-extract-pdf-render")]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HybridPdfRegionInput {
    pub(super) source: PathBuf,
    pub(super) regions: Vec<PdfPageRegionRenderRequest>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageRangeDoclingFallbackSourceProfileSummary {
    pub(crate) page_count: usize,
    pub(crate) estimated_weight_total: u64,
    pub(crate) estimated_weight_max: u32,
    pub(crate) estimated_structure_cost_total: u64,
    pub(crate) estimated_structure_cost_max: u32,
    pub(crate) content_bytes_total: u64,
    pub(crate) operation_count_total: u64,
    pub(crate) text_show_ops_total: u64,
    pub(crate) path_ops_total: u64,
    pub(crate) rectangle_ops_total: u64,
    pub(crate) draw_object_ops_total: u64,
    pub(crate) structure_authority_required_count: usize,
    pub(crate) fast_profile_risk_count: usize,
    pub(crate) backend_text_topup_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageRangeDoclingFallbackChunkTiming {
    pub(crate) page_start: u32,
    pub(crate) page_end: u32,
    pub(crate) one_based_start: u32,
    pub(crate) one_based_end: u32,
    pub(crate) elapsed_ms: f64,
    pub(crate) resource_rows: usize,
    pub(crate) document_extract_profile: String,
    pub(crate) hedged: bool,
    pub(crate) attempt_count: usize,
    pub(crate) hedge_delay_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) document_timing_total_elapsed_ms: Option<f64>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) document_timing_phase_elapsed_ms: BTreeMap<String, f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source_profile: Option<PageRangeDoclingFallbackSourceProfileSummary>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageRangeDoclingFallbackPlanRange {
    pub(crate) page_start: u32,
    pub(crate) page_end: u32,
    pub(crate) one_based_start: u32,
    pub(crate) one_based_end: u32,
    pub(crate) estimated_structure_cost_total: u64,
    pub(crate) estimated_structure_cost_max: u32,
    pub(crate) structure_authority_required_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageRangeDoclingFallbackPlanSummary {
    pub(crate) strategy: &'static str,
    pub(crate) target_chunk_count: usize,
    pub(crate) fallback_page_count: usize,
    pub(crate) range_count: usize,
    pub(crate) chunk_size: Option<u32>,
    pub(crate) source_profile_used: bool,
    pub(crate) estimated_structure_cost_total: u64,
    pub(crate) estimated_structure_cost_max: u32,
    pub(crate) structure_authority_required_count: usize,
    pub(crate) ranges: Vec<PageRangeDoclingFallbackPlanRange>,
}

pub(crate) struct HybridDocumentResourceBatch {
    pub(crate) batch: EngineRecordBatch,
    pub(crate) ocr_inputs: Vec<PdfOcrShardInput>,
    pub(crate) ocr_results: Vec<PdfOcrShardResult>,
    pub(crate) ocr_metrics: Vec<PdfOcrShardMetric>,
    pub(crate) page_count: u32,
    pub(crate) text_page_indices: Vec<u32>,
    pub(crate) page_range_docling_fallback_pages: Vec<u32>,
    pub(crate) page_range_docling_fallback_chunks: Vec<PageRangeDoclingFallbackChunkTiming>,
    pub(crate) page_range_docling_fallback_plan: Option<PageRangeDoclingFallbackPlanSummary>,
}

impl HybridDocumentResourceBatch {
    pub(crate) fn new(
        batch: EngineRecordBatch,
        ocr_inputs: Vec<PdfOcrShardInput>,
        ocr_results: Vec<PdfOcrShardResult>,
        ocr_metrics: Vec<PdfOcrShardMetric>,
        page_count: u32,
        text_page_indices: Vec<u32>,
    ) -> Self {
        Self {
            batch,
            ocr_inputs,
            ocr_results,
            ocr_metrics,
            page_count,
            text_page_indices,
            page_range_docling_fallback_pages: Vec::new(),
            page_range_docling_fallback_chunks: Vec::new(),
            page_range_docling_fallback_plan: None,
        }
    }

    pub(crate) fn with_page_range_docling_fallback_pages(mut self, pages: Vec<u32>) -> Self {
        self.page_range_docling_fallback_pages = pages;
        self
    }

    pub(crate) fn with_page_range_docling_fallback_chunks(
        mut self,
        chunks: Vec<PageRangeDoclingFallbackChunkTiming>,
    ) -> Self {
        self.page_range_docling_fallback_chunks = chunks;
        self
    }

    pub(crate) fn with_page_range_docling_fallback_plan(
        mut self,
        plan: PageRangeDoclingFallbackPlanSummary,
    ) -> Self {
        self.page_range_docling_fallback_plan = Some(plan);
        self
    }

    #[cfg(test)]
    pub(crate) fn native(batch: EngineRecordBatch) -> Self {
        let text_page_indices = resource_page_indices(&batch);
        let page_count = inferred_page_count(text_page_indices.as_slice());
        Self {
            batch,
            ocr_inputs: Vec::new(),
            ocr_results: Vec::new(),
            ocr_metrics: Vec::new(),
            page_count,
            text_page_indices,
            page_range_docling_fallback_pages: Vec::new(),
            page_range_docling_fallback_chunks: Vec::new(),
            page_range_docling_fallback_plan: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_ocr(
        batch: EngineRecordBatch,
        ocr_inputs: Vec<PdfOcrShardInput>,
        ocr_results: Vec<PdfOcrShardResult>,
    ) -> Self {
        let page_count = ocr_inputs
            .iter()
            .map(|input| input.page_index)
            .chain(ocr_results.iter().map(|result| result.page_index))
            .max()
            .map_or(0, |page_index| page_index.saturating_add(1));
        Self {
            batch,
            ocr_inputs,
            ocr_results,
            ocr_metrics: Vec::new(),
            page_count,
            text_page_indices: Vec::new(),
            page_range_docling_fallback_pages: Vec::new(),
            page_range_docling_fallback_chunks: Vec::new(),
            page_range_docling_fallback_plan: None,
        }
    }
}

#[cfg(test)]
fn resource_page_indices(batch: &EngineRecordBatch) -> Vec<u32> {
    batch
        .column_by_name("pageIndex")
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .map(|column| {
            (0..column.len())
                .filter_map(|row| {
                    (!column.is_null(row))
                        .then(|| column.value(row))
                        .and_then(|value| u32::try_from(value).ok())
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
fn inferred_page_count(page_indices: &[u32]) -> u32 {
    page_indices
        .iter()
        .copied()
        .max()
        .map_or(0, |page_index| page_index.saturating_add(1))
}
