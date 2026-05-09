#[test]
fn failed_page_recovery_mode_accepts_only_hosted_vlm_page() {
    assert_eq!(
        failed_page_recovery_mode_with_lookup(&|_key| None),
        HybridPdfFailedPageRecoveryMode::Disabled,
    );
    assert_eq!(
        failed_page_recovery_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV)
                .then(|| "hosted_vlm_page".to_string())
        }),
        HybridPdfFailedPageRecoveryMode::HostedVlmPage,
    );
    assert_eq!(
        failed_page_recovery_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY_ENV)
                .then(|| "full-document".to_string())
        }),
        HybridPdfFailedPageRecoveryMode::Disabled,
    );
}

#[test]
fn failed_page_recovery_candidates_only_cover_failed_non_hosted_pages() {
    let mut failed_page = sample_region_input();
    failed_page.shard_type = "page".to_string();
    failed_page.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    failed_page.ocr_engine = "docling-fast-text-ocr".to_string();
    failed_page.shard_element_id = "failed-page".to_string();

    let mut empty_page = failed_page.clone();
    empty_page.page_index = 2;
    empty_page.shard_element_id = "empty-page".to_string();

    let mut hosted_failed_page = failed_page.clone();
    hosted_failed_page.page_index = 3;
    hosted_failed_page.ocr_profile = PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string();
    hosted_failed_page.ocr_engine = "hosted-vlm-direct-ocr".to_string();
    hosted_failed_page.shard_element_id = "hosted-failed-page".to_string();

    let mut failed_region = failed_page.clone();
    failed_region.page_index = 4;
    failed_region.shard_type = "region".to_string();
    failed_region.shard_element_id = "failed-region".to_string();

    let mut failed_backend_text_page = failed_page.clone();
    failed_backend_text_page.page_index = 5;
    failed_backend_text_page.ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    failed_backend_text_page.ocr_engine = "docling-backend-text-ocr".to_string();
    failed_backend_text_page.shard_element_id = "failed-backend-text-page".to_string();

    let inputs = vec![
        failed_page.clone(),
        empty_page.clone(),
        hosted_failed_page.clone(),
        failed_region.clone(),
        failed_backend_text_page.clone(),
    ];
    let results = vec![
        PdfOcrShardResult::failed(&failed_page, "source page failed"),
        PdfOcrShardResult::succeeded(&empty_page, "   ", 1.0),
        PdfOcrShardResult::failed(&hosted_failed_page, "hosted page failed"),
        PdfOcrShardResult::failed(&failed_region, "region failed"),
        PdfOcrShardResult::failed(&failed_backend_text_page, "backend text failed"),
    ];

    let candidates = failed_page_recovery_candidates(inputs.as_slice(), results.as_slice());

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].0, 0);
    assert_eq!(
        candidates[0].1.ocr_profile,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE
    );
    assert_eq!(candidates[0].1.ocr_engine, "hosted-vlm-direct-ocr");
    assert_eq!(candidates[0].1.shard_element_id, "failed-page");
    assert_eq!(candidates[1].0, 1);
    assert_eq!(candidates[1].1.shard_element_id, "empty-page");
}

#[test]
fn failed_page_recovery_input_preserves_shard_identity() {
    let mut page = sample_region_input();
    page.shard_type = "page".to_string();
    page.ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    page.ocr_engine = "docling-fast-text-ocr".to_string();

    let recovery = failed_page_recovery_input(&page);

    assert_eq!(recovery.shard_element_id, page.shard_element_id);
    assert_eq!(recovery.reading_order_key, page.reading_order_key);
    assert_eq!(recovery.ocr_profile, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE);
    assert_eq!(recovery.ocr_engine, "hosted-vlm-direct-ocr");
}

