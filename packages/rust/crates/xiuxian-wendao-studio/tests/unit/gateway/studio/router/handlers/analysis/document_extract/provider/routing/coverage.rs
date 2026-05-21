#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    sample_ocr_input, sample_ocr_result, validate_hybrid_page_coverage,
    validate_hybrid_shard_coverage, validate_ocr_results_match_inputs,
    validate_successful_ocr_results, validate_successful_ocr_results_for_inputs_with_lookup,
};

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_validation_rejects_skipped_ocr_rows() {
    let Err(error) = validate_successful_ocr_results(&[sample_ocr_result(1, false)], 3, 1) else {
        panic!("expected non-success OCR status to fail");
    };

    assert!(error.contains("non-success status"));
    assert!(error.contains("shard-1"));
    assert!(error.contains("worker skipped"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_validation_allows_verified_empty_backend_text_source_pages() -> Result<(), String>
{
    let mut input = sample_ocr_input(16, "page");
    input.ocr_profile =
        xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_BACKEND_TEXT_PROFILE.to_string();
    input.image_path = "/tmp/source-page-range-00016.source-page-range".to_string();
    input.image_mime_type = "application/x-wendao-source-pdf-page".to_string();
    let mut result = sample_ocr_result(16, true);
    result.ocr_profile = input.ocr_profile.clone();
    result.text = Some(String::new());

    let Err(error) = validate_successful_ocr_results_for_inputs_with_lookup(
        &[result.clone()],
        21,
        1,
        &[input.clone()],
        &|_| None,
    ) else {
        panic!("empty backend-text source-page rows should fail by default");
    };
    assert!(error.contains("empty text"));

    validate_successful_ocr_results_for_inputs_with_lookup(&[result], 21, 1, &[input], &|key| {
        (key == "WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE")
            .then(|| "verified-empty".to_string())
    })
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_accepts_text_and_ocr_pages() -> Result<(), String> {
    validate_hybrid_page_coverage(3, &[0, 2], &[sample_ocr_result(1, true)])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_accepts_text_only_pages() -> Result<(), String> {
    validate_hybrid_page_coverage(3, &[0, 1, 2], &[])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_rejects_missing_pages() {
    let Err(error) = validate_hybrid_page_coverage(3, &[0], &[sample_ocr_result(1, true)]) else {
        panic!("expected missing page coverage to fail");
    };

    assert!(error.contains("missing page coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_rejects_duplicate_pages() {
    let Err(error) = validate_hybrid_page_coverage(3, &[0, 1], &[sample_ocr_result(1, true)])
    else {
        panic!("expected duplicate page coverage to fail");
    };

    assert!(error.contains("duplicate page coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_coverage_keeps_native_text_page() -> Result<(), String> {
    let input = sample_ocr_input(1, "region");
    let result = sample_ocr_result(1, true);

    validate_hybrid_shard_coverage(3, &[0, 1, 2], &[input], &[result])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_coverage_accepts_bound_parent_page() -> Result<(), String> {
    let parent = sample_ocr_input(1, "page");
    let mut region = sample_ocr_input(1, "region");
    region.shard_element_id = "region-shard-1".to_string();
    region.parent_shard_element_id = parent.shard_element_id.clone();
    let parent_result = sample_ocr_result(1, true);
    let mut region_result = sample_ocr_result(1, true);
    region_result.shard_element_id = region.shard_element_id.clone();

    validate_hybrid_shard_coverage(
        3,
        &[0, 2],
        &[parent, region],
        &[parent_result, region_result],
    )
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_coverage_rejects_unbound_parent_page() {
    let parent = sample_ocr_input(1, "page");
    let mut region = sample_ocr_input(1, "region");
    region.shard_element_id = "region-shard-1".to_string();
    region.parent_shard_element_id = "other-parent".to_string();
    let parent_result = sample_ocr_result(1, true);
    let mut region_result = sample_ocr_result(1, true);
    region_result.shard_element_id = region.shard_element_id.clone();

    let Err(error) = validate_hybrid_shard_coverage(
        3,
        &[0, 2],
        &[parent, region],
        &[parent_result, region_result],
    ) else {
        panic!("expected unbound region parent to fail");
    };

    assert!(error.contains("does not match page 1 coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_coverage_requires_native_text_page() {
    let input = sample_ocr_input(1, "region");
    let result = sample_ocr_result(1, true);

    let Err(error) = validate_hybrid_shard_coverage(3, &[0, 2], &[input], &[result]) else {
        panic!("expected region without native text page to fail");
    };

    assert!(error.contains("has no native text coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_page_shard_coverage_still_replaces_full_page() -> Result<(), String> {
    let input = sample_ocr_input(1, "page");
    let result = sample_ocr_result(1, true);

    validate_hybrid_shard_coverage(3, &[0, 2], &[input], &[result])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_validation_rejects_unknown_shard_result() {
    let input = sample_ocr_input(1, "region");
    let mut result = sample_ocr_result(1, true);
    result.shard_element_id = "unknown-shard".to_string();

    let Err(error) = validate_ocr_results_match_inputs(&[input], &[result]) else {
        panic!("expected unknown OCR result shard to fail");
    };

    assert!(error.contains("unknown shard id"));
}
