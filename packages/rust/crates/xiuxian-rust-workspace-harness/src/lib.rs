//! Workspace-owned Rust harness policy for xiuxian member crates.

use std::path::{Path, PathBuf};

use rust_lang_project_harness::{
    RustHarnessConfig, RustHarnessReport, RustProjectHarnessDependencyBaseline,
    RustProjectHarnessDownstreamPolicy, RustProjectHarnessWorkspaceEvidenceGraphMemberInput,
    RustProjectHarnessWorkspaceEvidenceGraphReceipt, RustProjectHarnessWorkspacePolicy,
};

pub use rust_lang_project_harness::{
    RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
    RustVerificationRequirement, RustVerificationTaskContract, RustVerificationTaskKind,
};

/// Human-readable label used in workspace evidence graph receipts.
pub const XIUXIAN_WORKSPACE_LABEL: &str = "xiuxian-artisan-workshop";

/// The rust harness git revision pinned by the xiuxian workspace policy.
pub const XIUXIAN_RUST_HARNESS_REV: &str = "f90dc81be0e9cdac6de4ec9378d57d78eaf6b1eb";

/// The rust harness crate version expected at [`XIUXIAN_RUST_HARNESS_REV`].
pub const XIUXIAN_RUST_HARNESS_VERSION: &str = "0.1.2";

/// Shared build-gate explanation for currently advisory cargo-check findings.
pub const XIUXIAN_BUILD_GATE_ADVICE_ALLOW_EXPLANATION: &str = "The workspace rs-harness gate is active; existing advisory findings remain visible through rs-harness check while workspace-owned cleanup continues.";

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

/// Assert the legacy-compatible member build gate from `CARGO_MANIFEST_DIR`.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
pub fn assert_member_build_gate_from_env() -> RustHarnessReport {
    assert_member_harness_build_gate_from_env()
}

/// Assert the legacy-compatible member harness build gate from `CARGO_MANIFEST_DIR`.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
pub fn assert_member_harness_build_gate_from_env() -> RustHarnessReport {
    assert_member_harness_build_gate_from_env_with_configure(|config| config)
}

/// Assert the member build gate with crate-local config extensions.
///
/// Use this when a member crate needs verification profile hints while still
/// inheriting the workspace-owned baseline policy.
///
/// # Panics
///
/// Panics when the rust project harness cargo-check gate fails for the member
/// crate being built.
#[track_caller]
pub fn assert_member_build_gate_from_env_with_configure<F>(configure: F) -> RustHarnessReport
where
    F: FnOnce(RustHarnessConfig) -> RustHarnessConfig,
{
    assert_member_harness_build_gate_from_env_with_configure(configure)
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

/// Assert the stricter downstream policy gate from `CARGO_MANIFEST_DIR`.
///
/// This includes semantic verification coverage and dependency-baseline checks.
/// It is intentionally exposed as an opt-in entrypoint while the workspace
/// migrates package gates from the legacy-compatible cargo-check gate.
///
/// # Panics
///
/// Panics when `CARGO_MANIFEST_DIR` is missing, when the cargo-check policy
/// gate fails, when semantic verification coverage is incomplete, or when the
/// dependency baseline does not match the workspace lockfile.
#[track_caller]
pub fn assert_member_downstream_policy_from_env() -> RustHarnessReport {
    let policy = xiuxian_member_policy(cargo_package_name_from_env());
    rust_lang_project_harness::assert_rust_project_harness_downstream_policy_from_env(&policy)
}

/// Build a workspace evidence graph receipt for selected member crates.
///
/// # Errors
///
/// Returns an error when rust harness verification planning fails for any
/// supplied member crate root.
pub fn xiuxian_workspace_evidence_graph_receipt<I, S, P>(
    workspace_root: &Path,
    members: I,
) -> Result<RustProjectHarnessWorkspaceEvidenceGraphReceipt, String>
where
    I: IntoIterator<Item = (S, P)>,
    S: Into<String>,
    P: Into<PathBuf>,
{
    let policy = xiuxian_workspace_policy();
    let member_inputs = members.into_iter().map(|(crate_label, project_root)| {
        let crate_label = crate_label.into();
        let member_policy = policy.member_crate(crate_label.clone());
        RustProjectHarnessWorkspaceEvidenceGraphMemberInput::new(
            crate_label,
            project_root,
            member_policy,
        )
    });

    rust_lang_project_harness::rust_project_harness_workspace_evidence_graph_receipt(
        workspace_root,
        XIUXIAN_WORKSPACE_LABEL,
        member_inputs,
    )
}

/// Render a workspace evidence graph receipt as compact JSON.
///
/// # Errors
///
/// Returns an error when the rust harness receipt cannot be serialized.
pub fn render_xiuxian_workspace_evidence_graph_receipt_json(
    receipt: &RustProjectHarnessWorkspaceEvidenceGraphReceipt,
) -> Result<String, String> {
    rust_lang_project_harness::render_rust_project_harness_workspace_evidence_graph_receipt_json(
        receipt,
    )
    .map_err(|error| error.to_string())
}

fn cargo_package_name_from_env() -> String {
    match std::env::var("CARGO_PKG_NAME") {
        Ok(crate_label) if !crate_label.is_empty() => crate_label,
        Ok(_) | Err(_) => String::from("unknown-cargo-package"),
    }
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

    #[test]
    fn workspace_evidence_graph_projects_member_policy_loop() {
        let workspace_root = workspace_root_for_tests();
        let receipt = match xiuxian_workspace_evidence_graph_receipt(
            &workspace_root,
            [
                (
                    "xiuxian-config-core",
                    workspace_root.join("packages/rust/crates/xiuxian-config-core"),
                ),
                (
                    "xiuxian-polyglot-orchestrator",
                    workspace_root.join("packages/rust/crates/xiuxian-polyglot-orchestrator"),
                ),
            ],
        ) {
            Ok(receipt) => receipt,
            Err(error) => panic!("workspace evidence graph receipt failed: {error}"),
        };

        assert_eq!(
            receipt.schema_id,
            rust_lang_project_harness::RUST_PROJECT_HARNESS_WORKSPACE_EVIDENCE_GRAPH_RECEIPT_SCHEMA_ID
        );
        assert_eq!(receipt.workspace_label, XIUXIAN_WORKSPACE_LABEL);
        assert_eq!(receipt.summary.member_crate_count, 2);
        assert_eq!(receipt.summary.dependency_baseline_package_count, 2);
        assert!(!receipt.nodes.is_empty());
        assert!(!receipt.edges.is_empty());
        assert!(
            receipt
                .trust_loop_steps
                .iter()
                .any(|step| step.key == "workspace_policy")
        );

        let rendered = match render_xiuxian_workspace_evidence_graph_receipt_json(&receipt) {
            Ok(rendered) => rendered,
            Err(error) => panic!("workspace evidence graph json render failed: {error}"),
        };
        assert!(rendered.contains(XIUXIAN_WORKSPACE_LABEL));
        assert!(rendered.contains(XIUXIAN_RUST_HARNESS_REV));
    }

    fn workspace_root_for_tests() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for candidate in manifest_dir.ancestors() {
            if candidate.join("Cargo.lock").is_file() && candidate.join("Cargo.toml").is_file() {
                return candidate.to_path_buf();
            }
        }
        panic!(
            "could not find workspace root from {}",
            manifest_dir.display()
        );
    }
}
