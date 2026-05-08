use std::path::{Path, PathBuf};

use super::source_pdf_page_texts;

#[test]
fn source_pdf_page_texts_extracts_fixture_page_text() {
    let texts = source_pdf_page_texts(autosearch_fixture().as_path(), &[0])
        .expect("fixture page text should extract");

    assert_eq!(texts.len(), 1);
    assert!(texts[0].len() > 1_000);
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
