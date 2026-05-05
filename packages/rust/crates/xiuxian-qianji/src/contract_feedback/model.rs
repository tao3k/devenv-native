//! Qianji-owned contract-feedback data model.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level execution mode for one contract-feedback run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractExecutionMode {
    /// Fail according to configured severities.
    Strict,
    /// Keep advisory output without failing by default.
    Advisory,
    /// Retain rich experimental evidence for research-oriented runs.
    Research,
}

/// Severity assigned to one contract finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// Informational finding.
    Info,
    /// Warning-level finding.
    Warning,
    /// Error-level finding.
    Error,
    /// Critical finding.
    Critical,
}

/// Evaluation mode that produced one finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingMode {
    /// Produced by deterministic logic.
    Deterministic,
    /// Produced by an advisory audit lane.
    Advisory,
    /// Produced during exploratory or research execution.
    Research,
}

/// Confidence attached to a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingConfidence {
    /// High-confidence finding.
    High,
    /// Medium-confidence finding.
    Medium,
    /// Low-confidence finding.
    Low,
}

/// Artifact category collected before rule evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    /// Source-code artifact.
    SourceFile,
    /// `OpenAPI` or related API contract artifact.
    OpenApiDocument,
    /// Human-authored documentation artifact.
    Documentation,
    /// Runtime trace or event-stream artifact.
    RuntimeTrace,
    /// Scenario or snapshot artifact.
    ScenarioSnapshot,
    /// Derived invariant or summary artifact.
    DerivedArtifact,
}

/// Evidence category attached to a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    /// File and span evidence from source code.
    SourceSpan,
    /// Evidence tied to a node inside an `OpenAPI` or schema document.
    OpenApiNode,
    /// Evidence tied to a prose documentation section.
    DocSection,
    /// Evidence captured from runtime execution.
    RuntimeTrace,
    /// Evidence tied to a scenario or snapshot.
    ScenarioSnapshot,
    /// Evidence derived from a synthesized invariant or aggregate artifact.
    DerivedInvariant,
}

/// Collection-time context for one contract-feedback suite run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CollectionContext {
    /// Stable suite identifier.
    pub suite_id: String,
    /// Optional crate name under evaluation.
    pub crate_name: Option<String>,
    /// Optional workspace root for path normalization.
    pub workspace_root: Option<PathBuf>,
    /// Caller-provided labels that help downstream reporting.
    pub labels: BTreeMap<String, String>,
}

/// One collected artifact passed to rule packs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectedArtifact {
    /// Stable artifact identifier.
    pub id: String,
    /// Artifact category.
    pub kind: ArtifactKind,
    /// Optional backing path on disk.
    pub path: Option<PathBuf>,
    /// Structured artifact payload.
    pub content: Value,
    /// Additional labels for filtering or reporting.
    pub labels: BTreeMap<String, String>,
}

/// Aggregated artifact set for one suite run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CollectedArtifacts {
    /// Collected artifact list.
    pub artifacts: Vec<CollectedArtifact>,
    /// Shared metadata for the collection run.
    pub metadata: BTreeMap<String, String>,
}

impl CollectedArtifacts {
    /// Append one artifact to the collection.
    pub fn push(&mut self, artifact: CollectedArtifact) {
        self.artifacts.push(artifact);
    }

    /// Return whether the collection contains no artifacts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    /// Return the number of collected artifacts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }
}

/// Positive and negative examples attached to one finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FindingExamples {
    /// Example inputs or snippets that satisfy the contract.
    pub good: Vec<String>,
    /// Example inputs or snippets that violate the contract.
    pub bad: Vec<String>,
}

/// One evidence entry attached to a finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingEvidence {
    /// Evidence category.
    pub kind: EvidenceKind,
    /// Optional path for the evidence source.
    pub path: Option<PathBuf>,
    /// Optional stable locator inside the evidence source.
    pub locator: Option<String>,
    /// Human-readable evidence message.
    pub message: String,
}

/// Normalized finding emitted by deterministic or advisory evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractFinding {
    /// Stable rule identifier.
    pub rule_id: String,
    /// Rule-pack identifier that emitted the finding.
    pub pack_id: String,
    /// Finding severity.
    pub severity: FindingSeverity,
    /// Finding production mode.
    pub mode: FindingMode,
    /// Confidence assigned to the finding.
    pub confidence: FindingConfidence,
    /// Advisory role identifiers that contributed to the finding.
    pub advisory_role_ids: Vec<String>,
    /// Trace identifiers linked to advisory execution.
    pub trace_ids: Vec<String>,
    /// Short title for reports.
    pub title: String,
    /// One-paragraph summary.
    pub summary: String,
    /// Why the finding matters.
    pub why_it_matters: String,
    /// Actionable remediation guidance.
    pub remediation: String,
    /// Evidence entries.
    pub evidence: Vec<FindingEvidence>,
    /// Positive and negative examples.
    pub examples: FindingExamples,
    /// Additional labels for grouping and export.
    pub labels: BTreeMap<String, String>,
}

impl ContractFinding {
    /// Create a new finding with empty evidence, examples, and labels.
    #[must_use]
    pub fn new(
        rule_id: impl Into<String>,
        pack_id: impl Into<String>,
        severity: FindingSeverity,
        mode: FindingMode,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            pack_id: pack_id.into(),
            severity,
            mode,
            confidence: FindingConfidence::High,
            advisory_role_ids: Vec::new(),
            trace_ids: Vec::new(),
            title: title.into(),
            summary: summary.into(),
            why_it_matters: String::new(),
            remediation: String::new(),
            evidence: Vec::new(),
            examples: FindingExamples::default(),
            labels: BTreeMap::new(),
        }
    }
}

/// Aggregated finding counts for one contract report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContractStats {
    /// Total number of findings.
    pub total: usize,
    /// Informational findings.
    pub info: usize,
    /// Warning findings.
    pub warning: usize,
    /// Error findings.
    pub error: usize,
    /// Critical findings.
    pub critical: usize,
    /// Deterministic findings.
    pub deterministic: usize,
    /// Advisory findings.
    pub advisory: usize,
    /// Research findings.
    pub research: usize,
}

impl ContractStats {
    /// Build stats from a slice of findings.
    #[must_use]
    pub fn from_findings(findings: &[ContractFinding]) -> Self {
        let mut stats = Self::default();

        for finding in findings {
            stats.total += 1;
            match finding.severity {
                FindingSeverity::Info => stats.info += 1,
                FindingSeverity::Warning => stats.warning += 1,
                FindingSeverity::Error => stats.error += 1,
                FindingSeverity::Critical => stats.critical += 1,
            }

            match finding.mode {
                FindingMode::Deterministic => stats.deterministic += 1,
                FindingMode::Advisory => stats.advisory += 1,
                FindingMode::Research => stats.research += 1,
            }
        }

        stats
    }
}

/// Report produced by one contract-feedback suite run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractReport {
    /// Stable suite identifier.
    pub suite_id: String,
    /// Top-level execution mode.
    pub execution_mode: ContractExecutionMode,
    /// RFC 3339 or equivalent timestamp chosen by the caller.
    pub generated_at: String,
    /// Findings emitted by the suite.
    pub findings: Vec<ContractFinding>,
    /// Aggregated stats derived from the findings.
    pub stats: ContractStats,
}

impl ContractReport {
    /// Build a report and derive stats from the provided findings.
    #[must_use]
    pub fn from_findings(
        suite_id: impl Into<String>,
        execution_mode: ContractExecutionMode,
        generated_at: impl Into<String>,
        findings: Vec<ContractFinding>,
    ) -> Self {
        let stats = ContractStats::from_findings(&findings);
        Self {
            suite_id: suite_id.into(),
            execution_mode,
            generated_at: generated_at.into(),
            findings,
            stats,
        }
    }
}
