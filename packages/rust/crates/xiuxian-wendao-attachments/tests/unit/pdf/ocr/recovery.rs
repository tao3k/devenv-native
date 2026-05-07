use std::collections::{BTreeMap, BTreeSet};

use super::{PdfOcrWorkerProfile, build_ocr_shard_inputs, sample_manifest, sample_region_manifest};
use crate::pdf::ocr::{
    PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE,
    downgrade_ocr2_region_parent_page_inputs, merge_ocr2_recovery_region_inputs,
    ocr2_region_parent_page_shards, prepare_ocr2_recovery_region_inputs,
};

#[test]
fn recovery_region_merge_downgrades_parent_page_and_binds_region() -> Result<(), String> {
    let mut page_inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile {
            profile_id: PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string(),
            engine: "hosted-vlm-direct-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
        },
    );
    let region_inputs = build_ocr_shard_inputs(
        &[sample_region_manifest()?],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    let region_pages = BTreeSet::from([3]);

    let merged = merge_ocr2_recovery_region_inputs(
        std::mem::take(&mut page_inputs),
        region_inputs,
        &region_pages,
    )?;

    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0].ocr_profile, PDF_OCR_FAST_TEXT_PROFILE);
    assert_eq!(merged[1].ocr_profile, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE);
    assert_eq!(
        merged[1].parent_shard_element_id,
        merged[0].shard_element_id
    );
    Ok(())
}

#[test]
fn recovery_region_prepare_rejects_missing_parent() -> Result<(), String> {
    let region_inputs = build_ocr_shard_inputs(
        &[sample_region_manifest()?],
        &PdfOcrWorkerProfile::docling_compatible(),
    );

    let error = prepare_ocr2_recovery_region_inputs(&BTreeMap::new(), region_inputs)
        .expect_err("missing parent page should fail");

    assert!(error.contains("has no parent page shard"));
    Ok(())
}

#[test]
fn recovery_region_parent_page_shards_are_keyed_by_page() {
    let mut inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile::docling_compatible(),
    );
    inputs[0].shard_type = "page".to_string();

    let parent_shards = ocr2_region_parent_page_shards(inputs.as_slice());

    assert_eq!(parent_shards.get(&3), Some(&inputs[0].shard_element_id));
}

#[test]
fn recovery_region_parent_downgrade_only_touches_requested_pages() {
    let mut inputs = build_ocr_shard_inputs(
        &[sample_manifest()],
        &PdfOcrWorkerProfile {
            profile_id: PDF_OCR_HOSTED_VLM_DIRECT_PROFILE.to_string(),
            engine: "hosted-vlm-direct-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
        },
    );

    downgrade_ocr2_region_parent_page_inputs(&mut inputs, &BTreeSet::from([9]));
    assert_eq!(inputs[0].ocr_profile, PDF_OCR_HOSTED_VLM_DIRECT_PROFILE);

    downgrade_ocr2_region_parent_page_inputs(&mut inputs, &BTreeSet::from([3]));
    assert_eq!(inputs[0].ocr_profile, PDF_OCR_FAST_TEXT_PROFILE);
}
