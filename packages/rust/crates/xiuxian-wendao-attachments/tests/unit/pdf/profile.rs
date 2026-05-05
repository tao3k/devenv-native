use std::path::Path;

use super::{source_pdf_page_profiles, source_pdf_page_profiles_cached};

#[test]
fn source_pdf_page_profiles_reports_missing_pdf_error() {
    let error = source_pdf_page_profiles(Path::new("/tmp/missing-wendao-profile.pdf"))
        .expect_err("missing PDF should fail profile extraction");

    assert!(error.contains("load PDF with lopdf"));
}

#[test]
fn source_pdf_page_profiles_cached_reports_missing_pdf_error() {
    let error = source_pdf_page_profiles_cached(Path::new("/tmp/missing-wendao-profile.pdf"))
        .expect_err("missing PDF should fail cached profile extraction");

    assert!(error.contains("load PDF with lopdf"));
}
