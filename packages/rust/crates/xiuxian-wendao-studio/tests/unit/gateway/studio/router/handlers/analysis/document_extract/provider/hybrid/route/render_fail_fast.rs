#[test]
fn pdfium_required_render_fallback_fails_fast() {
    let reason = "render status `fallback` is not eligible for hybrid OCR: load PDF `fixture.pdf`: PdfiumLibraryInternalError(FormatError)";
    let lookup = |key: &str| (key == "WENDAO_PDF_RENDER_REQUIRE_PDFIUM").then(|| "1".to_string());

    assert!(hybrid_page_ocr_render_failure_requires_fail_fast_with_lookup(reason, &lookup));
}

#[test]
fn pdfium_required_does_not_fail_fast_non_render_errors() {
    let lookup = |key: &str| (key == "WENDAO_PDF_RENDER_REQUIRE_PDFIUM").then(|| "1".to_string());

    assert!(
        !hybrid_page_ocr_render_failure_requires_fail_fast_with_lookup(
            "hosted VLM request timed out",
            &lookup,
        )
    );
}

#[test]
fn pdfium_not_required_keeps_legacy_fallback() {
    let reason = "render status `fallback` is not eligible for hybrid OCR: load PDF `fixture.pdf`: PdfiumLibraryInternalError(FormatError)";
    let lookup = |_key: &str| None;

    assert!(!hybrid_page_ocr_render_failure_requires_fail_fast_with_lookup(reason, &lookup));
}
