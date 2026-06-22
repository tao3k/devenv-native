//! Studio build-time project harness gate.

use xiuxian_rust_workspace_harness::prelude::{
    RustOwnerResponsibility, RustVerificationProfileHint,
};

fn main() {
    xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure(
        |config| {
            config
        .with_verification_profile_hint(RustVerificationProfileHint::new(
            "src/studio/router/handlers/analysis/document_extract/pdf_ocr_scheduler/capacity.rs",
            [
                RustOwnerResponsibility::LatencySensitive,
                RustOwnerResponsibility::AvailabilityCritical,
            ],
        ))
        .with_verification_profile_hint(RustVerificationProfileHint::new(
            "src/studio/router/handlers/analysis/document_extract/provider/transport.rs",
            [
                RustOwnerResponsibility::ExternalDependency,
                RustOwnerResponsibility::LatencySensitive,
            ],
        ))
        },
    );
}
