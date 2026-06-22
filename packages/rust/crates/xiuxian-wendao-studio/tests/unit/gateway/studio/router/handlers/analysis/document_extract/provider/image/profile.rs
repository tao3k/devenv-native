use xiuxian_wendao_server::transport::{
    DOCUMENT_EXTRACT_FAST_TEXT_PROFILE, DOCUMENT_EXTRACT_FULL_PROFILE,
    DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE, DocumentExtractMode,
};

use super::{
    gateway_document_extract_mode_for_source, gateway_document_extract_profile_for_source,
};

#[test]
fn auto_mode_keeps_image_extraction_on_sync_route() {
    assert_eq!(
        gateway_document_extract_mode_for_source("/tmp/scan.PNG"),
        DocumentExtractMode::Sync
    );
}

#[test]
fn full_profile_image_source_uses_hosted_vlm_image_profile() {
    assert_eq!(
        gateway_document_extract_profile_for_source("/tmp/scan.PNG", DOCUMENT_EXTRACT_FULL_PROFILE),
        DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
    );
}

#[test]
fn explicit_non_full_profile_is_not_rewritten_for_image_source() {
    assert_eq!(
        gateway_document_extract_profile_for_source(
            "/tmp/scan.png",
            DOCUMENT_EXTRACT_FAST_TEXT_PROFILE,
        ),
        DOCUMENT_EXTRACT_FAST_TEXT_PROFILE
    );
}

#[test]
fn non_image_full_profile_stays_full() {
    assert_eq!(
        gateway_document_extract_profile_for_source(
            "/tmp/report.pdf",
            DOCUMENT_EXTRACT_FULL_PROFILE
        ),
        DOCUMENT_EXTRACT_FULL_PROFILE
    );
}
