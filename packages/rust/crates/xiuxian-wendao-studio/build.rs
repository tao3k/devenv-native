//! Studio build-time project harness gate.

fn main() {
    xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure(
        |config| {
            config
        .with_verification_profile_hint(xiuxian_rust_workspace_harness::RustVerificationProfileHint::new(
            "src/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/capacity.rs",
            [
                xiuxian_rust_workspace_harness::RustOwnerResponsibility::LatencySensitive,
                xiuxian_rust_workspace_harness::RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_profile_hint(xiuxian_rust_workspace_harness::RustVerificationProfileHint::new(
            "src/studio/router/handlers/analysis/document_extract/provider/transport.rs",
            [
                xiuxian_rust_workspace_harness::RustOwnerResponsibility::ExternalDependency,
                xiuxian_rust_workspace_harness::RustOwnerResponsibility::LatencySensitive,
            ],
        ))
        },
    );
}
