//! Contract-feedback branch for advisory policy, evidence, and runner surfaces.

mod advisory;
mod model;
#[path = "../contract_feedback_pipeline.rs"]
mod pipeline;
#[path = "../contract_feedback_rest_docs.rs"]
mod rest_docs;
mod rule_pack;
mod runner;

pub use advisory::{
    AdvisoryAuditExecutor, AdvisoryAuditPolicy, AdvisoryAuditRequest, NoopAdvisoryAuditExecutor,
    RoleAuditFinding,
};
pub use model::{
    ArtifactKind, CollectedArtifact, CollectedArtifacts, CollectionContext, ContractExecutionMode,
    ContractFinding, ContractReport, ContractStats, EvidenceKind, FindingConfidence,
    FindingEvidence, FindingExamples, FindingMode, FindingSeverity,
};
pub use pipeline::{
    QianjiContractFeedbackRun, QianjiPersistedContractFeedbackRun, persist_contract_feedback_run,
    run_and_persist_contract_feedback_flow, run_contract_feedback_flow,
};
pub use rest_docs::{
    OpenApiFileRestDocsRulePack, build_rest_docs_collection_context,
    build_rest_docs_contract_suite, run_and_persist_rest_docs_contract_feedback,
    run_rest_docs_contract_feedback,
};
pub use rule_pack::{ContractSuite, RulePack, RulePackDescriptor};
pub use runner::{ContractRunConfig, ContractSuiteRunner};

#[cfg(feature = "llm")]
pub use rest_docs::{
    run_and_persist_rest_docs_contract_feedback_with_live_advisory,
    run_rest_docs_contract_feedback_with_live_advisory,
};

#[cfg(feature = "llm")]
pub use pipeline::{
    QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime,
    run_and_persist_contract_feedback_flow_with_live_advisory,
    run_contract_feedback_flow_with_live_advisory,
};
