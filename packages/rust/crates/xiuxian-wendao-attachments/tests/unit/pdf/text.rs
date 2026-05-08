use std::path::{Path, PathBuf};

use super::{source_pdf_page_text_results, source_pdf_page_texts};

#[test]
fn source_pdf_page_texts_extracts_fixture_page_text() {
    let texts = source_pdf_page_texts(autosearch_fixture().as_path(), &[0])
        .expect("fixture page text should extract");

    assert_eq!(texts.len(), 1);
    assert!(texts[0].len() > 1_000);
}

#[test]
fn source_pdf_page_text_results_preserve_successful_pages_with_page_errors() {
    let results = source_pdf_page_text_results(autosearch_fixture().as_path(), &[0, u32::MAX])
        .expect("fixture PDF should load");

    assert_eq!(results.len(), 2);
    assert!(results[0].as_ref().is_ok_and(|text| text.len() > 1_000));
    let error = results[1]
        .as_ref()
        .expect_err("overflow page index should remain row-local");
    assert!(error.contains("overflowed page number"));
}

#[test]
fn source_pdf_page_texts_reports_missing_pdf_error() {
    let error = source_pdf_page_texts(Path::new("/tmp/missing-wendao-text.pdf"), &[0])
        .expect_err("missing PDF should fail text extraction");

    assert!(error.contains("load PDF with lopdf"));
}

fn autosearch_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../tests/fixtures/document-extract/milestones/autosearch-2604.17337.pdf")
}
