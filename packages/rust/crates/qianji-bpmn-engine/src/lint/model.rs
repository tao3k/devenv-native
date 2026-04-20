//! Shared lint report model with LLM-friendly repair guidance.

use serde_json::Value;

/// Linted document family.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintDomain {
    /// BPMN workflow source.
    Bpmn,
    /// DMN decision source.
    Dmn,
}

/// Severity level for one lint issue.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    /// Blocking issue that prevents safe parser acceptance.
    Error,
}

/// One structured lint issue with repair guidance suitable for LLM-assisted fixes.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LintIssue {
    /// Stable machine-readable issue code.
    pub code: String,
    /// Blocking severity for the current bounded slice.
    pub severity: LintSeverity,
    /// Short title suitable for CLI rendering.
    pub title: String,
    /// Short explanation of what failed.
    pub summary: String,
    /// Why the parser or validator stopped.
    pub why_it_failed: String,
    /// Ordered repair steps that a human or LLM should follow.
    pub repair_guidance: Vec<String>,
    /// One direct editing prompt optimized for LLM-assisted repair.
    pub llm_fix_prompt: String,
    /// Structured evidence extracted from the parse failure.
    pub evidence: Value,
}

impl LintIssue {
    /// Creates one lint issue.
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        title: impl Into<String>,
        summary: impl Into<String>,
        why_it_failed: impl Into<String>,
        repair_guidance: Vec<String>,
        llm_fix_prompt: impl Into<String>,
        evidence: Value,
    ) -> Self {
        Self {
            code: code.into(),
            severity: LintSeverity::Error,
            title: title.into(),
            summary: summary.into(),
            why_it_failed: why_it_failed.into(),
            repair_guidance,
            llm_fix_prompt: llm_fix_prompt.into(),
            evidence,
        }
    }
}

/// Lint report for one BPMN or DMN source.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LintReport {
    /// Document family being linted.
    pub domain: LintDomain,
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Whether the source passed the bounded linter without blocking findings.
    pub ok: bool,
    /// Blocking findings emitted by the bounded linter.
    pub issues: Vec<LintIssue>,
}

impl LintReport {
    /// Creates a passing lint report.
    #[must_use]
    pub fn ok(domain: LintDomain, source_id: impl Into<String>) -> Self {
        Self {
            domain,
            source_id: source_id.into(),
            ok: true,
            issues: Vec::new(),
        }
    }

    /// Creates a failing lint report with one or more issues.
    #[must_use]
    pub fn blocking(
        domain: LintDomain,
        source_id: impl Into<String>,
        issues: Vec<LintIssue>,
    ) -> Self {
        Self {
            domain,
            source_id: source_id.into(),
            ok: false,
            issues,
        }
    }
}
