use std::path::Path;

use super::{
    PdfSourcePageProfile, classify_pdf_source_page, pdf_source_page_is_backend_text_topup_profile,
    pdf_source_page_is_fast_profile_risk, pdf_source_page_requires_structure_authority,
    source_pdf_page_profiles, source_pdf_page_profiles_cached,
};

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

#[test]
fn classifies_plain_text_pages_as_text_shortcut_eligible() {
    let profile = sample_profile(0, 16, 0, 0, 0, 0);

    let classification = classify_pdf_source_page(&profile);

    assert!(!classification.structure_authority_required);
    assert!(!classification.ocr_patch_candidate);
    assert!(classification.text_shortcut_eligible);
}

#[test]
fn classifies_draw_object_pages_as_structure_authority_required() {
    let profile = sample_profile(1, 24, 0, 0, 1, 0);

    let classification = classify_pdf_source_page(&profile);

    assert!(classification.structure_authority_required);
    assert!(!classification.text_shortcut_eligible);
    assert!(pdf_source_page_requires_structure_authority(&profile));
}

#[test]
fn classifies_table_path_band_as_structure_and_ocr_patch_candidate() {
    let profile = sample_profile(2, 180, 80, 0, 0, 720);

    let classification = classify_pdf_source_page(&profile);

    assert!(classification.structure_authority_required);
    assert!(classification.ocr_patch_candidate);
    assert!(!classification.text_shortcut_eligible);
    assert!(pdf_source_page_is_fast_profile_risk(&profile));
}

#[test]
fn keeps_backend_text_topup_signal_available_for_consumers() {
    let profile = sample_profile(3, 360, 0, 0, 0, 700);

    assert!(pdf_source_page_is_backend_text_topup_profile(&profile));
    assert!(!classify_pdf_source_page(&profile).structure_authority_required);
}

fn sample_profile(
    page_index: u32,
    text_show_ops: u32,
    path_ops: u32,
    rectangle_ops: u32,
    draw_object_ops: u32,
    operation_count: u32,
) -> PdfSourcePageProfile {
    PdfSourcePageProfile {
        page_index,
        content_bytes: 4096,
        operation_count,
        text_show_ops,
        path_ops,
        rectangle_ops,
        draw_object_ops,
        estimated_weight: 1,
    }
}
