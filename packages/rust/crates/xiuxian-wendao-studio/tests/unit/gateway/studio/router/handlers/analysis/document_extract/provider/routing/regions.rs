use std::fs;
use std::path::Path;

#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV, DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV,
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, HybridPdfOcr2RegionPlanner, assert_close,
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, has_ocr2_recovery_page_candidates,
    hybrid_page_ocr_region_context_ratio_with_lookup,
    hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr2_region_planner_with_lookup, sample_ocr_input,
};

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
            (key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV)
                .then(|| "profile_risk_window".to_string())
        }),
        HybridPdfOcr2RegionPlanner::ProfileRiskWindow
    );
    assert_eq!(
        hybrid_page_ocr2_region_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV)
                .then(|| "profile-risk-window-slices".to_string())
        }),
        HybridPdfOcr2RegionPlanner::ProfileRiskWindowSlices
    );
    assert_eq!(
        hybrid_page_ocr2_region_planner_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV)
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
    region.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    assert!(!has_ocr2_recovery_page_candidates(&[region]));

    let mut fast_page = sample_ocr_input(1, "page");
    fast_page.ocr_profile = "docling-fast-text-ocr".to_string();
    assert!(!has_ocr2_recovery_page_candidates(&[fast_page.clone()]));

    fast_page.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    assert!(has_ocr2_recovery_page_candidates(&[fast_page]));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn automatic_ocr2_recovery_region_requests_build_content_band() {
    let mut fast_page = sample_ocr_input(0, "page");
    fast_page.ocr_profile = "docling-fast-text-ocr".to_string();
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();

    let disabled = automatic_ocr2_recovery_region_requests_with_lookup(
        &[fast_page.clone(), ocr2_page.clone()],
        &|_key| None,
    );
    assert!(disabled.is_empty());

    let regions =
        automatic_ocr2_recovery_region_requests_with_lookup(&[fast_page, ocr2_page], &|key| {
            if key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV {
                Some("profile-risk-window".to_string())
            } else {
                (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "0".to_string())
            }
        });

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].page_index, 1);
    assert_eq!(regions[0].region_index, 1);
    assert_close(regions[0].region_box.left, 110.16);
    assert_close(regions[0].region_box.bottom, 237.6);
    assert_close(regions[0].region_box.right, 501.84);
    assert_close(regions[0].region_box.top, 665.28);
    assert_eq!(
        regions[0].reading_order_key.as_deref(),
        Some("000001.000050")
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn automatic_ocr2_recovery_region_requests_can_slice_content_band() {
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();

    let regions = automatic_ocr2_recovery_region_requests_with_lookup(&[ocr2_page], &|key| {
        if key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV {
            Some("profile-risk-window-slices".to_string())
        } else {
            (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "0".to_string())
        }
    });

    assert_eq!(regions.len(), 3);
    assert_eq!(
        regions
            .iter()
            .map(|region| region.region_index)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        regions
            .iter()
            .map(|region| region.reading_order_key.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("000001.000030"),
            Some("000001.000050"),
            Some("000001.000070")
        ]
    );
    assert_close(regions[0].region_box.left, 110.16);
    assert_close(regions[0].region_box.bottom, 522.72);
    assert_close(regions[0].region_box.right, 501.84);
    assert_close(regions[0].region_box.top, 665.28);
    assert_close(regions[1].region_box.bottom, 380.16);
    assert_close(regions[1].region_box.top, 522.72);
    assert_close(regions[2].region_box.bottom, 237.6);
    assert_close(regions[2].region_box.top, 380.16);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn automatic_ocr2_recovery_region_requests_adaptively_splits_large_band() {
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    ocr2_page.point_to_pixel_scale_x = 4.2;
    ocr2_page.point_to_pixel_scale_y = 4.2;

    let regions = automatic_ocr2_recovery_region_requests_with_lookup(&[ocr2_page], &|key| {
        if key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV {
            Some("profile-risk-window-adaptive".to_string())
        } else {
            (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "0".to_string())
        }
    });

    assert_eq!(regions.len(), 2);
    assert_eq!(
        regions
            .iter()
            .map(|region| region.region_index)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        regions
            .iter()
            .map(|region| region.reading_order_key.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("000001.000040"), Some("000001.000060")]
    );
    assert_close(regions[0].region_box.left, 110.16);
    assert_close(regions[0].region_box.bottom, 451.44);
    assert_close(regions[0].region_box.right, 501.84);
    assert_close(regions[0].region_box.top, 665.28);
    assert_close(regions[1].region_box.bottom, 237.6);
    assert_close(regions[1].region_box.top, 451.44);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn automatic_ocr2_recovery_region_requests_uses_structural_risk_for_adaptive_slices() {
    let mut low_complexity_neighbor = sample_ocr_input(1, "page");
    low_complexity_neighbor.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    low_complexity_neighbor.point_to_pixel_scale_x = 4.2;
    low_complexity_neighbor.point_to_pixel_scale_y = 4.2;
    let mut structural_risk = sample_ocr_input(2, "page");
    structural_risk.ocr_profile = "deepseek-ocr2-direct-vlm".to_string();
    structural_risk.point_to_pixel_scale_x = 3.0;
    structural_risk.point_to_pixel_scale_y = 3.0;
    let profiles = vec![
        sample_source_page_profile(1, false),
        sample_source_page_profile(2, true),
    ];

    let regions = automatic_ocr2_recovery_region_requests_for_profiles_with_lookup(
        &[low_complexity_neighbor, structural_risk],
        profiles.as_slice(),
        &|key| {
            if key == DOCUMENT_EXTRACT_PDF_OCR2_REGION_PLANNER_ENV {
                Some("profile-risk-window-adaptive".to_string())
            } else {
                (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "0".to_string())
            }
        },
    );

    let page_region_counts = regions.iter().fold(
        std::collections::BTreeMap::<u32, usize>::new(),
        |mut counts, region| {
            *counts.entry(region.page_index).or_default() += 1;
            counts
        },
    );
    assert_eq!(page_region_counts.get(&1), Some(&1));
    assert_eq!(page_region_counts.get(&2), Some(&3));
    assert_eq!(
        regions
            .iter()
            .filter(|region| region.page_index == 2)
            .map(|region| region.reading_order_key.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("000002.000030"),
            Some("000002.000050"),
            Some("000002.000070")
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
fn sample_source_page_profile(
    page_index: u32,
    fast_profile_risk: bool,
) -> xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
    xiuxian_wendao_attachments::pdf::profile::PdfSourcePageProfile {
        page_index,
        content_bytes: 1024,
        operation_count: if fast_profile_risk { 695 } else { 24 },
        text_show_ops: if fast_profile_risk { 195 } else { 10 },
        path_ops: if fast_profile_risk { 73 } else { 8 },
        rectangle_ops: if fast_profile_risk { 2 } else { 0 },
        draw_object_ops: 0,
        estimated_weight: 1,
    }
}
