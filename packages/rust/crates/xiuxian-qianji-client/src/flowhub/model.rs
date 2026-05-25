//! Shared report model for `qianji-client flowhub`.

use std::path::PathBuf;

use serde::Serialize;

use super::parse::FlowhubAction;

/// Structured report returned by the Qianji client Flowhub command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubCliOutput {
    /// Executed action.
    pub action: FlowhubAction,
    /// Whether all validations passed.
    pub passed: bool,
    /// Human-readable markdown report.
    pub rendered: String,
    /// Paths generated or observed by the command.
    pub generated_paths: Vec<PathBuf>,
    /// Detailed generated file status.
    pub generated_files: Vec<FlowhubGeneratedFile>,
    /// Flowhub Org+BPMN source pairs discovered for list-style commands.
    pub source_pairs: Vec<FlowhubSourcePairSummary>,
}

/// Status for one generated agent tracking file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGeneratedFile {
    /// Generated file path.
    pub path: PathBuf,
    /// True when this command created the file, false when the file already existed.
    pub created: bool,
}

/// Public summary of one Flowhub Org+BPMN source pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubSourcePairSummary {
    /// Scenario id declared in the Org property drawer.
    pub scenario_id: String,
    /// Org source path.
    pub org_source: PathBuf,
    /// SHA-256 of the Org source at registry read time.
    pub org_sha256: String,
    /// BPMN source path.
    pub bpmn_source: PathBuf,
    /// SHA-256 of the BPMN source at registry read time.
    pub bpmn_sha256: String,
    /// BPMN process id declared in the Org property drawer.
    pub bpmn_process_id: String,
}

/// Machine-readable Flowhub scenario registry response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowhubScenarioRegistry {
    /// Registry action label.
    pub action: String,
    /// Whether the registry passed validation.
    pub passed: bool,
    /// Flowhub root used to scan the registry.
    pub flowhub_root: String,
    /// Discovered source pairs.
    pub source_pairs: Vec<FlowhubScenarioRegistrySourcePair>,
    /// Registry validation details.
    pub validation: FlowhubScenarioRegistryValidation,
}

/// Machine-readable summary of one Flowhub Org+BPMN source pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowhubScenarioRegistrySourcePair {
    /// Scenario id declared in the Org property drawer.
    pub scenario_id: String,
    /// Org source path.
    pub org_source: String,
    /// SHA-256 of the Org source at registry read time.
    pub org_sha256: String,
    /// BPMN source path.
    pub bpmn_source: String,
    /// SHA-256 of the BPMN source at registry read time.
    pub bpmn_sha256: String,
    /// BPMN process id declared in the Org property drawer.
    pub bpmn_process_id: String,
}

/// Machine-readable Flowhub registry validation details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowhubScenarioRegistryValidation {
    /// True when the Flowhub Org+BPMN source-pair contract passed.
    pub flowhub_contract_passed: bool,
    /// Registry diagnostics collected while validating the source-pair contract.
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubValidation {
    pub(crate) flowhub_contract: FlowhubCheckStatus,
    pub(crate) generated_files: FlowhubCheckStatus,
    pub(crate) generated_metadata: FlowhubCheckStatus,
    pub(crate) org_lint: FlowhubCheckStatus,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) generated_metadata_failures: Vec<FlowhubGeneratedMetadataFailure>,
}

impl FlowhubValidation {
    pub(crate) fn passed(&self) -> bool {
        self.flowhub_contract.is_passed()
            && self.generated_files.is_passed()
            && self.generated_metadata.is_passed()
            && self.org_lint.is_passed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlowhubCheckStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubGeneratedMetadataFailure {
    pub(crate) path: PathBuf,
    pub(crate) key: String,
    pub(crate) actual: Option<String>,
    pub(crate) expected: String,
}

impl FlowhubCheckStatus {
    pub(crate) fn from_bool(passed: bool) -> Self {
        if passed { Self::Passed } else { Self::Failed }
    }

    pub(crate) fn as_bool(self) -> bool {
        self == Self::Passed
    }

    pub(crate) fn is_passed(self) -> bool {
        self.as_bool()
    }
}
