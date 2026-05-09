#[test]
fn pdf_ocr_scheduler_extracts_backend_text_locally_for_source_pages() {
    let mut inputs = vec![sample_ocr_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        0,
        "page",
    )];
    inputs[0].ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-backend-text-ocr".to_string();

    let results = local_backend_text_results_for_tests(inputs.as_slice());

    let result = results[0]
        .as_ref()
        .expect("backend text page should be extracted locally");
    assert_eq!(result.ocr_profile, PDF_OCR_BACKEND_TEXT_PROFILE);
    assert_eq!(result.text_mime_type, "text/markdown");
    assert!(result.text.as_deref().unwrap_or_default().len() > 1_000);
}

#[test]
fn pdf_ocr_scheduler_can_extract_fast_text_locally_for_parent_pages() {
    let mut inputs = vec![sample_ocr_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        5,
        "page",
    )];
    inputs[0].ocr_profile = PDF_OCR_FAST_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-fast-text-ocr".to_string();

    let backend_only = local_backend_text_results_for_tests(inputs.as_slice());
    assert!(
        backend_only[0].is_none(),
        "fast-text rows require the explicit local fast-text mode"
    );

    let results = local_backend_and_fast_text_results_for_tests(inputs.as_slice());

    let result = results[0]
        .as_ref()
        .expect("fast-text parent page should be extracted locally when enabled");
    assert_eq!(result.ocr_profile, PDF_OCR_FAST_TEXT_PROFILE);
    assert_eq!(result.text_mime_type, "text/markdown");
    assert!(result.text.as_deref().unwrap_or_default().len() > 1_000);
}

#[test]
fn pdf_ocr_scheduler_keeps_empty_local_backend_text_dispatching_by_default() {
    let mut inputs = vec![sample_source_page_range_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        0,
    )];
    inputs[0].ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-backend-text-ocr".to_string();

    let results = local_empty_backend_text_dispatch_python_results_for_tests(inputs.as_slice());

    assert!(
        results[0].is_none(),
        "empty local backend-text should still dispatch to Python unless fail-fast is enabled"
    );
}

#[test]
fn pdf_ocr_scheduler_can_fail_fast_empty_source_range_backend_text() {
    let mut inputs = vec![sample_source_page_range_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        1,
    )];
    inputs[0].ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-backend-text-ocr".to_string();

    let results = local_empty_backend_text_fail_fast_results_for_tests(inputs.as_slice());

    let result = results[0]
        .as_ref()
        .expect("empty source-page-range backend-text should become a local failed row");
    assert_eq!(result.status, PdfOcrShardResultStatus::Failed);
    let message = result.error_message.as_deref().unwrap_or_default();
    assert!(message.contains("local backend-text returned empty text"));
    assert!(message.contains("source-page-range placeholder"));
    assert!(message.contains("full-document fallback"));
}

#[test]
fn pdf_ocr_scheduler_can_fail_fast_source_range_backend_text_errors() {
    let mut inputs = vec![sample_source_page_range_input(
        autosearch_fixture().to_string_lossy().as_ref(),
        2,
    )];
    inputs[0].ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    inputs[0].ocr_engine = "docling-backend-text-ocr".to_string();

    let results = local_backend_text_error_fail_fast_results_for_tests(inputs.as_slice());

    let result = results[0]
        .as_ref()
        .expect("source-page-range backend-text extraction errors should become local failed rows");
    assert_eq!(result.status, PdfOcrShardResultStatus::Failed);
    let message = result.error_message.as_deref().unwrap_or_default();
    assert!(message.contains("local backend-text source extraction failed"));
    assert!(message.contains("synthetic lopdf failure"));
    assert!(message.contains("source-page-range placeholder"));
}

#[test]
fn pdf_ocr_scheduler_keeps_successful_pages_when_one_local_backend_page_fails() {
    let source = autosearch_fixture();
    let inputs = (0..3)
        .map(|page_index| {
            let mut input =
                sample_source_page_range_input(source.to_string_lossy().as_ref(), page_index);
            input.ocr_profile = PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
            input.ocr_engine = "docling-backend-text-ocr".to_string();
            input
        })
        .collect::<Vec<_>>();

    let results = local_partial_backend_text_error_fail_fast_results_for_tests(inputs.as_slice());

    assert_eq!(
        results[0]
            .as_ref()
            .and_then(|result| result.text.as_deref()),
        Some("local page 0")
    );
    assert_eq!(
        results[1].as_ref().map(|result| result.status.clone()),
        Some(PdfOcrShardResultStatus::Failed)
    );
    assert_eq!(
        results[2]
            .as_ref()
            .and_then(|result| result.text.as_deref()),
        Some("local page 2")
    );
    let message = results[1]
        .as_ref()
        .and_then(|result| result.error_message.as_deref())
        .unwrap_or_default();
    assert!(message.contains("synthetic page failure"));
}
