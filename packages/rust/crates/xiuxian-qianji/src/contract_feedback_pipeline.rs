//! End-to-end contract-feedback pipeline for Qianji-driven suite runs.

#[cfg(feature = "llm")]
use std::sync::Arc;
use std::{collections::BTreeMap, path::PathBuf};

use crate::contract_feedback::{
    AdvisoryAuditExecutor, CollectionContext, ContractFinding, ContractReport, ContractRunConfig,
    ContractSuite, ContractSuiteRunner, EvidenceKind, FindingConfidence, FindingMode,
    FindingSeverity,
};
use anyhow::Result;
use serde_json::{Value, json};
#[cfg(feature = "llm")]
use xiuxian_llm::llm::LlmClient;
#[cfg(feature = "llm")]
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_wendao_core::{
    ContractFindingConfidence, ContractFindingSeverity, ContractKnowledgeBatch,
    ContractKnowledgeDecision, ContractKnowledgeEnvelope, KnowledgeEntry,
    WendaoContractFeedbackAdapter,
};

#[cfg(feature = "llm")]
use crate::executors::{QianjiAdvisoryAuditExecutor, QianjiLlmAdvisoryAuditExecutor};
use crate::sovereign::ContractFeedbackKnowledgeSink;

#[cfg(feature = "llm")]
const DEFAULT_LIVE_FEEDBACK_MODEL: &str = "gpt-5.4-mini";
#[cfg(feature = "llm")]
const DEFAULT_LIVE_FEEDBACK_TEMPERATURE: f32 = 0.1;

/// Output of one contract-feedback execution.
#[derive(Debug, Clone)]
pub struct QianjiContractFeedbackRun {
    /// Contract report produced by the suite runner.
    pub report: ContractReport,
    /// Wendao-ready export batch derived from the report.
    pub knowledge_batch: ContractKnowledgeBatch,
    /// Wendao-native knowledge entries adapted from the batch.
    pub knowledge_entries: Vec<KnowledgeEntry>,
}

impl QianjiContractFeedbackRun {
    /// Build one Qianji contract-feedback output from an existing contract report.
    #[must_use]
    pub fn from_report(report: ContractReport) -> Self {
        let knowledge_batch = wendao_contract_knowledge_batch_from_report(&report);
        let knowledge_entries =
            WendaoContractFeedbackAdapter::knowledge_entries_from_batch(&knowledge_batch);

        Self {
            report,
            knowledge_batch,
            knowledge_entries,
        }
    }
}

fn wendao_contract_knowledge_batch_from_report(report: &ContractReport) -> ContractKnowledgeBatch {
    ContractKnowledgeBatch {
        suite_id: report.suite_id.clone().into(),
        generated_at: report.generated_at.clone(),
        entries: report
            .findings
            .iter()
            .map(|finding| {
                wendao_contract_knowledge_envelope_from_finding(
                    report.suite_id.as_str(),
                    report.generated_at.as_str(),
                    finding,
                )
            })
            .collect(),
    }
}

fn wendao_contract_knowledge_envelope_from_finding(
    suite_id: &str,
    generated_at: &str,
    finding: &ContractFinding,
) -> ContractKnowledgeEnvelope {
    let domain = finding
        .labels
        .get("domain")
        .cloned()
        .unwrap_or_else(|| finding.pack_id.clone());
    let evidence_excerpt = finding
        .evidence
        .first()
        .map(|evidence| evidence.message.clone());
    let source_path = first_source_path(finding);
    let decision = wendao_contract_knowledge_decision_from_severity(finding.severity);

    ContractKnowledgeEnvelope {
        entry_id: build_contract_feedback_entry_id(suite_id, finding).into(),
        suite_id: suite_id.to_string().into(),
        generated_at: generated_at.to_string(),
        rule_id: finding.rule_id.clone().into(),
        pack_id: finding.pack_id.clone().into(),
        domain: domain.clone(),
        severity: wendao_contract_finding_severity(finding.severity),
        decision,
        confidence: wendao_contract_finding_confidence(finding.confidence),
        title: format!("[{}] {}", finding.rule_id, finding.title),
        content: render_contract_feedback_content(finding, evidence_excerpt.as_deref()),
        summary: finding.summary.clone(),
        evidence_excerpt,
        why_it_matters: finding.why_it_matters.clone(),
        remediation: finding.remediation.clone(),
        good_example: finding.examples.good.first().cloned(),
        bad_example: finding.examples.bad.first().cloned(),
        source_path,
        tags: build_contract_feedback_tags(&domain, finding),
        metadata: build_contract_feedback_metadata(
            suite_id,
            generated_at,
            &domain,
            decision,
            finding,
        ),
    }
}

const fn wendao_contract_finding_severity(severity: FindingSeverity) -> ContractFindingSeverity {
    match severity {
        FindingSeverity::Info => ContractFindingSeverity::Info,
        FindingSeverity::Warning => ContractFindingSeverity::Warning,
        FindingSeverity::Error => ContractFindingSeverity::Error,
        FindingSeverity::Critical => ContractFindingSeverity::Critical,
    }
}

const fn wendao_contract_finding_confidence(
    confidence: FindingConfidence,
) -> ContractFindingConfidence {
    match confidence {
        FindingConfidence::High => ContractFindingConfidence::High,
        FindingConfidence::Medium => ContractFindingConfidence::Medium,
        FindingConfidence::Low => ContractFindingConfidence::Low,
    }
}

const fn wendao_contract_knowledge_decision_from_severity(
    severity: FindingSeverity,
) -> ContractKnowledgeDecision {
    match severity {
        FindingSeverity::Info => ContractKnowledgeDecision::Pass,
        FindingSeverity::Warning => ContractKnowledgeDecision::Warn,
        FindingSeverity::Error | FindingSeverity::Critical => ContractKnowledgeDecision::Fail,
    }
}

fn build_contract_feedback_entry_id(suite_id: &str, finding: &ContractFinding) -> String {
    let path_fragment = finding
        .labels
        .get("path")
        .cloned()
        .or_else(|| {
            first_source_path(finding).map(|path| {
                path.to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "::")
            })
        })
        .unwrap_or_else(|| "global".to_string());
    let mode_fragment = finding_mode_label(finding.mode);
    let advisory_role_fragment = advisory_role_fragment(finding);
    format!(
        "{suite_id}::{}::{}::{mode_fragment}::{path_fragment}{advisory_role_fragment}",
        finding.pack_id, finding.rule_id,
    )
}

fn render_contract_feedback_content(
    finding: &ContractFinding,
    evidence_excerpt: Option<&str>,
) -> String {
    let mut sections = vec![
        format!("Summary: {}", finding.summary),
        format!("Why it matters: {}", finding.why_it_matters),
        format!("Remediation: {}", finding.remediation),
    ];

    if let Some(evidence_excerpt) = evidence_excerpt {
        sections.push(format!("Evidence: {evidence_excerpt}"));
    }
    if let Some(example) = finding.examples.good.first() {
        sections.push(format!("Good example: {example}"));
    }
    if let Some(example) = finding.examples.bad.first() {
        sections.push(format!("Bad example: {example}"));
    }

    sections.join("\n")
}

fn build_contract_feedback_tags(domain: &str, finding: &ContractFinding) -> Vec<String> {
    let mut tags = vec![
        "contract_finding".to_string(),
        format!("pack:{}", finding.pack_id),
        format!("rule:{}", finding.rule_id),
        format!("severity:{}", finding_severity_label(finding.severity)),
        format!("mode:{}", finding_mode_label(finding.mode)),
        format!("domain:{domain}"),
    ];

    if let Some(path) = finding.labels.get("path") {
        tags.push(format!("path:{path}"));
    }

    tags.sort();
    tags.dedup();
    tags
}

fn build_contract_feedback_metadata(
    suite_id: &str,
    generated_at: &str,
    domain: &str,
    decision: ContractKnowledgeDecision,
    finding: &ContractFinding,
) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("suite_id".to_string(), json!(suite_id)),
        ("generated_at".to_string(), json!(generated_at)),
        ("domain".to_string(), json!(domain)),
        (
            "decision".to_string(),
            json!(contract_decision_label(decision)),
        ),
        (
            "confidence".to_string(),
            json!(finding_confidence_label(finding.confidence)),
        ),
        (
            "advisory_role_ids".to_string(),
            json!(finding.advisory_role_ids),
        ),
        ("trace_ids".to_string(), json!(finding.trace_ids)),
        ("labels".to_string(), json!(finding.labels)),
        (
            "evidence_kinds".to_string(),
            json!(
                finding
                    .evidence
                    .iter()
                    .map(|evidence| evidence_kind_label(evidence.kind))
                    .collect::<Vec<_>>()
            ),
        ),
    ])
}

fn first_source_path(finding: &ContractFinding) -> Option<PathBuf> {
    finding
        .evidence
        .iter()
        .find_map(|evidence| evidence.path.clone())
        .or_else(|| finding.labels.get("source_path").map(PathBuf::from))
}

const fn contract_decision_label(decision: ContractKnowledgeDecision) -> &'static str {
    match decision {
        ContractKnowledgeDecision::Pass => "pass",
        ContractKnowledgeDecision::Warn => "warn",
        ContractKnowledgeDecision::Fail => "fail",
    }
}

const fn finding_severity_label(severity: FindingSeverity) -> &'static str {
    match severity {
        FindingSeverity::Info => "info",
        FindingSeverity::Warning => "warning",
        FindingSeverity::Error => "error",
        FindingSeverity::Critical => "critical",
    }
}

const fn finding_mode_label(mode: FindingMode) -> &'static str {
    match mode {
        FindingMode::Deterministic => "deterministic",
        FindingMode::Advisory => "advisory",
        FindingMode::Research => "research",
    }
}

fn advisory_role_fragment(finding: &ContractFinding) -> String {
    if finding.mode != FindingMode::Advisory {
        return String::new();
    }

    finding
        .advisory_role_ids
        .first()
        .cloned()
        .or_else(|| finding.labels.get("role_id").cloned())
        .map(|role_id| format!("::role:{}", normalized_contract_fragment(&role_id)))
        .unwrap_or_default()
}

fn normalized_contract_fragment(fragment: &str) -> String {
    let mut normalized = String::with_capacity(fragment.len());
    for character in fragment.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push('_');
        }
    }
    normalized
}

const fn finding_confidence_label(confidence: FindingConfidence) -> &'static str {
    match confidence {
        FindingConfidence::High => "high",
        FindingConfidence::Medium => "medium",
        FindingConfidence::Low => "low",
    }
}

const fn evidence_kind_label(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::SourceSpan => "source_span",
        EvidenceKind::OpenApiNode => "openapi_node",
        EvidenceKind::DocSection => "doc_section",
        EvidenceKind::RuntimeTrace => "runtime_trace",
        EvidenceKind::ScenarioSnapshot => "scenario_snapshot",
        EvidenceKind::DerivedInvariant => "derived_invariant",
    }
}

/// Output of one contract-feedback execution after persistence into a sovereign sink.
#[derive(Debug, Clone)]
pub struct QianjiPersistedContractFeedbackRun {
    /// The original contract-feedback output before persistence.
    pub run: QianjiContractFeedbackRun,
    /// The knowledge entry ids acknowledged by the sink.
    pub persisted_entry_ids: Vec<String>,
}

/// Execute one contract suite and project the result into Wendao-ready knowledge entries.
///
/// # Errors
///
/// Returns an error when the suite runner fails to collect artifacts, evaluate rule packs, or run
/// the advisory executor for a triggered pack.
pub async fn run_contract_feedback_flow(
    suite: &ContractSuite,
    ctx: &CollectionContext,
    config: &ContractRunConfig,
    advisory_executor: &dyn AdvisoryAuditExecutor,
) -> Result<QianjiContractFeedbackRun> {
    let report = ContractSuiteRunner::new(advisory_executor)
        .run(suite, ctx, config)
        .await?;
    Ok(QianjiContractFeedbackRun::from_report(report))
}

/// Persist an existing contract-feedback run through a sovereign knowledge sink.
///
/// # Errors
///
/// Returns an error when the sink fails to persist the generated Wendao-native knowledge entries.
pub async fn persist_contract_feedback_run(
    run: QianjiContractFeedbackRun,
    sink: &dyn ContractFeedbackKnowledgeSink,
) -> Result<QianjiPersistedContractFeedbackRun> {
    let persisted_entry_ids = sink
        .persist_entries(&run.knowledge_entries)
        .await
        .map_err(anyhow::Error::msg)?;

    Ok(QianjiPersistedContractFeedbackRun {
        run,
        persisted_entry_ids,
    })
}

/// Execute one contract suite and persist the resulting knowledge entries through a sovereign sink.
///
/// # Errors
///
/// Returns an error when contract execution fails or when the sink fails to persist the generated
/// knowledge entries.
/// Positional boundary: this compatibility API keeps the established public call shape.
pub async fn run_and_persist_contract_feedback_flow(
    suite: &ContractSuite,
    ctx: &CollectionContext,
    config: &ContractRunConfig,
    advisory_executor: &dyn AdvisoryAuditExecutor,
    sink: &dyn ContractFeedbackKnowledgeSink,
) -> Result<QianjiPersistedContractFeedbackRun> {
    let run = run_contract_feedback_flow(suite, ctx, config, advisory_executor).await?;
    persist_contract_feedback_run(run, sink).await
}

/// Configuration for the `llm`-gated live contract-feedback lane.
#[cfg(feature = "llm")]
#[derive(Debug, Clone, PartialEq)]
pub struct QianjiLiveContractFeedbackOptions {
    /// Model name forwarded to the live advisory executor.
    pub model: String,
    /// Sampling temperature used for role critiques.
    pub temperature: f32,
    /// Optional cognitive-supervision threshold. When set, `ZhenfaPipeline` is enabled.
    pub cognitive_early_halt_threshold: Option<f32>,
}

#[cfg(feature = "llm")]
impl Default for QianjiLiveContractFeedbackOptions {
    fn default() -> Self {
        Self {
            model: DEFAULT_LIVE_FEEDBACK_MODEL.to_string(),
            temperature: DEFAULT_LIVE_FEEDBACK_TEMPERATURE,
            cognitive_early_halt_threshold: None,
        }
    }
}

/// Runtime dependencies for the `llm`-gated live contract-feedback lane.
#[cfg(feature = "llm")]
#[derive(Clone)]
pub struct QianjiLiveContractFeedbackRuntime {
    /// Planner runtime for role orchestration.
    pub orchestrator: Arc<ThousandFacesOrchestrator>,
    /// Persona registry used by the advisory planner.
    pub registry: Arc<PersonaRegistry>,
    /// LLM client used for live advisory execution.
    pub client: Arc<dyn LlmClient>,
}

#[cfg(feature = "llm")]
impl QianjiLiveContractFeedbackRuntime {
    /// Construct one live-advisory runtime bundle.
    #[must_use]
    pub fn new(
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        client: Arc<dyn LlmClient>,
    ) -> Self {
        Self {
            orchestrator,
            registry,
            client,
        }
    }
}

/// Execute one contract suite through the live `Qianji + Qianhuan + LLM` advisory lane and export
/// Wendao-ready knowledge entries.
///
/// # Errors
///
/// Returns an error when advisory planning fails, when the LLM-backed advisory executor fails, or
/// when the underlying contract suite run fails.
#[cfg(feature = "llm")]
/// Positional boundary: this compatibility API keeps the established public call shape.
pub async fn run_contract_feedback_flow_with_live_advisory(
    suite: &ContractSuite,
    ctx: &CollectionContext,
    config: &ContractRunConfig,
    orchestrator: Arc<ThousandFacesOrchestrator>,
    registry: Arc<PersonaRegistry>,
    client: Arc<dyn LlmClient>,
    options: QianjiLiveContractFeedbackOptions,
) -> Result<QianjiContractFeedbackRun> {
    let planner = QianjiAdvisoryAuditExecutor::new(orchestrator, registry);
    let mut live_executor = QianjiLlmAdvisoryAuditExecutor::new(planner, client, options.model)
        .with_temperature(options.temperature);
    if let Some(threshold) = options.cognitive_early_halt_threshold {
        live_executor = live_executor.with_cognitive_supervision(threshold);
    }

    run_contract_feedback_flow(suite, ctx, config, &live_executor).await
}

/// Execute one contract suite through the live advisory lane and persist the resulting knowledge
/// entries through a sovereign sink.
///
/// # Errors
///
/// Returns an error when live advisory execution fails or when the sink fails to persist the
/// generated knowledge entries.
#[cfg(feature = "llm")]
/// Positional boundary: this compatibility API keeps the established public call shape.
pub async fn run_and_persist_contract_feedback_flow_with_live_advisory(
    suite: &ContractSuite,
    ctx: &CollectionContext,
    config: &ContractRunConfig,
    runtime: QianjiLiveContractFeedbackRuntime,
    options: QianjiLiveContractFeedbackOptions,
    sink: &dyn ContractFeedbackKnowledgeSink,
) -> Result<QianjiPersistedContractFeedbackRun> {
    let run = run_contract_feedback_flow_with_live_advisory(
        suite,
        ctx,
        config,
        runtime.orchestrator,
        runtime.registry,
        runtime.client,
        options,
    )
    .await?;

    persist_contract_feedback_run(run, sink).await
}
