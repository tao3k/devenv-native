//! File-backed `rest_docs` contract-feedback helpers for real Qianji callers.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use xiuxian_testing::{
    ArtifactKind, CollectedArtifact, CollectedArtifacts, CollectionContext, ContractFinding,
    ContractRunConfig, ContractSuite, NoopAdvisoryAuditExecutor, RestDocsRulePack, RulePack,
    RulePackDescriptor,
};

use crate::sovereign::ContractFeedbackKnowledgeSink;

use super::pipeline::{
    QianjiContractFeedbackRun, QianjiPersistedContractFeedbackRun,
    run_and_persist_contract_feedback_flow, run_contract_feedback_flow,
};

const REST_DOCS_SUITE_ID: &str = "qianji-rest-docs-contract-feedback";

/// `rest_docs` rule-pack wrapper that reads one local `OpenAPI` file from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenApiFileRestDocsRulePack {
    openapi_path: PathBuf,
    artifact_id: String,
}

impl OpenApiFileRestDocsRulePack {
    /// Create a new file-backed `rest_docs` rule pack.
    #[must_use]
    pub fn new(openapi_path: impl Into<PathBuf>) -> Self {
        let openapi_path = openapi_path.into();
        let artifact_id = format!("openapi:{}", openapi_path.display());
        Self {
            openapi_path,
            artifact_id,
        }
    }

    /// Return the backing `OpenAPI` document path.
    #[must_use]
    pub fn openapi_path(&self) -> &Path {
        &self.openapi_path
    }

    fn load_openapi_document(&self) -> Result<serde_json::Value> {
        let raw = fs::read_to_string(&self.openapi_path).with_context(|| {
            format!(
                "failed to read OpenAPI document at {}",
                self.openapi_path.display()
            )
        })?;

        serde_json::from_str(&raw)
            .or_else(|_| serde_yaml::from_str::<serde_json::Value>(&raw))
            .with_context(|| {
                format!(
                    "failed to parse OpenAPI document at {} as JSON or YAML",
                    self.openapi_path.display()
                )
            })
    }
}

impl RulePack for OpenApiFileRestDocsRulePack {
    fn descriptor(&self) -> RulePackDescriptor {
        RestDocsRulePack.descriptor()
    }

    fn collect(&self, _ctx: &CollectionContext) -> Result<CollectedArtifacts> {
        let mut artifacts = CollectedArtifacts::default();
        artifacts.push(CollectedArtifact {
            id: self.artifact_id.clone(),
            kind: ArtifactKind::OpenApiDocument,
            path: Some(self.openapi_path.clone()),
            content: self.load_openapi_document()?,
            labels: BTreeMap::from([("artifact_source".to_string(), "openapi_file".to_string())]),
        });
        Ok(artifacts)
    }

    fn evaluate(&self, artifacts: &CollectedArtifacts) -> Result<Vec<ContractFinding>> {
        RestDocsRulePack.evaluate(artifacts)
    }
}

/// Build a bounded contract suite that evaluates one local `OpenAPI` file with the built-in
/// `rest_docs` pack.
#[must_use]
pub fn build_rest_docs_contract_suite(openapi_path: impl Into<PathBuf>) -> ContractSuite {
    let mut suite = ContractSuite::new(REST_DOCS_SUITE_ID, "v1");
    suite.register_rule_pack(Box::new(OpenApiFileRestDocsRulePack::new(openapi_path)));
    suite
}

/// Build collection context for one file-backed `rest_docs` contract-feedback run.
#[must_use]
pub fn build_rest_docs_collection_context(
    openapi_path: &Path,
    workspace_root: Option<PathBuf>,
) -> CollectionContext {
    CollectionContext {
        suite_id: REST_DOCS_SUITE_ID.to_string(),
        crate_name: None,
        workspace_root,
        labels: BTreeMap::from([
            ("artifact_source".to_string(), "openapi_file".to_string()),
            (
                "openapi_path".to_string(),
                openapi_path.to_string_lossy().into_owned(),
            ),
        ]),
    }
}

/// Run file-backed `rest_docs` contract feedback without persistence.
///
/// # Errors
///
/// Returns an error when the `OpenAPI` file cannot be loaded, when deterministic evaluation
/// fails, or when the advisory executor fails for a triggered advisory lane.
pub async fn run_rest_docs_contract_feedback(
    openapi_path: impl Into<PathBuf>,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_executor: &dyn xiuxian_testing::AdvisoryAuditExecutor,
) -> Result<QianjiContractFeedbackRun> {
    let suite = build_rest_docs_contract_suite(openapi_path);
    run_contract_feedback_flow(&suite, &collection_context, config, advisory_executor).await
}

/// Run file-backed `rest_docs` contract feedback and persist the result through the provided sink.
///
/// # Errors
///
/// Returns an error when the `OpenAPI` file cannot be loaded, when the deterministic contract run
/// fails, or when the sink cannot persist the generated knowledge entries.
pub async fn run_and_persist_rest_docs_contract_feedback(
    openapi_path: impl Into<PathBuf>,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    sink: &dyn ContractFeedbackKnowledgeSink,
) -> Result<QianjiPersistedContractFeedbackRun> {
    let suite = build_rest_docs_contract_suite(openapi_path);
    run_and_persist_contract_feedback_flow(
        &suite,
        &collection_context,
        config,
        &NoopAdvisoryAuditExecutor,
        sink,
    )
    .await
}

#[cfg(test)]
#[path = "../tests/unit/contract_feedback/rest_docs.rs"]
mod tests;
