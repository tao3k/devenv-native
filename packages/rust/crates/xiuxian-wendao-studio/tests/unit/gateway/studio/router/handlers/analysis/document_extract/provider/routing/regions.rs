#[cfg(feature = "document-extract-pdf-source-range")]
use std::fs;
#[cfg(feature = "document-extract-pdf-source-range")]
use std::path::Path;

#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV, DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV,
    HybridPdfOcr2RegionPlanner, assert_close,
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, has_ocr2_recovery_page_candidates,
    hybrid_page_ocr_region_context_ratio_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup, sample_ocr_input,
};

#[cfg(feature = "document-extract-pdf-source-range")]
mod automatic;

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_requests_match_selected_source() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    fs::write(source.as_path(), b"%PDF").map_err(|error| error.to_string())?;
    let source_text = source.to_string_lossy();
    let regions_json = format!(
        r#"[{{"source":"{source_text}","regions":[{{"pageIndex":0,"regionIndex":3,"regionBox":{{"left":72.0,"bottom":72.0,"right":144.0,"top":144.0}},"readingOrderKey":"000000.000003"}}]}}]"#
    );

    let regions =
        hybrid_page_ocr_region_requests_for_source_with_lookup(source.as_path(), &|key| {
            (key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).then(|| regions_json.clone())
        })?;

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].page_index, 0);
    assert_eq!(regions[0].region_index, 3);
    assert_close(regions[0].region_box.left, 59.04);
    assert_close(regions[0].region_box.bottom, 59.04);
    assert_close(regions[0].region_box.right, 156.96);
    assert_close(regions[0].region_box.top, 156.96);
    assert_eq!(
        regions[0].reading_order_key.as_deref(),
        Some("000000.000003")
    );
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_requests_reject_missing_source() {
    let regions_json = r#"[{"source":"/tmp/other.pdf","regions":[{"pageIndex":0,"regionIndex":0,"regionBox":{"left":0.0,"bottom":0.0,"right":10.0,"top":10.0}}]}]"#;

    let Err(error) = hybrid_page_ocr_region_requests_for_source_with_lookup(
        Path::new("/tmp/source.pdf"),
        &|key| (key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).then(|| regions_json.into()),
    ) else {
        panic!("expected missing source to fail");
    };

    assert!(error.contains("no hybrid PDF region fixture matched"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_context_ratio_accepts_zero_override() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    fs::write(source.as_path(), b"%PDF").map_err(|error| error.to_string())?;
    let source_text = source.to_string_lossy();
    let regions_json = format!(
        r#"[{{"source":"{source_text}","regions":[{{"pageIndex":0,"regionIndex":3,"regionBox":{{"left":72.0,"bottom":72.0,"right":144.0,"top":144.0}}}}]}}]"#
    );

    let regions =
        hybrid_page_ocr_region_requests_for_source_with_lookup(source.as_path(), &|key| {
            if key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV {
                Some(regions_json.clone())
            } else {
                (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "0".to_string())
            }
        })?;

    assert_eq!(regions[0].region_box.left, 72.0);
    assert_eq!(regions[0].region_box.bottom, 72.0);
    assert_eq!(regions[0].region_box.right, 144.0);
    assert_eq!(regions[0].region_box.top, 144.0);
    assert_eq!(
        hybrid_page_ocr_region_context_ratio_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "1.8".to_string())
        }),
        1.0
    );
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_region_planner_accepts_profile_risk_window() {
    assert_eq!(
        hybrid_page_ocr2_region_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV)
                .then(|| "profile_risk_window".to_string())
        }),
        HybridPdfOcr2RegionPlanner::ProfileRiskWindow
    );
    assert_eq!(
        hybrid_page_ocr2_region_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV)
                .then(|| "profile-risk-window-slices".to_string())
        }),
        HybridPdfOcr2RegionPlanner::ProfileRiskWindowSlices
    );
    assert_eq!(
        hybrid_page_ocr2_region_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV)
                .then(|| "profile-risk-window-adaptive".to_string())
        }),
        HybridPdfOcr2RegionPlanner::ProfileRiskWindowAdaptive
    );
    assert_eq!(
        hybrid_page_ocr2_region_planner_with_lookup(&|_key| None),
        HybridPdfOcr2RegionPlanner::Disabled
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn ocr2_region_candidate_detection_requires_direct_page_profile() {
    let mut region = sample_ocr_input(1, "region");
    region.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
    assert!(!has_ocr2_recovery_page_candidates(&[region]));

    let mut fast_page = sample_ocr_input(1, "page");
    fast_page.ocr_profile = "docling-fast-text-ocr".to_string();
    assert!(!has_ocr2_recovery_page_candidates(&[fast_page.clone()]));

    fast_page.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
    assert!(has_ocr2_recovery_page_candidates(&[fast_page]));
}
