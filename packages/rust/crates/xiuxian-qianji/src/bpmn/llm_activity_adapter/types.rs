//! Typed route decision model for BPMN host-work LLM activities.

use crate::bpmn::QianjiBpmnPendingHostWorkHttpResponse;
use crate::runtime_config::QianjiRuntimeLlmConfig;
use crate::workflow_config::QianjiWorkflowLlmTaskConfig;
use xiuxian_qianji_control::{ArtifactRef, LlmActivityTask};

/// Schema marker for BPMN host-work to LLM activity route decisions.
pub const BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA: &str =
    "qianji.bpmn.host_work.llm_activity_route.v1";

/// Inputs required to route one BPMN pending host-work item into an LLM task.
#[derive(Debug, Clone, Copy)]
pub struct BpmnHostWorkLlmActivityRouteInput<'a> {
    /// Workflow instance identifier that owns the pending work.
    pub instance_id: &'a str,
    /// Optional server-recorded BPMN source reference.
    pub bpmn_source_ref: Option<&'a str>,
    /// Workflow-task config profile name.
    pub profile: &'a str,
    /// Pending host-work item from the workflow snapshot.
    pub pending_work: &'a QianjiBpmnPendingHostWorkHttpResponse,
    /// Loaded workflow-task profile.
    pub workflow_config: &'a QianjiWorkflowLlmTaskConfig,
    /// Resolved global runtime LLM defaults.
    pub runtime_llm: &'a QianjiRuntimeLlmConfig,
    /// Local prompt claim-check artifact.
    pub prompt_ref: &'a ArtifactRef,
    /// Optional local context claim-check artifact.
    pub context_ref: Option<&'a ArtifactRef>,
    /// Optional response schema claim-check artifact.
    pub response_schema_ref: Option<&'a ArtifactRef>,
}

/// Secret-free endpoint decision for a routed BPMN LLM activity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHostWorkLlmEndpointDecision {
    /// Provider label used for operator/debug routing metadata.
    pub provider: String,
    /// Model selected for the LLM activity request.
    pub model: String,
    /// OpenAI-compatible base URL used by the worker.
    pub base_url: String,
    /// Environment variable name containing the provider API key.
    pub api_key_env: String,
    /// OpenAI-compatible wire mode.
    pub wire_api: String,
}

/// Raw DTO boundary: route decision returned before scheduling an LLM activity.
///
/// Semantic field boundary: this public DTO preserves externally serialized
/// field names for operator inspection and control-plane handoff.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHostWorkLlmRouteDecision {
    /// Route decision schema marker.
    pub schema: String,
    /// Workflow-task config profile name.
    pub profile: String,
    /// Workflow instance identifier.
    pub instance_id: String,
    /// BPMN process identifier.
    pub process_id: String,
    /// Runtime token identifier.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Optional BPMN node identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// BPMN activity identifier.
    pub activity_id: String,
    /// Optional host-generated work identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
    /// Optional server-recorded BPMN source reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bpmn_source_ref: Option<String>,
    /// Secret-free provider endpoint decision.
    pub endpoint: BpmnHostWorkLlmEndpointDecision,
    /// Validated control-plane LLM activity contract.
    pub llm_activity: LlmActivityTask,
}
