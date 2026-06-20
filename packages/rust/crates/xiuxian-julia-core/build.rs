//! Julia crate build-time project harness gate.

use xiuxian_rust_workspace_harness::prelude::{
    RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
    RustVerificationRequirement, RustVerificationTaskContract, RustVerificationTaskKind,
};

fn main() {
    xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure(
        |config| {
            config
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/lib.rs", [RustOwnerResponsibility::PublicApi])
            .with_rationale("crate root owns the public package API for build-time verification"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new("src/polyglot/", [RustOwnerResponsibility::PublicApi])
            .with_task_kinds([RustVerificationTaskKind::Regression])
            .with_task_contract(
                RustVerificationTaskKind::Regression,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::AfterUnitTestsPass,
                    "Regression check must exercise the Julia polyglot readiness bridge",
                    [
                        RustVerificationRequirement::new(
                            "command",
                            "cargo test -p xiuxian-julia-core --lib polyglot",
                        ),
                        RustVerificationRequirement::new(
                            "target",
                            "lib unit tests mounted from tests/unit/polyglot/",
                        ),
                        RustVerificationRequirement::new(
                            "coverage",
                            "profile refs, manifest refs, readiness evidence, admission, and snapshots",
                        ),
                    ],
                ),
            )
            .with_rationale(
                "Julia polyglot bridge owns readiness evidence projections for the orchestrator chain",
            ),
        )
        },
    );
}
