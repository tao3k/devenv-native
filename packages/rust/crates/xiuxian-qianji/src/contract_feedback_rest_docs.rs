//! File-backed `rest_docs` contract-feedback helpers for real Qianji callers.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::contract_feedback::{
    AdvisoryAuditExecutor, AdvisoryAuditRequest, ArtifactKind, CollectedArtifact,
    CollectedArtifacts, CollectionContext, ContractFinding, ContractReport, ContractRunConfig,
    ContractSuite, EvidenceKind, FindingEvidence, FindingMode, FindingSeverity,
    NoopAdvisoryAuditExecutor, RulePack, RulePackDescriptor,
};
use anyhow::{Context, Result};
use serde_json::{Map, Value};

#[cfg(feature = "llm")]
use crate::executors::{QianjiAdvisoryAuditExecutor, QianjiLlmAdvisoryAuditExecutor};
use crate::sovereign::ContractFeedbackKnowledgeSink;

use super::pipeline::{
    QianjiContractFeedbackRun, QianjiPersistedContractFeedbackRun, persist_contract_feedback_run,
};
#[cfg(feature = "llm")]
use super::pipeline::{QianjiLiveContractFeedbackOptions, QianjiLiveContractFeedbackRuntime};

const REST_DOCS_SUITE_ID: &str = "qianji-rest-docs-contract-feedback";
const REST_DOCS_PACK_ID: &str = "rest_docs";
const HTTP_METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

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

    fn collect_openapi_artifacts(&self) -> Result<CollectedArtifacts> {
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

    fn evaluate_openapi_artifacts(artifacts: &CollectedArtifacts) -> Vec<ContractFinding> {
        let mut findings = Vec::new();

        for artifact in &artifacts.artifacts {
            if artifact.kind != ArtifactKind::OpenApiDocument {
                continue;
            }

            findings.extend(RestDocsEvaluator::new(artifact).evaluate());
        }

        findings
    }
}

impl RulePack for OpenApiFileRestDocsRulePack {
    fn descriptor(&self) -> RulePackDescriptor {
        RulePackDescriptor {
            id: REST_DOCS_PACK_ID,
            version: "v1",
            domains: &["rest", "docs", "openapi"],
            default_mode: FindingMode::Deterministic,
        }
    }

    fn collect(&self, _ctx: &CollectionContext) -> Result<CollectedArtifacts> {
        self.collect_openapi_artifacts()
    }

    fn evaluate(&self, artifacts: &CollectedArtifacts) -> Result<Vec<ContractFinding>> {
        Ok(Self::evaluate_openapi_artifacts(artifacts))
    }
}

struct RestDocsEvaluator<'a> {
    artifact: &'a CollectedArtifact,
}

impl<'a> RestDocsEvaluator<'a> {
    fn new(artifact: &'a CollectedArtifact) -> Self {
        Self { artifact }
    }

    fn evaluate(&self) -> Vec<ContractFinding> {
        let mut findings = Vec::new();
        let Some(paths) = self
            .artifact
            .content
            .get("paths")
            .and_then(Value::as_object)
        else {
            return findings;
        };

        for (path_name, path_item) in paths {
            let Some(path_object) = self.resolve_object(path_item) else {
                continue;
            };

            for method in HTTP_METHODS {
                let Some(operation_value) = path_object.get(method) else {
                    continue;
                };
                let Some(operation) = self.resolve_object(operation_value) else {
                    continue;
                };

                findings.extend(self.check_endpoint_purpose(path_name, method, operation));
                findings.extend(self.check_response_documentation(path_name, method, operation));
                findings.extend(self.check_request_examples(path_name, method, operation));
            }
        }

        findings
    }

    fn check_endpoint_purpose(
        &self,
        path_name: &str,
        method: &str,
        operation: &Map<String, Value>,
    ) -> Option<ContractFinding> {
        let summary = operation.get("summary").and_then(Value::as_str);
        let description = operation.get("description").and_then(Value::as_str);
        if !is_blank(summary) || !is_blank(description) {
            return None;
        }

        let mut finding = self.base_finding(
            "REST-R001",
            FindingSeverity::Error,
            path_name,
            method,
            "Missing endpoint purpose",
            format!(
                "The {} {} operation is missing both `summary` and `description`.",
                method.to_uppercase(),
                path_name
            ),
        );
        finding.why_it_matters = "External callers, reviewers, and knowledge-indexing pipelines need a stable purpose statement for every reachable endpoint.".to_string();
        finding.remediation = "Add a non-empty `summary` or `description` that explains what the endpoint does and when callers should use it.".to_string();
        finding
            .examples
            .good
            .push("GET /health includes a short summary like `Check gateway health`.".to_string());
        finding.examples.bad.push(
            "GET /health exposes only response schemas with no human-readable purpose.".to_string(),
        );
        finding.evidence.push(self.open_api_evidence(
            path_name,
            method,
            None,
            "Operation is missing both `summary` and `description`.".to_string(),
        ));
        Some(finding)
    }

    fn check_response_documentation(
        &self,
        path_name: &str,
        method: &str,
        operation: &Map<String, Value>,
    ) -> Option<ContractFinding> {
        let Some(responses) = operation.get("responses").and_then(Value::as_object) else {
            return Some(self.missing_responses_finding(path_name, method));
        };

        let issues = self.collect_response_issues(responses);
        if issues.is_empty() {
            return None;
        }

        Some(self.response_documentation_finding(path_name, method, issues))
    }

    fn check_request_examples(
        &self,
        path_name: &str,
        method: &str,
        operation: &Map<String, Value>,
    ) -> Option<ContractFinding> {
        let request_body = operation.get("requestBody")?;
        let request_body_object = self.resolve_object(request_body)?;
        let content = request_body_object
            .get("content")
            .and_then(Value::as_object)?;

        let non_trivial_media = content
            .iter()
            .filter_map(|(media_type, media_value)| {
                let media_object = media_value.as_object()?;
                let schema = media_object.get("schema");
                self.schema_is_non_trivial(schema)
                    .then_some((media_type, media_object, schema))
            })
            .collect::<Vec<_>>();

        if non_trivial_media.is_empty()
            || non_trivial_media.iter().any(|(_, media_object, schema)| {
                media_type_has_examples(media_object) || self.schema_has_examples(*schema)
            })
        {
            return None;
        }

        let missing_media_types = non_trivial_media
            .into_iter()
            .map(|(media_type, _, _)| media_type.clone())
            .collect::<Vec<_>>();

        let mut finding = self.base_finding(
            "REST-R007",
            FindingSeverity::Warning,
            path_name,
            method,
            "Missing request-body example",
            format!(
                "The {} {} operation has a non-trivial request body but no request example.",
                method.to_uppercase(),
                path_name
            ),
        );
        finding.why_it_matters = "Concrete request examples make REST contracts easier to review, test, and consume correctly, especially when the schema is object-shaped or referenced.".to_string();
        finding.remediation = "Add `example` or `examples` data to at least one non-trivial request media type or its resolved schema.".to_string();
        finding.examples.good.push(
            "Provide an `application/json` example that mirrors a realistic request payload."
                .to_string(),
        );
        finding.examples.bad.push(
            "Define an object schema with several fields but leave all request examples empty."
                .to_string(),
        );
        finding.evidence.push(self.open_api_evidence(
            path_name,
            method,
            Some("/requestBody"),
            format!(
                "Non-trivial request media types are missing examples: {}.",
                missing_media_types.join(", ")
            ),
        ));
        Some(finding)
    }

    fn missing_responses_finding(&self, path_name: &str, method: &str) -> ContractFinding {
        let mut finding = self.base_finding(
            "REST-R003",
            FindingSeverity::Error,
            path_name,
            method,
            "Missing response documentation",
            format!(
                "The {} {} operation does not declare any documented responses.",
                method.to_uppercase(),
                path_name
            ),
        );
        finding.why_it_matters = "REST contracts need explicit success and error response coverage so clients and reviewers can reason about expected behavior.".to_string();
        finding.remediation =
            "Add documented success and error responses with non-empty descriptions.".to_string();
        finding.evidence.push(self.open_api_evidence(
            path_name,
            method,
            Some("/responses"),
            "Operation is missing the `responses` object.".to_string(),
        ));
        finding
    }

    fn collect_response_issues(&self, responses: &Map<String, Value>) -> Vec<ResponseIssue> {
        let mut issues = Vec::new();
        self.extend_response_issues_for_class(responses, ResponseClass::Success, &mut issues);
        self.extend_response_issues_for_class(responses, ResponseClass::Error, &mut issues);
        issues
    }

    fn extend_response_issues_for_class(
        &self,
        responses: &Map<String, Value>,
        class: ResponseClass,
        issues: &mut Vec<ResponseIssue>,
    ) {
        let statuses = collect_statuses(responses, |status| class.matches_status(status));
        if statuses.is_empty() {
            issues.push(ResponseIssue {
                locator_suffix: "/responses".to_string(),
                message: class.missing_coverage_message().to_string(),
            });
            return;
        }

        for status in statuses {
            let Some(response) = responses
                .get(status)
                .and_then(|value| self.resolve_object(value))
            else {
                continue;
            };
            if has_non_empty_description(response) {
                continue;
            }

            issues.push(ResponseIssue {
                locator_suffix: format!("/responses/{status}"),
                message: class.missing_description_message(status),
            });
        }
    }

    fn response_documentation_finding(
        &self,
        path_name: &str,
        method: &str,
        issues: Vec<ResponseIssue>,
    ) -> ContractFinding {
        let mut finding = self.base_finding(
            "REST-R003",
            FindingSeverity::Error,
            path_name,
            method,
            "Incomplete response documentation",
            format!(
                "The {} {} operation is missing required response documentation.",
                method.to_uppercase(),
                path_name
            ),
        );
        finding.why_it_matters = "Clients need documented success and error responses to handle the API safely and to keep generated contracts aligned with implementation intent.".to_string();
        finding.remediation = "Document at least one success response and one error response, and give each response a non-empty description.".to_string();
        finding.examples.good.push("Document `200` and `400` responses with short descriptions such as `Query succeeded` and `Invalid request`.".to_string());
        finding.examples.bad.push(
            "Expose response status codes without descriptions or omit error responses entirely."
                .to_string(),
        );
        finding.evidence.extend(issues.into_iter().map(|issue| {
            self.open_api_evidence(
                path_name,
                method,
                Some(&issue.locator_suffix),
                issue.message,
            )
        }));
        finding
    }

    fn base_finding(
        &self,
        rule_id: &str,
        severity: FindingSeverity,
        path_name: &str,
        method: &str,
        title: &str,
        summary: String,
    ) -> ContractFinding {
        let mut finding = ContractFinding::new(
            rule_id,
            REST_DOCS_PACK_ID,
            severity,
            FindingMode::Deterministic,
            title,
            summary,
        );
        finding.labels = self.labels_for_operation(path_name, method);
        finding
    }

    fn labels_for_operation(&self, path_name: &str, method: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("artifact_id".to_string(), self.artifact.id.clone()),
            ("http_method".to_string(), method.to_uppercase()),
            ("path".to_string(), path_name.to_string()),
        ])
    }

    fn open_api_evidence(
        &self,
        path_name: &str,
        method: &str,
        locator_suffix: Option<&str>,
        message: String,
    ) -> FindingEvidence {
        let mut locator = format!("/paths/{}/{}", escape_json_pointer(path_name), method);
        if let Some(suffix) = locator_suffix {
            locator.push_str(suffix);
        }

        FindingEvidence {
            kind: EvidenceKind::OpenApiNode,
            path: self.artifact.path.clone(),
            locator: Some(locator),
            message,
        }
    }

    fn resolve_object<'b>(&'b self, value: &'b Value) -> Option<&'b Map<String, Value>> {
        resolve_value(&self.artifact.content, value, 0)?.as_object()
    }

    fn schema_is_non_trivial(&self, schema: Option<&Value>) -> bool {
        let Some(resolved) =
            schema.and_then(|value| resolve_value(&self.artifact.content, value, 0))
        else {
            return false;
        };
        let Some(schema_object) = resolved.as_object() else {
            return false;
        };

        if schema_object.contains_key("properties")
            || schema_object.contains_key("items")
            || schema_object.contains_key("allOf")
            || schema_object.contains_key("anyOf")
            || schema_object.contains_key("oneOf")
        {
            return true;
        }

        matches!(
            schema_object.get("type").and_then(Value::as_str),
            Some("object" | "array")
        )
    }

    fn schema_has_examples(&self, schema: Option<&Value>) -> bool {
        let Some(resolved) =
            schema.and_then(|value| resolve_value(&self.artifact.content, value, 0))
        else {
            return false;
        };
        let Some(schema_object) = resolved.as_object() else {
            return false;
        };

        if schema_object.contains_key("example") {
            return true;
        }

        schema_object
            .get("examples")
            .and_then(Value::as_array)
            .is_some_and(|examples| !examples.is_empty())
    }
}

struct ResponseIssue {
    locator_suffix: String,
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum ResponseClass {
    Success,
    Error,
}

impl ResponseClass {
    fn matches_status(self, status: &str) -> bool {
        match self {
            Self::Success => is_success_status(status),
            Self::Error => is_error_status(status),
        }
    }

    const fn missing_coverage_message(self) -> &'static str {
        match self {
            Self::Success => "Operation is missing a documented success response.",
            Self::Error => "Operation is missing a documented error response.",
        }
    }

    fn missing_description_message(self, status: &str) -> String {
        match self {
            Self::Success => {
                format!("Success response `{status}` is missing a non-empty description.")
            }
            Self::Error => format!("Error response `{status}` is missing a non-empty description."),
        }
    }
}

fn collect_statuses<P>(responses: &Map<String, Value>, predicate: P) -> BTreeSet<&str>
where
    P: Fn(&str) -> bool,
{
    responses
        .keys()
        .filter(|status| predicate(status.as_str()))
        .map(String::as_str)
        .collect()
}

fn is_success_status(status: &str) -> bool {
    status.starts_with('2')
}

fn is_error_status(status: &str) -> bool {
    status == "default" || status.starts_with('4') || status.starts_with('5')
}

fn has_non_empty_description(object: &Map<String, Value>) -> bool {
    !is_blank(object.get("description").and_then(Value::as_str))
}

fn media_type_has_examples(media_type: &Map<String, Value>) -> bool {
    if media_type.contains_key("example") {
        return true;
    }

    media_type
        .get("examples")
        .and_then(Value::as_object)
        .is_some_and(|examples| !examples.is_empty())
}

fn resolve_value<'a>(document: &'a Value, value: &'a Value, depth: usize) -> Option<&'a Value> {
    if depth > 8 {
        return Some(value);
    }

    let Some(reference) = value.get("$ref").and_then(Value::as_str) else {
        return Some(value);
    };
    let pointer = reference.strip_prefix('#')?;
    document
        .pointer(pointer)
        .and_then(|resolved| resolve_value(document, resolved, depth + 1))
        .or(Some(value))
}

fn escape_json_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn is_blank(value: Option<&str>) -> bool {
    value.is_none_or(|text| text.trim().is_empty())
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
    advisory_executor: &dyn AdvisoryAuditExecutor,
) -> Result<QianjiContractFeedbackRun> {
    let report =
        run_rest_docs_contract_report(openapi_path, collection_context, config, advisory_executor)
            .await?;
    Ok(QianjiContractFeedbackRun::from_report(report))
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
    let run = run_rest_docs_contract_feedback(
        openapi_path,
        collection_context,
        config,
        &NoopAdvisoryAuditExecutor,
    )
    .await?;
    persist_contract_feedback_run(run, sink).await
}

/// Run file-backed `rest_docs` contract feedback with live advisory execution.
///
/// # Errors
///
/// Returns an error when deterministic REST docs evaluation fails or the live advisory executor
/// fails.
#[cfg(feature = "llm")]
/// Positional boundary: this compatibility API keeps the established public call shape.
pub async fn run_rest_docs_contract_feedback_with_live_advisory(
    openapi_path: impl Into<PathBuf>,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    runtime: QianjiLiveContractFeedbackRuntime,
    options: QianjiLiveContractFeedbackOptions,
) -> Result<QianjiContractFeedbackRun> {
    let planner = QianjiAdvisoryAuditExecutor::new(runtime.orchestrator, runtime.registry);
    let mut live_executor =
        QianjiLlmAdvisoryAuditExecutor::new(planner, runtime.client, options.model)
            .with_temperature(options.temperature);
    if let Some(threshold) = options.cognitive_early_halt_threshold {
        live_executor = live_executor.with_cognitive_supervision(threshold);
    }

    run_rest_docs_contract_feedback(openapi_path, collection_context, config, &live_executor).await
}

/// Run and persist file-backed `rest_docs` contract feedback with live advisory execution.
///
/// # Errors
///
/// Returns an error when contract feedback execution fails or the sink rejects generated entries.
#[cfg(feature = "llm")]
/// Positional boundary: this compatibility API keeps the established public call shape.
pub async fn run_and_persist_rest_docs_contract_feedback_with_live_advisory(
    openapi_path: impl Into<PathBuf>,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    runtime: QianjiLiveContractFeedbackRuntime,
    options: QianjiLiveContractFeedbackOptions,
    sink: &dyn ContractFeedbackKnowledgeSink,
) -> Result<QianjiPersistedContractFeedbackRun> {
    let run = run_rest_docs_contract_feedback_with_live_advisory(
        openapi_path,
        collection_context,
        config,
        runtime,
        options,
    )
    .await?;
    persist_contract_feedback_run(run, sink).await
}

async fn run_rest_docs_contract_report(
    openapi_path: impl Into<PathBuf>,
    collection_context: CollectionContext,
    config: &ContractRunConfig,
    advisory_executor: &dyn AdvisoryAuditExecutor,
) -> Result<ContractReport> {
    let pack = OpenApiFileRestDocsRulePack::new(openapi_path);
    let artifacts = pack.collect_openapi_artifacts()?;
    let deterministic_findings =
        OpenApiFileRestDocsRulePack::evaluate_openapi_artifacts(&artifacts);
    let advisory_policy = config.advisory_policy_for_pack(REST_DOCS_PACK_ID);
    let mut findings = deterministic_findings.clone();

    if advisory_policy.enabled
        && deterministic_findings
            .iter()
            .any(|finding| finding.severity >= advisory_policy.min_severity)
    {
        let advisory_request = AdvisoryAuditRequest {
            suite_id: REST_DOCS_SUITE_ID.to_string(),
            pack_id: REST_DOCS_PACK_ID.to_string(),
            pack_version: "v1".to_string(),
            pack_domains: vec![
                "rest".to_string(),
                "docs".to_string(),
                "openapi".to_string(),
            ],
            findings: deterministic_findings,
            artifacts,
            collection_context,
            requested_roles: advisory_policy.requested_roles,
        };
        let advisory_findings = advisory_executor.run(advisory_request).await?;
        findings.extend(
            advisory_findings
                .into_iter()
                .enumerate()
                .map(|(ordinal, finding)| {
                    finding.into_contract_finding(REST_DOCS_PACK_ID, ordinal)
                }),
        );
    }

    Ok(ContractReport::from_findings(
        REST_DOCS_SUITE_ID,
        config.execution_mode,
        config.generated_at.clone(),
        findings,
    ))
}

#[cfg(test)]
#[path = "../tests/unit/contract_feedback/rest_docs.rs"]
mod tests;
