//! End-to-end contract-feedback pipeline for Qianji-driven suite runs.

#[cfg(feature = "llm")]
use std::sync::Arc;

use anyhow::Result;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::LlmClient;
#[cfg(feature = "llm")]
use xiuxian_qianhuan::{PersonaRegistry, ThousandFacesOrchestrator};
use xiuxian_testing::{
    AdvisoryAuditExecutor, CollectionContext, ContractKnowledgeBatch, ContractReport,
    ContractRunConfig, ContractSuite, ContractSuiteRunner,
};
use xiuxian_wendao_core::{KnowledgeEntry, WendaoContractFeedbackAdapter};

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
        let knowledge_batch = ContractKnowledgeBatch::from_report(&report);
        let knowledge_entries =
            WendaoContractFeedbackAdapter::knowledge_entries_from_batch(&knowledge_batch);

        Self {
            report,
            knowledge_batch,
            knowledge_entries,
        }
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
