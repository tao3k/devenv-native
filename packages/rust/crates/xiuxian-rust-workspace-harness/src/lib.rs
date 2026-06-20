//! Workspace-owned Rust harness policy for xiuxian member crates.

use rust_lang_project_harness::{
    RustHarnessConfig, RustHarnessReport, RustProjectHarnessDependencyBaseline,
    RustProjectHarnessDownstreamPolicy, RustProjectHarnessWorkspacePolicy,
};

/// Types intentionally exposed for member `build.rs` policy hints.
pub mod prelude {
    pub use rust_lang_project_harness::{
        RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
        RustVerificationRequirement, RustVerificationTaskContract, RustVerificationTaskKind,
    };
}

/// Human-readable label used in workspace evidence graph receipts.
pub const XIUXIAN_WORKSPACE_LABEL: &str = "xiuxian-artisan-workshop";

/// The rust harness git revision pinned by the xiuxian workspace policy.
pub const XIUXIAN_RUST_HARNESS_REV: &str = "6dea4b1afc46bec0b4b480b9ab99b9d471d045ee";

/// The rust harness crate version expected at [`XIUXIAN_RUST_HARNESS_REV`].
pub const XIUXIAN_RUST_HARNESS_VERSION: &str = "0.1.2";

/// Shared build-gate explanation for advisory cargo-check findings.
pub const XIUXIAN_BUILD_GATE_ADVICE_ALLOW_EXPLANATION: &str = concat!(
    "scope=xiuxian workspace build.rs gates; ",
    "owner=xiuxian-rust-workspace-harness; ",
    "finding_category=advisory project-policy migrations; ",
    "why_safe_now=warnings/errors remain blocking and advisory findings stay visible in harness output; ",
    "cleanup_trigger=promote assert_member_harness_build_gate_from_env to the strict downstream policy gate after transitive rust-harness rev drift is removed"
);

/// Build the shared harness config used by xiuxian member build gates.
#[must_use]
pub fn xiuxian_workspace_harness_config() -> RustHarnessConfig {
    rust_lang_project_harness::default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(XIUXIAN_BUILD_GATE_ADVICE_ALLOW_EXPLANATION)
}

/// Build the shared dependency baseline for workspace evidence receipts.
#[must_use]
pub fn xiuxian_workspace_dependency_baseline() -> RustProjectHarnessDependencyBaseline {
    RustProjectHarnessDependencyBaseline::new().require_git_package(
        "rust-lang-project-harness",
        XIUXIAN_RUST_HARNESS_VERSION,
        format!("rev={XIUXIAN_RUST_HARNESS_REV}"),
    )
}

/// Build the workspace-owned policy used to derive member crate policies.
#[must_use]
pub fn xiuxian_workspace_policy() -> RustProjectHarnessWorkspacePolicy {
    RustProjectHarnessWorkspacePolicy::new(
        XIUXIAN_WORKSPACE_LABEL,
        xiuxian_workspace_harness_config(),
    )
    .with_dependency_baseline(xiuxian_workspace_dependency_baseline())
}

/// Derive a member crate policy from the shared xiuxian workspace policy.
#[must_use]
pub fn xiuxian_member_policy(crate_label: impl Into<String>) -> RustProjectHarnessDownstreamPolicy {
    xiuxian_workspace_policy().member_crate(crate_label)
}

/// Assert the legacy-compatible member harness build gate from `CARGO_MANIFEST_DIR`.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
#[expect(
    clippy::must_use_candidate,
    reason = "build scripts may rely on the panic side effect without reading the report"
)]
pub fn assert_member_harness_build_gate_from_env() -> RustHarnessReport {
    assert_member_harness_build_gate_from_env_with_configure(|config| config)
}

/// Assert the member harness build gate with crate-local config extensions.
///
/// Use this when a member crate needs verification profile hints while still
/// inheriting the workspace-owned baseline policy.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
pub fn assert_member_harness_build_gate_from_env_with_configure<F>(
    configure: F,
) -> RustHarnessReport
where
    F: FnOnce(RustHarnessConfig) -> RustHarnessConfig,
{
    let config = xiuxian_workspace_harness_config();
    let config = configure(config);
    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_policy_carries_harness_rev_baseline() {
        let policy = xiuxian_workspace_policy();
        let member_policy = policy.member_crate("xiuxian-config-core");
        let packages = match member_policy.dependency_baseline() {
            Some(dependency_baseline) => dependency_baseline.packages(),
            None => panic!("expected rust harness dependency baseline"),
        };

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name(), "rust-lang-project-harness");
        assert_eq!(packages[0].version(), XIUXIAN_RUST_HARNESS_VERSION);
        assert_eq!(
            packages[0].source_contains(),
            format!("rev={XIUXIAN_RUST_HARNESS_REV}")
        );
    }
}
