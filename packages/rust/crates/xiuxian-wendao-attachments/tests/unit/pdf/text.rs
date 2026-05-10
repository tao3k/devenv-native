use std::path::{Path, PathBuf};

use super::{source_pdf_page_text_results, source_pdf_page_texts};

#[test]
fn source_pdf_page_texts_extracts_fixture_page_text() {
    let texts = match source_pdf_page_texts(autosearch_fixture().as_path(), &[0]) {
        Ok(texts) => texts,
        Err(error) => panic!("fixture page text should extract: {error}"),
    };

    assert_eq!(texts.len(), 1);
    assert!(texts[0].len() > 1_000);
}

#[test]
fn source_pdf_page_text_results_preserve_successful_pages_with_page_errors() {
    let results = match source_pdf_page_text_results(autosearch_fixture().as_path(), &[0, u32::MAX])
    {
        Ok(results) => results,
        Err(error) => panic!("fixture PDF should load: {error}"),
    };

    assert_eq!(results.len(), 2);
    assert!(results[0].as_ref().is_ok_and(|text| text.len() > 1_000));
    let Err(error) = results[1].as_ref() else {
        panic!("overflow page index should remain row-local")
    };
    assert!(error.contains("overflowed page number"));
}

#[test]
fn source_pdf_page_texts_reports_missing_pdf_error() {
    let Err(error) = source_pdf_page_texts(Path::new("/tmp/missing-wendao-text.pdf"), &[0]) else {
        panic!("missing PDF should fail text extraction")
    };

    assert!(error.contains("load PDF with lopdf"));
}

fn autosearch_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../tests/fixtures/document-extract/milestones/autosearch-2604.17337.pdf")
}
