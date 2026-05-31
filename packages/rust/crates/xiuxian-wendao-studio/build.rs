//! Studio build-time project harness gate.

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config()
        .with_verification_profile_hint(rust_lang_project_harness::RustVerificationProfileHint::new(
            "src/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/capacity.rs",
            [
                rust_lang_project_harness::RustOwnerResponsibility::LatencySensitive,
                rust_lang_project_harness::RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_profile_hint(rust_lang_project_harness::RustVerificationProfileHint::new(
            "src/studio/router/handlers/analysis/document_extract/provider/transport.rs",
            [
                rust_lang_project_harness::RustOwnerResponsibility::ExternalDependency,
                rust_lang_project_harness::RustOwnerResponsibility::LatencySensitive,
            ],
        ))
        .with_cargo_check_advice_allow_explanation(
            "Studio still exposes several stable Flight provider trait surfaces inherited from xiuxian-wendao-server; typed request migration is tracked separately while build-time harness warnings remain blocking.",
        );
    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
