//! Shared report model for `qianji-client flowhub`.

use std::path::PathBuf;

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
    /// BPMN source path.
    pub bpmn_source: PathBuf,
    /// BPMN process id declared in the Org property drawer.
    pub bpmn_process_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubValidation {
    pub(crate) flowhub_contract: FlowhubCheckStatus,
    pub(crate) generated_files: FlowhubCheckStatus,
    pub(crate) generated_metadata: FlowhubCheckStatus,
    pub(crate) org_lint: FlowhubCheckStatus,
    pub(crate) diagnostics: Vec<String>,
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
