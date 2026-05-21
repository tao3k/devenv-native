#[test]
fn endpoint_index_for_request_round_robins_endpoint_pool() -> Result<(), String> {
    assert_eq!(endpoint_index_for_request(0, 3)?, 0);
    assert_eq!(endpoint_index_for_request(1, 3)?, 1);
    assert_eq!(endpoint_index_for_request(2, 3)?, 2);
    assert_eq!(endpoint_index_for_request(3, 3)?, 0);
    assert!(endpoint_index_for_request(0, 0).is_err());
    Ok(())
}

#[test]
fn source_pdf_page_range_endpoint_affinity_targets_single_fast_text_pdf_page() {
    let mut input = sample_ocr_input("/tmp/source.pdf", 5, "page");
    input.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    assert!(
        source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
            std::slice::from_ref(&input),
            &enabled,
        )
    );

    let disabled = |_: &str| None;
    assert!(
        !source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
            std::slice::from_ref(&input),
            &disabled,
        )
    );
}

#[test]
fn source_pdf_page_range_endpoint_affinity_routes_single_fast_text_chunk_to_first_endpoint() {
    let mut input = sample_ocr_input("/tmp/source.pdf", 5, "page");
    input.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    let Ok(endpoint_index) = source_pdf_page_range_chunk_endpoint_index_with_lookup(
        4,
        std::slice::from_ref(&input),
        &enabled,
        || Err("affinity should not advance the round-robin cursor".to_string()),
    ) else {
        panic!("single fast-text source chunk should resolve")
    };

    assert_eq!(endpoint_index, 0);
    assert!(
        source_pdf_page_range_chunk_endpoint_index_with_lookup(0, &[input], &enabled, || Err(
            "affinity should not advance the round-robin cursor".to_string()
        ),)
        .is_err()
    );
}

#[test]
fn source_pdf_page_range_endpoint_affinity_uses_round_robin_for_other_chunks() {
    let mut first = sample_ocr_input("/tmp/source.pdf", 11, "page");
    let mut second = sample_ocr_input("/tmp/source.pdf", 12, "page");
    first.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    second.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    let Ok(endpoint_index) = source_pdf_page_range_chunk_endpoint_index_with_lookup(
        4,
        &[first, second],
        &enabled,
        || Ok(2),
    ) else {
        panic!("multi-page fast-text source chunk should use round-robin")
    };

    assert_eq!(endpoint_index, 2);
}

#[test]
fn source_pdf_page_range_endpoint_affinity_rejects_non_single_fast_text_pdf_chunks() {
    let mut fast_page = sample_ocr_input("/tmp/source.pdf", 5, "page");
    fast_page.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let mut second_fast_page = sample_ocr_input("/tmp/source.pdf", 6, "page");
    second_fast_page.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let mut region = sample_ocr_input("/tmp/source.pdf", 5, "region");
    region.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let mut png_source = sample_ocr_input("/tmp/source.png", 5, "page");
    png_source.ocr_profile = super::PDF_OCR_FAST_TEXT_PROFILE.to_string();
    let enabled = |key: &str| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_FAST_TEXT_ENDPOINT_AFFINITY")
            .then(|| "single-page-first".to_string())
    };

    assert!(
        !source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(
            &[fast_page, second_fast_page],
            &enabled,
        )
    );
    assert!(!source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(&[region], &enabled,));
    assert!(
        !source_pdf_page_range_chunk_prefers_first_endpoint_with_lookup(&[png_source], &enabled,)
    );
}
