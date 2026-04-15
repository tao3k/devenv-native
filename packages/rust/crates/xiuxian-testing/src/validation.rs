//! Test directory structure validation.
//!
//! This module provides utilities to validate that test files follow the
//! xiuxian testing conventions, ensuring consistent organization across crates.
//!
//! # Directory Structure Convention
//!
//! ```text
//! tests/
//! ├── scenarios/           # Scenario-based tests (managed by ScenarioFramework)
//! ├── snapshots/           # Insta snapshots (auto-generated)
//! ├── fixtures/            # Test fixtures and data files
//! ├── support/             # Test helper modules
//! ├── unit/                # Unit tests (*.rs, snake_case naming)
//! │   ├── entity.rs
//! │   └── storage.rs
//! ├── integration/         # Integration tests (*.rs, snake_case naming)
//! │   ├── dependency_indexer.rs
//! │   └── link_graph.rs
//! ├── performance/         # Optional performance gates and stress suites
//! ├── scenarios_test.rs    # Scenario test entry point
//! ├── unit_test.rs         # Cargo entry point for tests/unit/*
//! ├── integration_test.rs  # Cargo entry point for tests/integration/*
//! ├── performance_test.rs  # Cargo entry point for tests/performance/*
//! └── xiuxian-testing-gate.rs # Unified test-policy and integration mount gate
//! ```
//!
//! # Naming Conventions
//!
//! - **Unit tests**: `tests/unit/{module}.rs` (e.g., `entity.rs`, `storage.rs`)
//! - **Integration tests**: `tests/integration/{feature}.rs`
//! - **Test entry points**: Explicit root harness or gate files only
//!   (for example `unit_test.rs`, `integration_test.rs`, `performance_test.rs`,
//!   `scenarios_test.rs`, `xiuxian-testing-gate.rs`)
//!
//! # Forbidden Patterns
//!
//! - `tests/test_*.rs` → Move to `tests/unit/` or `tests/integration/`
//! - `tests/*_unit.rs` → Move to `tests/unit/{name}.rs`
//! - `tests/*_integration.rs` → Move to `tests/integration/{name}.rs`
//! - Scattered files in `tests/` root → Organize into subdirectories

use std::fs;
use std::path::{Path, PathBuf};

use syn::Item;

/// Optional structure policy overrides loaded from crate-level configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestsStructurePolicy {
    /// Additional allowed directories directly under `tests/`.
    pub allowed_directories: Vec<String>,
    /// Additional allowed Rust file names directly under `tests/`.
    pub allowed_root_files: Vec<String>,
}

/// Represents a violation of the test directory structure convention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureViolation {
    /// The file or directory that violates the convention.
    pub path: PathBuf,
    /// The type of violation.
    pub kind: ViolationKind,
    /// Suggested fix for the violation.
    pub suggestion: String,
}

/// The type of structure violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationKind {
    /// File uses `test_` prefix in tests root (should be in unit/ or integration/).
    TestPrefixInRoot,
    /// File uses `_unit.rs` suffix in tests root (should be in unit/).
    UnitSuffixInRoot,
    /// File uses `_integration.rs` suffix in tests root (should be in integration/).
    IntegrationSuffixInRoot,
    /// File uses `_py.rs` suffix suggesting Python binding tests.
    PySuffixInRoot,
    /// Scattered test file in root without proper categorization.
    ScatteredTestFile,
    /// Directory not in the allowed list.
    UnexpectedDirectory,
    /// Nested unit-test file has regressed into a monolithic suite.
    BloatedUnitTestFile,
    /// Nested integration-test file has regressed into a monolithic suite.
    BloatedIntegrationTestFile,
}

impl std::fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TestPrefixInRoot => write!(f, "test_ prefix in tests root"),
            Self::UnitSuffixInRoot => write!(f, "_unit.rs suffix in tests root"),
            Self::IntegrationSuffixInRoot => write!(f, "_integration.rs suffix in tests root"),
            Self::PySuffixInRoot => write!(f, "_py.rs suffix in tests root"),
            Self::ScatteredTestFile => write!(f, "scattered test file in root"),
            Self::UnexpectedDirectory => write!(f, "unexpected directory"),
            Self::BloatedUnitTestFile => write!(f, "bloated unit test file"),
            Self::BloatedIntegrationTestFile => write!(f, "bloated integration test file"),
        }
    }
}

/// Allowed directories in tests/ root.
const ALLOWED_DIRS: &[&str] = &[
    "scenarios",
    "snapshots",
    "fixtures",
    "support",
    "unit",
    "integration",
    "performance",
    "common",
];

/// Allowed root file names in tests/ (entry points and explicit gateways).
const ALLOWED_ROOT_FILE_PATTERNS: &[&str] = &[
    "mod.rs",
    "lib.rs",
    "unit_test.rs",
    "integration_test.rs",
    "performance_test.rs",
    "scenarios_test.rs",
    "xiuxian-testing-gate.rs",
];

const MAX_UNIT_TEST_EFFECTIVE_LINES: usize = 260;
const MIN_UNIT_TEST_FUNCTIONS: usize = 8;
const MAX_INTEGRATION_TEST_EFFECTIVE_LINES: usize = 420;
const MIN_INTEGRATION_TEST_FUNCTIONS: usize = 12;

/// Check if a file name matches an allowed root file pattern.
fn is_allowed_root_file(name: &str, policy: Option<&TestsStructurePolicy>) -> bool {
    // Allow explicit root file names and policy overrides only.
    ALLOWED_ROOT_FILE_PATTERNS.contains(&name)
        || policy.is_some_and(|config| config.allowed_root_files.iter().any(|entry| entry == name))
}

/// Check if a directory name is allowed directly under tests/.
fn is_allowed_directory(name: &str, policy: Option<&TestsStructurePolicy>) -> bool {
    ALLOWED_DIRS.contains(&name)
        || policy.is_some_and(|config| config.allowed_directories.iter().any(|entry| entry == name))
}

/// Check if a file name indicates it should be in unit/.
fn is_unit_test_file(name: &str) -> bool {
    name.ends_with("_unit.rs") || name.starts_with("unit_")
}

/// Check if a file name indicates it should be in integration/.
fn is_integration_test_file(name: &str) -> bool {
    name.ends_with("_integration.rs")
        || name.starts_with("integration_")
        || name.contains("_indexer_")
        || name.contains("_debug")
}

/// Validate the structure of a tests/ directory.
///
/// # Arguments
///
/// * `tests_dir` - Path to the tests/ directory to validate.
///
/// # Returns
///
/// A vector of violations found. Empty if the structure is valid.
///
/// # Example
///
/// ```
/// use xiuxian_testing::validation::validate_tests_structure;
/// use std::path::Path;
///
/// let violations = validate_tests_structure(Path::new("tests"));
/// for v in &violations {
///     println!("{}: {} - {}", v.path.display(), v.kind, v.suggestion);
/// }
/// ```
#[must_use]
pub fn validate_tests_structure(tests_dir: &Path) -> Vec<StructureViolation> {
    validate_tests_structure_with_policy(tests_dir, None)
}

/// Validate the structure of a tests/ directory with optional policy overrides.
#[must_use]
pub fn validate_tests_structure_with_policy(
    tests_dir: &Path,
    policy: Option<&TestsStructurePolicy>,
) -> Vec<StructureViolation> {
    let mut violations = Vec::new();

    if !tests_dir.exists() {
        return violations;
    }

    let Ok(entries) = fs::read_dir(tests_dir) else {
        return violations;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // Check if directory is allowed
            if !is_allowed_directory(&name, policy) {
                violations.push(StructureViolation {
                    path: path.clone(),
                    kind: ViolationKind::UnexpectedDirectory,
                    suggestion: format!(
                        "Consider moving '{name}' to a standard location. Only keep it directly under tests/ when a harness cannot own it, and document that reason in tests/xiuxian-testings-rules.toml [tests].allowed_directories"
                    ),
                });
            }
        } else if Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
        {
            // Check .rs file naming

            // Skip allowed root files
            if is_allowed_root_file(&name, policy) {
                continue;
            }

            // Check for test_ prefix (should be in unit/ or integration/)
            if name.starts_with("test_") {
                let base_name = name.strip_prefix("test_").unwrap_or(&name);
                let category = if is_integration_test_file(&name) {
                    "integration"
                } else {
                    "unit"
                };
                let suggested_name = base_name.replace("_integration", "").replace("_unit", "");
                violations.push(StructureViolation {
                    path: path.clone(),
                    kind: ViolationKind::TestPrefixInRoot,
                    suggestion: format!("Move to tests/{category}/{suggested_name}.rs"),
                });
                continue;
            }

            // Check for unit test file patterns
            if is_unit_test_file(&name) {
                let base_name = name
                    .strip_suffix("_unit.rs")
                    .or_else(|| name.strip_prefix("unit_"))
                    .unwrap_or(&name)
                    .strip_suffix(".rs")
                    .unwrap_or(&name);
                violations.push(StructureViolation {
                    path: path.clone(),
                    kind: ViolationKind::UnitSuffixInRoot,
                    suggestion: format!("Move to tests/unit/{base_name}.rs"),
                });
                continue;
            }

            // Check for _integration.rs suffix
            if name.ends_with("_integration.rs") {
                let base_name = name.strip_suffix("_integration.rs").unwrap_or(&name);
                violations.push(StructureViolation {
                    path: path.clone(),
                    kind: ViolationKind::IntegrationSuffixInRoot,
                    suggestion: format!("Move to tests/integration/{base_name}.rs"),
                });
                continue;
            }

            // Check for _py.rs suffix (Python binding tests)
            if name.ends_with("_py.rs") {
                let base_name = name.strip_suffix("_py.rs").unwrap_or(&name);
                violations.push(StructureViolation {
                    path: path.clone(),
                    kind: ViolationKind::PySuffixInRoot,
                    suggestion: format!(
                        "Move to tests/integration/{base_name}_python.rs or tests/unit/{base_name}_python.rs"
                    ),
                });
                continue;
            }

            // Any other root-level Rust test file should be organized or explicitly allowed.
            violations.push(StructureViolation {
                path: path.clone(),
                kind: ViolationKind::ScatteredTestFile,
                suggestion: "Move to tests/unit/, tests/integration/, or tests/performance/ behind an explicit harness entry point. Only allow a root file via tests/xiuxian-testings-rules.toml [tests].allowed_root_files when it must stay at tests/ root, and include an explanation."
                    .to_string(),
            });
        }
    }

    violations
}

/// Validate tests structure for a specific crate.
///
/// # Arguments
///
/// * `crate_path` - Path to the crate root (containing Cargo.toml).
///
/// # Returns
///
/// A vector of violations found in the crate's tests/ directory.
#[must_use]
pub fn validate_crate_tests(crate_path: &Path) -> Vec<StructureViolation> {
    validate_crate_tests_with_policy(crate_path, None)
}

/// Validate tests structure for a specific crate with optional policy overrides.
#[must_use]
pub fn validate_crate_tests_with_policy(
    crate_path: &Path,
    policy: Option<&TestsStructurePolicy>,
) -> Vec<StructureViolation> {
    let tests_dir = crate_path.join("tests");
    let mut violations = validate_tests_structure_with_policy(&tests_dir, policy);
    violations.extend(validate_test_leaf_files(
        crate_path,
        "unit",
        ViolationKind::BloatedUnitTestFile,
        MAX_UNIT_TEST_EFFECTIVE_LINES,
        MIN_UNIT_TEST_FUNCTIONS,
    ));
    violations.extend(validate_test_leaf_files(
        crate_path,
        "integration",
        ViolationKind::BloatedIntegrationTestFile,
        MAX_INTEGRATION_TEST_EFFECTIVE_LINES,
        MIN_INTEGRATION_TEST_FUNCTIONS,
    ));
    violations
}

fn validate_test_leaf_files(
    crate_root: &Path,
    suite_name: &'static str,
    kind: ViolationKind,
    max_effective_lines: usize,
    min_test_functions: usize,
) -> Vec<StructureViolation> {
    let suite_dir = crate_root.join("tests").join(suite_name);
    if !suite_dir.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_test_rust_files(&suite_dir, &mut files);
    files
        .into_iter()
        .filter_map(|path| {
            check_test_leaf_file(
                crate_root,
                &path,
                suite_name,
                kind,
                max_effective_lines,
                min_test_functions,
            )
        })
        .collect()
}

fn collect_test_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_test_rust_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
        {
            files.push(path);
        }
    }
}

fn check_test_leaf_file(
    crate_root: &Path,
    path: &Path,
    suite_name: &'static str,
    kind: ViolationKind,
    max_effective_lines: usize,
    min_test_functions: usize,
) -> Option<StructureViolation> {
    let content = fs::read_to_string(path).ok()?;
    let effective_code_lines = count_effective_code_lines(&content);
    if effective_code_lines < max_effective_lines {
        return None;
    }

    let parsed = syn::parse_file(&content).ok()?;
    let test_functions = count_test_functions(&parsed.items);
    if test_functions < min_test_functions {
        return None;
    }

    let rendered_path = path
        .strip_prefix(crate_root)
        .unwrap_or(path)
        .display()
        .to_string();
    let stem = path.file_stem().and_then(|stem| stem.to_str())?;
    let split_target = path.with_extension("");
    let split_target = split_target
        .strip_prefix(crate_root)
        .unwrap_or(&split_target)
        .display()
        .to_string();
    let suite_label = format!("post-harness {suite_name} tree");

    Some(StructureViolation {
        path: path.to_path_buf(),
        kind,
        suggestion: format!(
            "`{rendered_path}` carries {effective_code_lines} effective code lines across {test_functions} test functions. Split it into a folder-first {suite_name} suite such as `{split_target}/mod.rs` plus focused leaves (`{stem}` helpers, config, or rule-family shards) so the {suite_label} stays navigable under the testing-gate harness."
        ),
    })
}

fn count_effective_code_lines(text: &str) -> usize {
    text.lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("//")
                && !line.starts_with("/*")
                && !line.starts_with('*')
                && !line.starts_with("*/")
                && !line.starts_with("#[")
                && !line.starts_with("#![")
        })
        .count()
}

fn count_test_functions(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| match item {
            Item::Fn(item_fn) => usize::from(item_fn.attrs.iter().any(|attr| {
                attr.path()
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "test")
            })),
            Item::Mod(item_mod) => item_mod
                .content
                .as_ref()
                .map_or(0, |(_, nested_items)| count_test_functions(nested_items)),
            _ => 0,
        })
        .sum()
}

/// Get a summary report of violations.
///
/// # Arguments
///
/// * `violations` - List of violations to summarize.
///
/// # Returns
///
/// A human-readable summary string.
#[must_use]
pub fn format_violation_report(violations: &[StructureViolation]) -> String {
    use std::fmt::Write;

    if violations.is_empty() {
        return "✅ No violations found. Tests structure follows conventions.".to_string();
    }

    let mut report = String::new();
    let _ = write!(
        report,
        "❌ Found {} test structure violation(s):\n\n",
        violations.len()
    );

    for (i, v) in violations.iter().enumerate() {
        let _ = write!(
            report,
            "{}. {} ({})\n   💡 {}\n\n",
            i + 1,
            v.path.display(),
            v.kind,
            v.suggestion
        );
    }

    report.push_str(
        "\n📖 See: packages/rust/crates/xiuxian-testing/src/validation.rs for conventions\n",
    );

    report
}

#[cfg(test)]
#[path = "../tests/unit/validation.rs"]
mod tests;
