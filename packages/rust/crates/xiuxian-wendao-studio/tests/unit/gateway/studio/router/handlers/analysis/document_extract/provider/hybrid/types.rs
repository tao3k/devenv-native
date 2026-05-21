use super::{
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV, HybridPdfOcr2RegionPipelineMode,
    HybridPdfOcr2RegionRenderChunkMode, HybridPdfOcr2ScaffoldMode,
    hybrid_page_ocr2_region_pipeline_mode_with_lookup,
    hybrid_page_ocr2_region_render_chunk_mode_with_lookup,
    hybrid_page_ocr2_scaffold_mode_with_lookup,
};

#[test]
fn scaffold_mode_accepts_region_table_json() {
    assert_eq!(
        hybrid_page_ocr2_scaffold_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV)
                .then(|| "region_table_json".to_string())
        }),
        HybridPdfOcr2ScaffoldMode::RegionTableJson
    );
}

#[test]
fn scaffold_mode_defaults_unknown_values_to_disabled() {
    assert_eq!(
        hybrid_page_ocr2_scaffold_mode_with_lookup(&|_key| None),
        HybridPdfOcr2ScaffoldMode::Disabled
    );
    assert_eq!(
        hybrid_page_ocr2_scaffold_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE_ENV)
                .then(|| "unknown".to_string())
        }),
        HybridPdfOcr2ScaffoldMode::Disabled
    );
}

#[test]
fn ocr2_region_pipeline_mode_accepts_render_dispatch() {
    assert_eq!(
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE_ENV)
                .then(|| "render_dispatch".to_string())
        }),
        HybridPdfOcr2RegionPipelineMode::RenderDispatch
    );
}

#[test]
fn ocr2_region_pipeline_mode_defaults_unknown_values_to_disabled() {
    assert_eq!(
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|_key| None),
        HybridPdfOcr2RegionPipelineMode::Disabled
    );
    assert_eq!(
        hybrid_page_ocr2_region_pipeline_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE_ENV)
                .then(|| "parallel".to_string())
        }),
        HybridPdfOcr2RegionPipelineMode::Disabled
    );
}

#[test]
fn ocr2_region_render_chunk_mode_accepts_region() {
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
                .then(|| "region".to_string())
        }),
        HybridPdfOcr2RegionRenderChunkMode::Region
    );
}

#[test]
fn ocr2_region_render_chunk_mode_accepts_region_seed_page() {
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
                .then(|| "region_seed_page".to_string())
        }),
        HybridPdfOcr2RegionRenderChunkMode::RegionSeedPage
    );
    assert_eq!(
        HybridPdfOcr2RegionRenderChunkMode::RegionSeedPage.as_str(),
        "region-seed-page"
    );
}

#[test]
fn ocr2_region_render_chunk_mode_accepts_all() {
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
                .then(|| "all".to_string())
        }),
        HybridPdfOcr2RegionRenderChunkMode::All
    );
    assert_eq!(HybridPdfOcr2RegionRenderChunkMode::All.as_str(), "all");
}

#[test]
fn ocr2_region_render_chunk_mode_accepts_page_area_desc() {
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
                .then(|| "page_area_desc".to_string())
        }),
        HybridPdfOcr2RegionRenderChunkMode::PageAreaDesc
    );
    assert_eq!(
        HybridPdfOcr2RegionRenderChunkMode::PageAreaDesc.as_str(),
        "page-area-desc"
    );
}

#[test]
fn ocr2_region_render_chunk_mode_accepts_page_max_area_desc() {
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
                .then(|| "page_max_area_desc".to_string())
        }),
        HybridPdfOcr2RegionRenderChunkMode::PageMaxAreaDesc
    );
    assert_eq!(
        HybridPdfOcr2RegionRenderChunkMode::PageMaxAreaDesc.as_str(),
        "page-max-area-desc"
    );
}

#[test]
fn ocr2_region_render_chunk_mode_defaults_unknown_values_to_page() {
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|_key| None),
        HybridPdfOcr2RegionRenderChunkMode::Page
    );
    assert_eq!(
        hybrid_page_ocr2_region_render_chunk_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK_ENV)
                .then(|| "shard".to_string())
        }),
        HybridPdfOcr2RegionRenderChunkMode::Page
    );
}
