//! Unified crate-level test policy validation.
//!
//! This module combines structure validation and external-test policy validation
//! into a single reusable entry point for consumer crates.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::external_test::{ExternalTestValidationIssue, validate_external_test_mounts};
use crate::validation::{
    StructureViolation, TestsStructurePolicy, format_violation_report,
    validate_crate_tests_with_policy,
};

const TEST_POLICY_CONFIG_FILE: &str = "tests/xiuxian-testings-rules.toml";

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CrateTestPolicyToml {
    tests: TestsPolicyToml,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TestsPolicyToml {
    allowed_root_files: Vec<AllowedTestEntryToml>,
    allowed_directories: Vec<AllowedTestEntryToml>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AllowedTestEntryToml {
    name: String,
    explanation: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CargoManifestToml {
    test: Vec<CargoTestTargetToml>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CargoTestTargetToml {
    path: String,
}

/// Full test policy report for a crate.
#[derive(Debug, Default)]
pub struct CrateTestPolicyReport {
    /// External test policy issues found under `src/`.
    pub external_test_issues: Vec<ExternalTestValidationIssue>,
    /// Test directory structure violations found under `tests/`.
    pub structure_violations: Vec<StructureViolation>,
}

impl CrateTestPolicyReport {
    /// Returns true when the crate satisfies both test policy layers.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.external_test_issues.is_empty() && self.structure_violations.is_empty()
    }
}

/// A test target that does not mount the shared crate test-policy harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestTargetGateViolation {
    /// The Cargo test target source file that omits the harness.
    pub target_file: PathBuf,
    /// Suggested fix.
    pub suggestion: String,
}

/// A crate source root that does not mount the shared source-side policy harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTestPolicyHarnessViolation {
    /// The crate root source file that omits the harness.
    pub source_file: PathBuf,
    /// Suggested fix.
    pub suggestion: String,
}

/// Full harness report for a crate.
#[derive(Debug, Default)]
pub struct CrateTestPolicyHarnessReport {
    /// Shared crate test-policy report.
    pub policy_report: CrateTestPolicyReport,
    /// Test targets that do not mount the shared harness.
    pub target_gate_violations: Vec<TestTargetGateViolation>,
    /// Source-backed unit-test roots that omit the shared harness.
    pub source_gate_violations: Vec<SourceTestPolicyHarnessViolation>,
}

impl CrateTestPolicyHarnessReport {
    /// Returns true when the harness report is clean.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.policy_report.is_clean()
            && self.target_gate_violations.is_empty()
            && self.source_gate_violations.is_empty()
    }
}

/// Validate the full crate test policy.
#[must_use]
pub fn validate_crate_test_policy(crate_root: &Path) -> CrateTestPolicyReport {
    validate_crate_test_policy_with_structure_policy(crate_root, None)
}

fn validate_crate_test_policy_with_structure_policy(
    crate_root: &Path,
    structure_policy: Option<&TestsStructurePolicy>,
) -> CrateTestPolicyReport {
    CrateTestPolicyReport {
        external_test_issues: validate_external_test_mounts(crate_root),
        structure_violations: validate_crate_tests_with_policy(crate_root, structure_policy),
    }
}

fn normalize_allowed_entries(
    entries: Vec<AllowedTestEntryToml>,
    field_name: &str,
    config_path: &Path,
) -> Result<Vec<String>, String> {
    let mut normalized = Vec::with_capacity(entries.len());

    for entry in entries {
        let name = entry.name.trim().to_string();
        let explanation = entry.explanation.trim().to_string();

        if name.is_empty() {
            return Err(format!(
                "Failed to parse {}: [tests].{field_name} entry is missing name",
                config_path.display()
            ));
        }

        if explanation.is_empty() {
            return Err(format!(
                "Failed to parse {}: [tests].{field_name} entry `{name}` is missing explanation",
                config_path.display()
            ));
        }

        normalized.push(name);
    }

    Ok(normalized)
}

fn load_structure_policy_from_toml(
    crate_root: &Path,
) -> Result<Option<TestsStructurePolicy>, String> {
    let config_path = crate_root.join(TEST_POLICY_CONFIG_FILE);
    if !config_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|error| format!("Failed to read {}: {error}", config_path.display()))?;
    let parsed: CrateTestPolicyToml = toml::from_str(&content)
        .map_err(|error| format!("Failed to parse {}: {error}", config_path.display()))?;

    Ok(Some(TestsStructurePolicy {
        allowed_root_files: normalize_allowed_entries(
            parsed.tests.allowed_root_files,
            "allowed_root_files",
            &config_path,
        )?,
        allowed_directories: normalize_allowed_entries(
            parsed.tests.allowed_directories,
            "allowed_directories",
            &config_path,
        )?,
    }))
}

/// Validate crate test policy with project-level `tests/xiuxian-testings-rules.toml`
/// overrides.
///
/// This reads `{crate_root}/tests/xiuxian-testings-rules.toml` when present.
///
/// # Errors
///
/// Returns an error when the workspace test-policy TOML exists but cannot be read
/// or parsed.
pub fn validate_crate_test_policy_with_workspace_config(
    crate_root: &Path,
) -> Result<CrateTestPolicyReport, String> {
    let structure_policy = load_structure_policy_from_toml(crate_root)?;
    Ok(validate_crate_test_policy_with_structure_policy(
        crate_root,
        structure_policy.as_ref(),
    ))
}

/// Validate only the test directory structure with
/// `tests/xiuxian-testings-rules.toml` overrides.
///
/// # Errors
///
/// Returns an error when the workspace test-policy TOML exists but cannot be read
/// or parsed.
pub fn validate_crate_tests_structure_with_workspace_config(
    crate_root: &Path,
) -> Result<Vec<StructureViolation>, String> {
    let structure_policy = load_structure_policy_from_toml(crate_root)?;
    Ok(validate_crate_tests_with_policy(
        crate_root,
        structure_policy.as_ref(),
    ))
}

fn collect_test_target_files(crate_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut targets = BTreeSet::new();
    let tests_dir = crate_root.join("tests");

    if let Ok(entries) = fs::read_dir(&tests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            {
                targets.insert(path);
            }
        }
    }

    let manifest_path = crate_root.join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Failed to read {}: {error}", manifest_path.display()))?;
    let parsed: CargoManifestToml = toml::from_str(&content)
        .map_err(|error| format!("Failed to parse {}: {error}", manifest_path.display()))?;

    for target in parsed.test {
        let target_path = target.path.trim();
        if target_path.is_empty() {
            continue;
        }
        targets.insert(crate_root.join(target_path));
    }

    Ok(targets.into_iter().collect())
}

fn file_contains_policy_harness(content: &str) -> bool {
    [
        "crate_test_policy_harness!(",
        "crate_test_policy_source_harness!(",
        "assert_crate_test_policy_harness(",
        "assert_crate_test_policy_with_workspace_config(",
        "assert_crate_test_policy(",
    ]
    .iter()
    .any(|needle| content.contains(needle))
}

fn validate_test_target_gate_mounts(
    crate_root: &Path,
) -> Result<Vec<TestTargetGateViolation>, String> {
    let mut violations = Vec::new();

    for target_file in collect_test_target_files(crate_root)? {
        let content = fs::read_to_string(&target_file)
            .map_err(|error| format!("Failed to read {}: {error}", target_file.display()))?;
        if file_contains_policy_harness(&content) {
            continue;
        }

        violations.push(TestTargetGateViolation {
            target_file: target_file.strip_prefix(crate_root).unwrap_or(&target_file).to_path_buf(),
            suggestion: "Add `xiuxian_testing::crate_test_policy_harness!();` near the top of this test target so narrow `cargo test --test <target>` runs still execute the shared crate test-policy gate.".to_string(),
        });
    }

    Ok(violations)
}

fn collect_source_test_mount_files(crate_root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, mounted_sources: &mut BTreeSet<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|error| format!("Failed to read {}: {error}", dir.display()))?;

        for entry in entries {
            let entry = entry
                .map_err(|error| format!("Failed to read {} entry: {error}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                walk(&path, mounted_sources)?;
                continue;
            }

            if !path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
            {
                continue;
            }

            let content = fs::read_to_string(&path)
                .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
            let mut saw_cfg_test = false;
            let mut saw_path_attr = false;

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("//")
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with('*')
                {
                    continue;
                }
                if trimmed.starts_with("#[cfg(") && trimmed.contains("test") {
                    saw_cfg_test = true;
                    continue;
                }
                if saw_cfg_test && trimmed.starts_with("#[path =") {
                    saw_path_attr = true;
                    continue;
                }
                if saw_cfg_test
                    && saw_path_attr
                    && trimmed.starts_with("mod ")
                    && trimmed.ends_with(';')
                {
                    mounted_sources.insert(path.clone());
                    break;
                }
                if !trimmed.is_empty() && !trimmed.starts_with("#[") {
                    saw_cfg_test = false;
                    saw_path_attr = false;
                }
            }
        }

        Ok(())
    }

    let src_dir = crate_root.join("src");
    let mut mounted_sources = BTreeSet::new();

    if src_dir.exists() {
        walk(&src_dir, &mut mounted_sources)?;
    }

    Ok(mounted_sources.into_iter().collect())
}

fn validate_source_test_policy_harness(
    crate_root: &Path,
) -> Result<Vec<SourceTestPolicyHarnessViolation>, String> {
    let mounted_sources = collect_source_test_mount_files(crate_root)?;
    if mounted_sources.is_empty() {
        return Ok(Vec::new());
    }

    let src_dir = crate_root.join("src");
    let host_candidates = [src_dir.join("lib.rs"), src_dir.join("main.rs")]
        .into_iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>();

    for candidate in &host_candidates {
        let content = fs::read_to_string(candidate)
            .map_err(|error| format!("Failed to read {}: {error}", candidate.display()))?;
        if file_contains_policy_harness(&content) {
            return Ok(Vec::new());
        }
    }

    let source_file = host_candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| crate_root.join("src/lib.rs"));
    let relative_path = source_file
        .strip_prefix(crate_root)
        .unwrap_or(&source_file)
        .to_path_buf();

    Ok(vec![SourceTestPolicyHarnessViolation {
        source_file: relative_path,
        suggestion: "Add `xiuxian_testing::crate_test_policy_source_harness!(\"../tests/unit/lib_policy.rs\");` to the crate source root so `cargo test --lib` also runs the shared crate test-policy gate without keeping the gate body inline in src/.".to_string(),
    }])
}

/// Validate the full crate test-policy harness.
///
/// # Errors
///
/// Returns an error when the workspace test-policy TOML or `Cargo.toml` cannot
/// be read or parsed.
pub fn validate_crate_test_policy_harness(
    crate_root: &Path,
) -> Result<CrateTestPolicyHarnessReport, String> {
    let policy_report = validate_crate_test_policy_with_workspace_config(crate_root)?;
    let target_gate_violations = validate_test_target_gate_mounts(crate_root)?;
    let source_gate_violations = validate_source_test_policy_harness(crate_root)?;
    Ok(CrateTestPolicyHarnessReport {
        policy_report,
        target_gate_violations,
        source_gate_violations,
    })
}

/// Format a human-readable crate test-policy harness report.
#[must_use]
pub fn format_crate_test_policy_harness_report(report: &CrateTestPolicyHarnessReport) -> String {
    if report.is_clean() {
        return "✅ Crate test-policy harness is valid.".to_string();
    }

    let mut output = String::new();
    if !report.policy_report.is_clean() {
        output.push_str(&format_crate_test_policy_report(&report.policy_report));
        if !report.target_gate_violations.is_empty() {
            output.push('\n');
        }
    }

    if !report.target_gate_violations.is_empty() {
        let _ = writeln!(output, "Test Target Gate Policy:");
        for violation in &report.target_gate_violations {
            let _ = writeln!(
                output,
                "- {}: {}",
                violation.target_file.display(),
                violation.suggestion
            );
        }
    }

    if !report.source_gate_violations.is_empty() {
        if !output.is_empty() {
            output.push('\n');
        }
        let _ = writeln!(output, "Source Test Gate Policy:");
        for violation in &report.source_gate_violations {
            let _ = writeln!(
                output,
                "- {}: {}",
                violation.source_file.display(),
                violation.suggestion
            );
        }
    }

    output
}

/// Format a human-readable full crate test policy report.
#[must_use]
pub fn format_crate_test_policy_report(report: &CrateTestPolicyReport) -> String {
    if report.is_clean() {
        return "✅ Crate test policy is valid.".to_string();
    }

    let mut output = String::new();
    let _ = writeln!(output, "❌ Crate test policy violations detected.\n");

    if !report.external_test_issues.is_empty() {
        let _ = writeln!(output, "External Test Policy:");
        for issue in &report.external_test_issues {
            let _ = writeln!(output, "- {}", issue.description());
        }
        output.push('\n');
    }

    if !report.structure_violations.is_empty() {
        let _ = writeln!(output, "Test Structure Policy:");
        output.push_str(&format_violation_report(&report.structure_violations));
    }

    output
}

/// Assert that a crate satisfies the shared xiuxian test policy.
///
/// # Panics
///
/// Panics when the crate violates the shared xiuxian test policy. The panic message contains the
/// formatted policy report so callers can see every detected violation.
#[track_caller]
pub fn assert_crate_test_policy(crate_root: &Path) {
    let report = validate_crate_test_policy(crate_root);
    assert!(
        report.is_clean(),
        "{}",
        format_crate_test_policy_report(&report)
    );
}

/// Assert crate test policy using optional project-level
/// `tests/xiuxian-testings-rules.toml` overrides.
///
/// # Panics
///
/// Panics when the TOML config cannot be loaded or when the crate violates the configured policy.
#[track_caller]
pub fn assert_crate_test_policy_with_workspace_config(crate_root: &Path) {
    let report = validate_crate_test_policy_with_workspace_config(crate_root)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        report.is_clean(),
        "{}",
        format_crate_test_policy_report(&report)
    );
}

/// Assert that the crate's test targets mount the shared test-policy harness.
///
/// # Panics
///
/// Panics when the crate violates the shared crate test policy or when any
/// Cargo test target omits the shared harness.
#[track_caller]
pub fn assert_crate_test_policy_harness(crate_root: &Path) {
    let report =
        validate_crate_test_policy_harness(crate_root).unwrap_or_else(|error| panic!("{error}"));
    assert!(
        report.is_clean(),
        "{}",
        format_crate_test_policy_harness_report(&report)
    );
}

/// Assert only test directory structure using `tests/xiuxian-testings-rules.toml`
/// overrides.
///
/// # Panics
///
/// Panics when TOML config cannot be loaded or when test structure violations are found.
#[track_caller]
pub fn assert_crate_tests_structure_with_workspace_config(crate_root: &Path) {
    let violations = validate_crate_tests_structure_with_workspace_config(crate_root)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        violations.is_empty(),
        "{}",
        format_violation_report(&violations)
    );
}

#[cfg(test)]
#[path = "../tests/unit/policy/mod.rs"]
mod tests;
