#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV, DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV, HybridPdfOcrProfilePlanner, PdfPageRenderSelection,
    apply_hybrid_page_hosted_vlm_profile_plan_for_profiles,
    apply_hybrid_page_ocr_profile_plan_for_profiles,
    apply_hybrid_page_ocr2_profile_plan_for_profiles, hybrid_page_ocr_profile_planner_with_lookup,
    hybrid_page_ocr_render_profile_with_lookup, hybrid_page_ocr_render_selection_with_lookup,
    sample_ocr_input,
};

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_selection_defaults_to_shard_fallback() {
    let selection = hybrid_page_ocr_render_selection_with_lookup(&|_| None);

    assert_eq!(selection, PdfPageRenderSelection::ShardFallbackPages);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_selection_accepts_all_pages_override() {
    let selection = hybrid_page_ocr_render_selection_with_lookup(&|key| {
        (key == DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV).then(|| "all-pages".to_string())
    });

    assert_eq!(selection, PdfPageRenderSelection::AllPages);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_selection_accepts_region_shards_override() {
    let selection = hybrid_page_ocr_render_selection_with_lookup(&|key| {
        (key == DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV).then(|| "region-shards".to_string())
    });

    assert_eq!(selection, PdfPageRenderSelection::RegionShards);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_profile_applies_ocr2_dpi_override_only_to_ocr2_pages() {
    let profile = hybrid_page_ocr_render_profile_with_lookup(true, &|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI_ENV).then(|| "360".to_string())
    });

    assert_eq!(profile.dpi, 360);

    let compatible_profile = hybrid_page_ocr_render_profile_with_lookup(false, &|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI_ENV).then(|| "360".to_string())
    });

    assert_eq!(compatible_profile.dpi, 300);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_profile_rejects_ocr2_dpi_downgrade() {
    let profile = hybrid_page_ocr_render_profile_with_lookup(true, &|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR2_RENDER_DPI_ENV).then(|| "180".to_string())
    });

    assert_eq!(profile.dpi, 300);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_profile_planner_accepts_fast_risk_window_override() {
    let planner = hybrid_page_ocr_profile_planner_with_lookup(&|key| {
        (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
            .then(|| "fast_risk_window".to_string())
    });

    assert_eq!(planner, HybridPdfOcrProfilePlanner::FastRiskWindow);
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV).then(|| "fast-all".to_string())
        }),
        HybridPdfOcrProfilePlanner::FastAll
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV).then(|| "ocr2-all".to_string())
        }),
        HybridPdfOcrProfilePlanner::HostedVlmAll
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
                .then(|| "ocr2_risk_window".to_string())
        }),
        HybridPdfOcrProfilePlanner::HostedVlmRiskWindow
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
                .then(|| "hosted_vlm_risk_window".to_string())
        }),
        HybridPdfOcrProfilePlanner::HostedVlmRiskWindow
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|_| None),
        HybridPdfOcrProfilePlanner::Disabled
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_risk_window_keeps_source_range_rendering() {
    assert!(HybridPdfOcrProfilePlanner::HostedVlmAll.requires_rendered_page_images());
    assert!(!HybridPdfOcrProfilePlanner::HostedVlmRiskWindow.requires_rendered_page_images());
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_profile_plan_keeps_risk_window_accurate() {
    let inputs = (0..6)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..6)
        .map(|page_index| sample_source_page_profile(page_index, page_index == 2))
        .collect::<Vec<_>>();

    let planned = apply_hybrid_page_ocr_profile_plan_for_profiles(inputs, profiles.as_slice());
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-fast-text-ocr",
            "docling-compatible-page-ocr-v1",
            "docling-compatible-page-ocr-v1",
            "docling-compatible-page-ocr-v1",
            "docling-fast-text-ocr",
            "docling-fast-text-ocr",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_profile_plan_keeps_risk_window_accurate() {
    let inputs = (0..6)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..6)
        .map(|page_index| sample_source_page_profile(page_index, page_index == 2))
        .collect::<Vec<_>>();

    let planned =
        apply_hybrid_page_hosted_vlm_profile_plan_for_profiles(inputs, profiles.as_slice());
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-fast-text-ocr",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-fast-text-ocr",
            "docling-fast-text-ocr",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn legacy_ocr2_profile_plan_alias_uses_hosted_vlm_profile() {
    let inputs = (0..6)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..6)
        .map(|page_index| sample_source_page_profile(page_index, page_index == 2))
        .collect::<Vec<_>>();

    let planned = apply_hybrid_page_ocr2_profile_plan_for_profiles(inputs, profiles.as_slice());
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-fast-text-ocr",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-fast-text-ocr",
            "docling-fast-text-ocr",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn sample_source_page_profile(
    page_index: u32,
    fast_profile_risk: bool,
) -> xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
    xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
        page_index,
        content_bytes: 1024,
        operation_count: if fast_profile_risk { 695 } else { 24 },
        text_show_ops: if fast_profile_risk { 195 } else { 10 },
        path_ops: if fast_profile_risk { 73 } else { 8 },
        rectangle_ops: if fast_profile_risk { 2 } else { 0 },
        draw_object_ops: 0,
        estimated_weight: 1,
    }
}
