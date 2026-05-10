#[test]
fn docling_structure_recovery_page_range_fallback_covers_structure_pages_and_failed_text() {
    let default_page = sample_page_input(0, PDF_OCR_DEFAULT_PROFILE, "docling-compatible-ocr");
    let backend_page =
        sample_page_input(1, PDF_OCR_BACKEND_TEXT_PROFILE, "docling-backend-text-ocr");
    let hosted_page = sample_page_input(
        2,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        "hosted-vlm-direct-ocr",
    );
    let inputs = vec![
        default_page.clone(),
        backend_page.clone(),
        hosted_page.clone(),
    ];
    let results = vec![
        PdfOcrShardResult::succeeded(&default_page, "default text", 1.0),
        PdfOcrShardResult::failed(&backend_page, "backend text failed"),
        PdfOcrShardResult::succeeded(&hosted_page, "hosted text", 1.0),
    ];

    let pages = docling_page_range_fallback_page_indices(&inputs, &results, true);

    assert_eq!(pages.into_iter().collect::<Vec<_>>(), vec![0, 1]);
}

#[test]
fn docling_structure_recovery_eager_fallback_covers_structure_pages_only() {
    let default_page = sample_page_input(0, PDF_OCR_DEFAULT_PROFILE, "docling-compatible-ocr");
    let backend_page =
        sample_page_input(1, PDF_OCR_BACKEND_TEXT_PROFILE, "docling-backend-text-ocr");
    let fast_page = sample_page_input(2, PDF_OCR_FAST_TEXT_PROFILE, "docling-fast-text-ocr");
    let hosted_page = sample_page_input(
        3,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        "hosted-vlm-direct-ocr",
    );
    let inputs = vec![default_page, backend_page, fast_page, hosted_page];

    let pages = docling_structure_recovery_page_range_fallback_pages(&inputs, true);

    assert_eq!(pages.into_iter().collect::<Vec<_>>(), vec![0]);
    assert!(docling_structure_recovery_page_range_fallback_pages(&inputs, false).is_empty());
}

#[test]
fn docling_structure_recovery_eager_fallback_keeps_hosted_work_schedulable() {
    let default_page = sample_page_input(0, PDF_OCR_DEFAULT_PROFILE, "docling-compatible-ocr");
    let hosted_page = sample_page_input(
        1,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        "hosted-vlm-direct-ocr",
    );
    let mut hosted_region = sample_region_input();
    hosted_region.page_index = 1;
    hosted_region.shard_element_id = "hosted-region".to_string();
    hosted_region.parent_shard_element_id = hosted_page.shard_element_id.clone();
    hosted_region.reading_order_key = "000001.000100".to_string();
    let inputs = vec![default_page, hosted_page.clone(), hosted_region.clone()];
    let fallback_pages = std::collections::BTreeSet::from([0]);

    let scheduled =
        scheduled_inputs_without_docling_page_range_fallback_pages(inputs, &fallback_pages);

    assert_eq!(scheduled.len(), 2);
    assert_eq!(scheduled[0].shard_element_id, hosted_page.shard_element_id);
    assert_eq!(
        scheduled[1].shard_element_id,
        hosted_region.shard_element_id
    );
}

#[test]
fn docling_page_range_fallback_allows_failed_structure_rows_in_docling_mode() {
    let default_page = sample_page_input(0, PDF_OCR_DEFAULT_PROFILE, "docling-compatible-ocr");
    let hosted_page = sample_page_input(
        1,
        PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
        "hosted-vlm-direct-ocr",
    );
    let inputs = vec![default_page.clone(), hosted_page.clone()];
    let results = vec![
        PdfOcrShardResult::failed(&default_page, "structure page OCR failed"),
        PdfOcrShardResult::failed(&hosted_page, "hosted OCR failed"),
    ];
    let pages = docling_page_range_fallback_page_indices(&inputs, &results, true);

    assert_eq!(pages.into_iter().collect::<Vec<_>>(), vec![0]);
    assert!(has_unhandled_non_success_result(
        &inputs,
        &results,
        &std::collections::BTreeSet::from([0]),
        true,
    ));
    assert!(has_unhandled_non_success_result(
        &inputs,
        &results,
        &std::collections::BTreeSet::from([0]),
        false,
    ));
}

#[test]
fn legacy_page_range_fallback_only_covers_failed_backend_text() {
    let default_page = sample_page_input(0, PDF_OCR_DEFAULT_PROFILE, "docling-compatible-ocr");
    let backend_page =
        sample_page_input(1, PDF_OCR_BACKEND_TEXT_PROFILE, "docling-backend-text-ocr");
    let inputs = vec![default_page.clone(), backend_page.clone()];
    let results = vec![
        PdfOcrShardResult::failed(&default_page, "default page failed"),
        PdfOcrShardResult::failed(&backend_page, "backend text failed"),
    ];

    let pages = docling_page_range_fallback_page_indices(&inputs, &results, false);

    assert_eq!(pages.into_iter().collect::<Vec<_>>(), vec![1]);
    assert!(has_unhandled_non_success_result(
        &inputs,
        &results,
        &std::collections::BTreeSet::from([1]),
        false,
    ));
}
