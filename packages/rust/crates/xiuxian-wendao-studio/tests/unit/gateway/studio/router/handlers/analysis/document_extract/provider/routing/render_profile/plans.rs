#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_profile_plan_keeps_risk_window_accurate() {
    let inputs = (0..6)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..6)
        .map(|page_index| sample_source_page_profile(page_index, page_index == 2))
        .collect::<Vec<_>>();

    let planned = apply_hybrid_page_ocr_profile_plan_for_profiles(inputs, profiles.as_slice());
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-fast-text-ocr",
            "docling-compatible-page-ocr-v1",
            "docling-compatible-page-ocr-v1",
            "docling-compatible-page-ocr-v1",
            "docling-fast-text-ocr",
            "docling-fast-text-ocr",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_profile_plan_keeps_risk_window_accurate() {
    let inputs = (0..6)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..6)
        .map(|page_index| sample_source_page_profile(page_index, page_index == 2))
        .collect::<Vec<_>>();

    let planned =
        apply_hybrid_page_hosted_vlm_profile_plan_for_profiles(inputs, profiles.as_slice());
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-fast-text-ocr",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-fast-text-ocr",
            "docling-fast-text-ocr",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_backend_text_plan_keeps_topup_pages_fast_text() {
    let inputs = (0..7)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..7)
        .map(|page_index| {
            let mut profile = sample_source_page_profile(page_index, page_index == 3);
            if page_index == 1 {
                profile.operation_count = 700;
                profile.text_show_ops = 340;
            }
            profile
        })
        .collect::<Vec<_>>();

    let planned = apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles(
        inputs,
        profiles.as_slice(),
    );
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-backend-text-ocr-v1",
            "docling-fast-text-ocr",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-backend-text-ocr-v1",
            "docling-backend-text-ocr-v1",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_backend_text_plan_can_disable_topup_pages() {
    let inputs = (0..7)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..7)
        .map(|page_index| {
            let mut profile = sample_source_page_profile(page_index, page_index == 3);
            if page_index == 1 {
                profile.operation_count = 700;
                profile.text_show_ops = 340;
            }
            profile
        })
        .collect::<Vec<_>>();

    let planned = apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup(
        inputs,
        profiles.as_slice(),
        &|key| (key == DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV).then(|| "disabled".to_string()),
    );
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-backend-text-ocr-v1",
            "docling-backend-text-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-backend-text-ocr-v1",
            "docling-backend-text-ocr-v1",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr2_backend_text_plan_can_host_topup_pages_on_vlm() {
    let inputs = (0..7)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = (0..7)
        .map(|page_index| {
            let mut profile = sample_source_page_profile(page_index, page_index == 3);
            if page_index == 1 {
                profile.operation_count = 700;
                profile.text_show_ops = 340;
            }
            profile
        })
        .collect::<Vec<_>>();

    let planned = apply_hybrid_page_hosted_vlm_backend_text_profile_plan_for_profiles_with_lookup(
        inputs,
        profiles.as_slice(),
        &|key| {
            (key == DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP_ENV).then(|| "hosted-vlm".to_string())
        },
    );
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();
    let ocr_engines = planned
        .iter()
        .map(|input| input.ocr_engine.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-backend-text-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-backend-text-ocr-v1",
            "docling-backend-text-ocr-v1",
        ]
    );
    assert_eq!(
        ocr_engines,
        vec![
            "docling-backend-text-ocr",
            "hosted-vlm-topup-ocr",
            "hosted-vlm-direct-ocr",
            "hosted-vlm-direct-ocr",
            "hosted-vlm-direct-ocr",
            "docling-backend-text-ocr",
            "docling-backend-text-ocr",
        ]
    );
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn docling_structure_recovery_keeps_structure_pages_off_backend_text() {
    let inputs = (0..4)
        .map(|page_index| sample_ocr_input(page_index, "page"))
        .collect::<Vec<_>>();
    let profiles = vec![
        sample_source_page_profile_with_counts(0, 10, 0, 0, 1, 24, 1024),
        sample_source_page_profile_with_counts(1, 20, 0, 0, 0, 30, 1024),
        sample_source_page_profile_with_counts(2, 180, 80, 0, 0, 720, 1024),
        sample_source_page_profile_with_counts(3, 360, 0, 0, 0, 700, 1024),
    ];

    let planned = apply_hybrid_page_docling_structure_recovery_profile_plan_for_profiles(
        inputs,
        profiles.as_slice(),
    );
    let ocr_profiles = planned
        .iter()
        .map(|input| input.ocr_profile.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ocr_profiles,
        vec![
            "docling-compatible-page-ocr-v1",
            "docling-backend-text-ocr-v1",
            "hosted-vlm-direct-ocr-v1",
            "docling-fast-text-ocr",
        ]
    );
}
