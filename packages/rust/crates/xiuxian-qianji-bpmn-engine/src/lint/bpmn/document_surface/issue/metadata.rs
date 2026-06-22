use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::summary::document_surface_evidence;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn resource_role_metadata_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
    let evidence = document_surface_evidence(source, "resourceRole", "resource_role");
    if resource_role_count_from_evidence(&evidence) == 0 {
        return None;
    }
    let source_id = &source.source_id;
    Some(
        LintIssue::from_parts(
            "bpmn.unsupported_resource_role_metadata",
            "Resource-role assignment semantics are deferred",
            format!(
                "Source '{source_id}' contains direct process or global-task BPMN resource-role metadata."
            ),
            "The bounded engine preserves process/global-task `resourceRole`, `performer`, `humanPerformer`, `potentialOwner`, `resourceRef`, `resourceParameterBinding`, and `resourceAssignmentExpression` declarations as snapshot evidence, but it does not execute generic assignment, scheduling, authorization, delegation, escalation, resource-parameter binding, or worklist semantics from those declarations.",
            vec![
                "Keep process/global-task resource roles as metadata when they are useful for audit, interchange, or future assignment policy.".to_string(),
                "Do not rely on process/global-task resource roles to filter worklists, authorize task completion, schedule workers, or evaluate resource parameter bindings in this bounded runtime slice.".to_string(),
                "For current executable human-work routing, use task-local `humanPerformer` or `potentialOwner` hints on user/manual tasks, or route through explicit host-work metadata.".to_string(),
            ],
            format!(
                "Repair BPMN source '{source_id}' by treating direct process/global-task resource-role declarations as metadata only. If assignment behavior is required now, move bounded routing hints to task-local `humanPerformer` or `potentialOwner` metadata on user/manual tasks, or model the assignment decision as explicit host-work or workflow data."
            ),
            evidence,
        )
        .with_structured_repair(json!({
            "schema_version": 1,
            "contract": "bpmn.native.resource_role.metadata_only.v1",
            "strategy": "preserve_resource_roles_as_metadata",
            "allowed_runtime_surface": [
                "task_local_humanPerformer_routing_hint",
                "task_local_potentialOwner_routing_hint",
                "explicit_host_work_metadata",
                "workflow_variables"
            ],
            "deferred_semantics": [
                "generic_resource_role_assignment",
                "resource_parameter_binding_evaluation",
                "authorization",
                "scheduling",
                "delegation",
                "escalation",
                "administrative_reassignment"
            ]
        })),
    )
}

pub(super) fn flow_element_metadata_issue(source: &BpmnSourceFile) -> Option<LintIssue> {
    let evidence = document_surface_evidence(source, "flowElement", "flow_element_metadata");
    if flow_element_metadata_count_from_evidence(&evidence) == 0 {
        return None;
    }
    let source_id = &source.source_id;
    Some(
        LintIssue::from_parts(
            "bpmn.unsupported_flow_element_metadata",
            "Flow-element audit and classification semantics are deferred",
            format!(
                "Source '{source_id}' contains BPMN flow-element auditing, monitoring, or category metadata."
            ),
            "The bounded engine preserves standard BPMN `auditing`, `monitoring`, and `categoryValueRef` declarations on flow elements as snapshot evidence, but it does not execute audit hooks, monitoring telemetry, category classification, scheduling, authorization, policy enforcement, or runtime routing from those declarations.",
            vec![
                "Keep flow-element auditing, monitoring, and category references as metadata when they are useful for audit, interchange, or future policy work.".to_string(),
                "Do not rely on flow-element metadata to emit telemetry, enforce authorization, schedule work, classify runtime paths, or change token routing in this bounded runtime slice.".to_string(),
                "Model executable audit, monitoring, or policy decisions as explicit service tasks, host-dispatched tasks, gateway conditions, or workflow variables.".to_string(),
            ],
            format!(
                "Repair BPMN source '{source_id}' by treating flow-element auditing, monitoring, and category references as metadata only. If executable policy or telemetry behavior is required now, model it with explicit supported tasks, conditions, or host-work payload fields."
            ),
            evidence,
        )
        .with_structured_repair(json!({
            "schema_version": 1,
            "contract": "bpmn.native.flow_element.metadata_only.v1",
            "strategy": "preserve_flow_element_metadata",
            "allowed_runtime_surface": [
                "supported_process_flow",
                "supported_events",
                "supported_tasks",
                "gateway_conditions",
                "host_work_metadata",
                "workflow_variables"
            ],
            "deferred_semantics": [
                "audit_hook_execution",
                "monitoring_telemetry",
                "category_classification",
                "authorization_policy",
                "scheduling_policy",
                "runtime_routing_from_category_refs"
            ]
        })),
    )
}

fn resource_role_count_from_evidence(evidence: &serde_json::Value) -> u64 {
    let Some(resource_roles) = evidence.pointer("/snapshot/resource_roles") else {
        return 0;
    };
    resource_roles
        .get("process_role_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + resource_roles
            .get("global_task_role_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
}

fn flow_element_metadata_count_from_evidence(evidence: &serde_json::Value) -> u64 {
    evidence
        .pointer("/snapshot/flow_element_metadata/element_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}
