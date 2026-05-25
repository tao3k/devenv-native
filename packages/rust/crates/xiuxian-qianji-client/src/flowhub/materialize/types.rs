//! Shared materialization data types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AgentPlanSourceMetadata {
    pub(super) scenario_id: String,
    pub(super) org_source: String,
    pub(super) org_sha256: String,
    pub(super) bpmn_source: String,
    pub(super) bpmn_sha256: String,
    pub(super) bpmn_process_id: String,
}
