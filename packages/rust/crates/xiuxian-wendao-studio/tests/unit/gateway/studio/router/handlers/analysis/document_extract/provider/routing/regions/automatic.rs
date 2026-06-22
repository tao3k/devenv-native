use super::{
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV,
    DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV,
    DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV, assert_close,
    automatic_ocr2_recovery_region_requests_for_profiles_with_lookup,
    automatic_ocr2_recovery_region_requests_with_lookup, sample_ocr_input,
};

#[test]
fn automatic_ocr2_recovery_region_requests_build_content_band() {
    let mut fast_page = sample_ocr_input(0, "page");
    fast_page.ocr_profile = "docling-fast-text-ocr".to_string();
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();

    let disabled = automatic_ocr2_recovery_region_requests_with_lookup(
        &[fast_page.clone(), ocr2_page.clone()],
        &|_key| None,
    );
    assert!(disabled.is_empty());

    let regions =
        automatic_ocr2_recovery_region_requests_with_lookup(&[fast_page, ocr2_page], &|key| {
            if key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV {
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

#[test]
fn automatic_ocr2_recovery_region_requests_skip_hosted_topup_pages() {
    let mut hosted_topup_page = sample_ocr_input(1, "page");
    hosted_topup_page.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
    hosted_topup_page.ocr_engine = "hosted-vlm-topup-ocr".to_string();

    let regions =
        automatic_ocr2_recovery_region_requests_with_lookup(&[hosted_topup_page], &|key| {
            if key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV {
                Some("profile-risk-window".to_string())
            } else {
                (key == DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV).then(|| "0".to_string())
            }
        });

    assert!(regions.is_empty());
}

#[test]
fn automatic_ocr2_recovery_region_requests_can_slice_content_band() {
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();

    let regions = automatic_ocr2_recovery_region_requests_with_lookup(&[ocr2_page], &|key| {
        if key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV {
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

#[test]
fn automatic_ocr2_recovery_region_requests_adaptively_splits_large_band() {
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
    ocr2_page.point_to_pixel_scale_x = 4.2;
    ocr2_page.point_to_pixel_scale_y = 4.2;

    let regions = automatic_ocr2_recovery_region_requests_with_lookup(&[ocr2_page], &|key| {
        if key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV {
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

#[test]
fn automatic_ocr2_recovery_region_requests_uses_structural_risk_for_adaptive_slices() {
    let mut low_complexity_neighbor = sample_ocr_input(1, "page");
    low_complexity_neighbor.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
    low_complexity_neighbor.point_to_pixel_scale_x = 4.2;
    low_complexity_neighbor.point_to_pixel_scale_y = 4.2;
    let mut structural_risk = sample_ocr_input(2, "page");
    structural_risk.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
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
            if key == DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV {
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
    assert_eq!(page_region_counts.get(&1), Some(&2));
    assert_eq!(page_region_counts.get(&2), Some(&3));
    assert_eq!(
        regions
            .iter()
            .filter(|region| region.page_index == 1)
            .map(|region| region.reading_order_key.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("000001.000040"), Some("000001.000060")]
    );
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

#[test]
fn automatic_ocr2_recovery_region_requests_uses_configured_patch_sizing() {
    let mut ocr2_page = sample_ocr_input(1, "page");
    ocr2_page.ocr_profile = "hosted-vlm-direct-ocr-v1".to_string();
    ocr2_page.point_to_pixel_scale_x = 4.2;
    ocr2_page.point_to_pixel_scale_y = 4.2;

    let regions =
        automatic_ocr2_recovery_region_requests_with_lookup(&[ocr2_page], &|key| match key {
            DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER_ENV => {
                Some("profile-risk-window-adaptive".to_string())
            }
            DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS_ENV => Some("750000".to_string()),
            DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES_ENV => Some("4".to_string()),
            DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO_ENV => Some("0".to_string()),
            _ => None,
        });

    assert_eq!(regions.len(), 4);
    assert_eq!(
        regions
            .iter()
            .map(|region| region.region_index)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

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
