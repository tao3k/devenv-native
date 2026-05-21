//! Public lint contracts and entrypoints for BPMN and DMN sources.

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::dmn_model_api::DmnSourceFile;
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

/// One byte span inside a source document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LintSourceSpan {
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset.
    pub end: usize,
}

impl LintSourceSpan {
    /// Creates a byte span.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Source-aware diagnostic metadata for compact LLM renderers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LintSourceDiagnostic {
    /// Source identifier used by the lint input.
    pub source_id: String,
    /// Primary byte span to highlight.
    pub span: LintSourceSpan,
    /// Label attached to the highlighted span.
    pub label: String,
    /// Compact repair hint suitable for LLM observations.
    pub help: String,
}

impl LintSourceDiagnostic {
    /// Creates source-aware diagnostic metadata.
    #[must_use]
    pub fn new(
        source_id: impl Into<String>,
        span: LintSourceSpan,
        label: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            span,
            label: label.into(),
            help: help.into(),
        }
    }
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
    /// Machine-readable repair plan for LLM and tool consumers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_repair: Option<Value>,
    /// Optional source-aware diagnostic metadata for compact renderers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_diagnostic: Option<LintSourceDiagnostic>,
    /// Structured evidence extracted from the parse failure.
    pub evidence: Value,
}

/// Input for constructing one structured lint issue.
#[derive(Debug, Clone, PartialEq)]
pub struct LintIssueInput {
    /// Stable machine-readable issue code.
    pub code: String,
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
    pub fn new(input: LintIssueInput) -> Self {
        Self::from_parts(
            input.code,
            input.title,
            input.summary,
            input.why_it_failed,
            input.repair_guidance,
            input.llm_fix_prompt,
            input.evidence,
        )
    }

    pub(crate) fn from_parts(
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
            structured_repair: None,
            source_diagnostic: None,
            evidence,
        }
    }

    /// Attaches a machine-readable repair plan.
    #[must_use]
    pub fn with_structured_repair(mut self, structured_repair: Value) -> Self {
        self.structured_repair = Some(structured_repair);
        self
    }

    /// Attaches source-aware diagnostic metadata.
    #[must_use]
    pub fn with_source_diagnostic(mut self, source_diagnostic: LintSourceDiagnostic) -> Self {
        self.source_diagnostic = Some(source_diagnostic);
        self
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

/// Lints one BPMN source and returns an LLM-friendly blocking report.
#[must_use]
pub fn lint_bpmn_source(source: &BpmnSourceFile) -> LintReport {
    crate::lint::lint_bpmn_source_impl(source)
}

/// Lints one DMN source and returns an LLM-friendly blocking report.
#[must_use]
pub fn lint_dmn_source(source: &DmnSourceFile) -> LintReport {
    crate::lint::lint_dmn_source_impl(source)
}
