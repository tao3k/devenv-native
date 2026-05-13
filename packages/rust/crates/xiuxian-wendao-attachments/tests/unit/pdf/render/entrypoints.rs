use super::should_retry_pdfium_render_message;

#[test]
fn pdfium_render_retry_matches_internal_errors_only() {
    assert!(should_retry_pdfium_render_message(
        "load page 13: PdfiumLibraryInternalError(Unknown)"
    ));
    assert!(should_retry_pdfium_render_message(
        "load PDF `/tmp/source.pdf`: PdfiumLibraryInternalError(FormatError)"
    ));
    assert!(!should_retry_pdfium_render_message(
        "load PDF with lopdf: trailer not found"
    ));
    assert!(!should_retry_pdfium_render_message(
        "unsupported non-PDF input"
    ));
}
