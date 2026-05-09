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
        (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV).then(|| "360".to_string())
    });

    assert_eq!(profile.dpi, 360);

    let compatible_profile = hybrid_page_ocr_render_profile_with_lookup(false, &|key| {
        (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV).then(|| "360".to_string())
    });

    assert_eq!(compatible_profile.dpi, 300);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_profile_rejects_ocr2_dpi_downgrade() {
    let profile = hybrid_page_ocr_render_profile_with_lookup(true, &|key| {
        (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI_ENV).then(|| "180".to_string())
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
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
                .then(|| "hosted-vlm-all".to_string())
        }),
        HybridPdfOcrProfilePlanner::HostedVlmAll
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
                .then(|| "hosted_vlm_risk_window".to_string())
        }),
        HybridPdfOcrProfilePlanner::HostedVlmRiskWindow
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
                .then(|| "hosted-vlm-risk-window-backend-text".to_string())
        }),
        HybridPdfOcrProfilePlanner::HostedVlmRiskWindowBackendText
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER_ENV)
                .then(|| "docling-structure-recovery".to_string())
        }),
        HybridPdfOcrProfilePlanner::DoclingStructureRecovery
    );
    assert_eq!(
        hybrid_page_ocr_profile_planner_with_lookup(&|_| None),
        HybridPdfOcrProfilePlanner::Disabled
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_pdf_backend_text_topup_accepts_disabled_override() {
    assert_eq!(
        hybrid_pdf_backend_text_topup_with_lookup(&|_| None),
        HybridPdfBackendTextTopup::Profile
    );
    assert_eq!(
        hybrid_pdf_backend_text_topup_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV).then(|| "disabled".to_string())
        }),
        HybridPdfBackendTextTopup::Disabled
    );
    assert_eq!(
        hybrid_pdf_backend_text_topup_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV).then(|| "profile".to_string())
        }),
        HybridPdfBackendTextTopup::Profile
    );
    assert_eq!(
        hybrid_pdf_backend_text_topup_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV).then(|| "hosted_vlm".to_string())
        }),
        HybridPdfBackendTextTopup::HostedVlm
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_risk_window_keeps_source_range_rendering() {
    assert!(HybridPdfOcrProfilePlanner::HostedVlmAll.requires_rendered_page_images());
    assert!(!HybridPdfOcrProfilePlanner::HostedVlmRiskWindow.requires_rendered_page_images());
    assert!(
        !HybridPdfOcrProfilePlanner::HostedVlmRiskWindowBackendText.requires_rendered_page_images()
    );
    assert!(!HybridPdfOcrProfilePlanner::DoclingStructureRecovery.requires_rendered_page_images());
}
