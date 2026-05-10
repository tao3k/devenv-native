use std::path::Path;

use super::{
    PdfSourcePageProfile, classify_pdf_source_page, pdf_source_page_is_backend_text_topup_profile,
    pdf_source_page_is_fast_profile_risk, pdf_source_page_is_text_table_candidate,
    pdf_source_page_requires_structure_authority, pdf_source_page_structure_cost,
    source_pdf_page_profiles, source_pdf_page_profiles_cached,
};

#[test]
fn source_pdf_page_profiles_reports_missing_pdf_error() {
    let Err(error) = source_pdf_page_profiles(Path::new("/tmp/missing-wendao-profile.pdf")) else {
        panic!("missing PDF should fail profile extraction")
    };

    assert!(error.contains("load PDF with lopdf"));
}

#[test]
fn source_pdf_page_profiles_cached_reports_missing_pdf_error() {
    let Err(error) = source_pdf_page_profiles_cached(Path::new("/tmp/missing-wendao-profile.pdf"))
    else {
        panic!("missing PDF should fail cached profile extraction")
    };

    assert!(error.contains("load PDF with lopdf"));
}

#[test]
fn classifies_plain_text_pages_as_text_shortcut_eligible() {
    let profile = sample_profile(0, 16, 0, 0, 0, 0);

    let classification = classify_pdf_source_page(&profile);

    assert!(!classification.structure_authority_required);
    assert!(!classification.ocr_patch_candidate);
    assert!(classification.text_shortcut_eligible);
    assert_eq!(
        classification.estimated_structure_cost,
        pdf_source_page_structure_cost(&profile)
    );
}

#[test]
fn classifies_draw_object_pages_as_structure_authority_required() {
    let profile = sample_profile(1, 24, 0, 0, 1, 0);

    let classification = classify_pdf_source_page(&profile);

    assert!(classification.structure_authority_required);
    assert!(!classification.text_shortcut_eligible);
    assert!(pdf_source_page_requires_structure_authority(&profile));
    assert!(classification.estimated_structure_cost > profile.estimated_weight);
}

#[test]
fn classifies_table_path_band_as_structure_and_ocr_patch_candidate() {
    let profile = sample_profile(2, 180, 80, 0, 0, 720);

    let classification = classify_pdf_source_page(&profile);

    assert!(classification.structure_authority_required);
    assert!(classification.ocr_patch_candidate);
    assert!(!classification.text_shortcut_eligible);
    assert!(pdf_source_page_is_fast_profile_risk(&profile));
    assert!(classification.estimated_structure_cost >= 512);
}

#[test]
fn classifies_dense_text_grid_as_structure_authority_required() {
    let profile = PdfSourcePageProfile {
        page_index: 3,
        content_bytes: 12_000,
        operation_count: 360,
        text_show_ops: 100,
        path_ops: 0,
        rectangle_ops: 0,
        draw_object_ops: 0,
        estimated_weight: 1,
    };

    let classification = classify_pdf_source_page(&profile);

    assert!(pdf_source_page_is_text_table_candidate(&profile));
    assert!(classification.structure_authority_required);
    assert!(!classification.text_shortcut_eligible);
    assert!(classification.estimated_structure_cost > profile.estimated_weight);
}

#[test]
fn keeps_short_dense_text_out_of_text_table_structure_guard() {
    let profile = PdfSourcePageProfile {
        page_index: 4,
        content_bytes: 8_192,
        operation_count: 360,
        text_show_ops: 150,
        path_ops: 0,
        rectangle_ops: 0,
        draw_object_ops: 0,
        estimated_weight: 1,
    };

    let classification = classify_pdf_source_page(&profile);

    assert!(!pdf_source_page_is_text_table_candidate(&profile));
    assert!(!classification.structure_authority_required);
    assert!(classification.text_shortcut_eligible);
}

#[test]
fn keeps_backend_text_topup_signal_available_for_consumers() {
    let profile = sample_profile(3, 360, 0, 0, 0, 700);

    assert!(pdf_source_page_is_backend_text_topup_profile(&profile));
    assert!(!classify_pdf_source_page(&profile).structure_authority_required);
}

#[test]
fn structure_cost_prioritizes_structural_pages_over_dense_text() {
    let dense_text = sample_profile(4, 360, 0, 0, 0, 700);
    let structure_risk = sample_profile(5, 180, 80, 0, 0, 720);

    assert!(
        pdf_source_page_structure_cost(&structure_risk)
            > pdf_source_page_structure_cost(&dense_text)
    );
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
