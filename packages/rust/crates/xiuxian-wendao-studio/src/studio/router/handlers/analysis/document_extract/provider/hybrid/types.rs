use std::path::PathBuf;

#[cfg(test)]
use arrow::array::{Array, Int32Array};
use arrow::record_batch::RecordBatch as EngineRecordBatch;
use xiuxian_wendao_attachments::pdf::metrics::PdfOcrShardMetric;
use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, PdfOcrShardResult};
use xiuxian_wendao_attachments::pdf::render::PdfPageRegionRenderRequest;

pub(crate) const DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION";
pub(crate) const DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON";
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI";
pub(crate) const DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO";
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER";
#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE";
#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(crate) const DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE";

#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_SCAFFOLD_REGION_TABLE_JSON_MODE: &str = "region-table-json";
#[cfg(any(feature = "document-extract-pdf-render", test))]
const OCR2_REGION_PIPELINE_RENDER_DISPATCH_MODE: &str = "render-dispatch";

#[cfg(any(feature = "document-extract-pdf-render", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcr2ScaffoldMode {
    Disabled,
    RegionTableJson,
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HybridPdfOcr2RegionPipelineMode {
    Disabled,
    RenderDispatch,
}

#[cfg(any(feature = "document-extract-pdf-render", test))]
impl HybridPdfOcr2RegionPipelineMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::RenderDispatch => OCR2_REGION_PIPELINE_RENDER_DISPATCH_MODE,
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

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HybridPdfRegionInput {
    pub(super) source: PathBuf,
    pub(super) regions: Vec<PdfPageRegionRenderRequest>,
}

pub(crate) struct HybridDocumentResourceBatch {
    pub(crate) batch: EngineRecordBatch,
    pub(crate) ocr_inputs: Vec<PdfOcrShardInput>,
    pub(crate) ocr_results: Vec<PdfOcrShardResult>,
    pub(crate) ocr_metrics: Vec<PdfOcrShardMetric>,
    pub(crate) page_count: u32,
    pub(crate) text_page_indices: Vec<u32>,
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
        }
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
