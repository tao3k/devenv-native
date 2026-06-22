use super::checkpoint::{load_required_checkpoint, resolve_checkpoint_store};
use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::control::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowTaskClaimReport,
    QianjiBpmnWorkflowTaskClaimRequest, QianjiBpmnWorkflowTaskReleaseReport,
    QianjiBpmnWorkflowTaskReleaseRequest, QianjiBpmnWorkflowWorklistItem,
    QianjiBpmnWorkflowWorklistReport, QianjiBpmnWorkflowWorklistRequest,
    QianjiBpmnWorkflowWorklistRoutingFilter,
};
use crate::bpmn::error::BpmnOrchestrationError;
use crate::bpmn::execution::DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS;
use crate::bpmn::ownership::QianjiBpmnSchedulerLeaseConfig;
use crate::telemetry::unix_millis_now;
use std::io;
use xiuxian_qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskResourceRoleSpec,
    BpmnLaneMembershipSpec, PendingHumanTaskClaimInput, PendingHumanTaskClaimRequest,
    PendingHumanTaskReleaseInput, PendingHumanTaskReleaseRequest, claim_pending_human_task,
    release_pending_human_task,
};

pub(crate) async fn claim_workflow_task(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowTaskClaimRequest,
) -> Result<QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowControlError> {
    let (checkpoint_store, mut checkpoint) =
        load_required_checkpoint(service, &request.instance_id, &request.checkpoint_backend)
            .await?;
    let outcome = claim_pending_human_task(
        &mut checkpoint.state,
        PendingHumanTaskClaimRequest::from_input(PendingHumanTaskClaimInput {
            token_id: request.claim.token_id.into(),
            process_id: request.claim.process_id.as_str().into(),
            activity_id: request.claim.activity_id.as_str().into(),
            claimant: request.claim.claimant.clone(),
            claimed_at_ms: unix_millis_now().into(),
        }),
    )
    .map_err(BpmnOrchestrationError::from)?;

    if outcome.changed {
        checkpoint.sequence = checkpoint.state.sequence;
        save_claimed_checkpoint(
            service,
            &request.checkpoint_backend,
            &checkpoint_store,
            &checkpoint,
        )
        .await?;
    }

    Ok(QianjiBpmnWorkflowTaskClaimReport {
        checkpoint_store,
        checkpoint_sequence: checkpoint.sequence,
        instance: checkpoint.state,
        claimed_work: outcome.pending_host_work,
        changed: outcome.changed,
    })
}

pub(crate) async fn release_workflow_task(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowTaskReleaseRequest,
) -> Result<QianjiBpmnWorkflowTaskReleaseReport, QianjiBpmnWorkflowControlError> {
    let (checkpoint_store, mut checkpoint) =
        load_required_checkpoint(service, &request.instance_id, &request.checkpoint_backend)
            .await?;
    let outcome = release_pending_human_task(
        &mut checkpoint.state,
        PendingHumanTaskReleaseRequest::from_input(PendingHumanTaskReleaseInput {
            token_id: request.release.token_id.into(),
            process_id: request.release.process_id.as_str().into(),
            activity_id: request.release.activity_id.as_str().into(),
            claimant: request.release.claimant.clone(),
            released_at_ms: unix_millis_now().into(),
        }),
    )
    .map_err(BpmnOrchestrationError::from)?;

    if outcome.changed {
        checkpoint.sequence = checkpoint.state.sequence;
        save_claimed_checkpoint(
            service,
            &request.checkpoint_backend,
            &checkpoint_store,
            &checkpoint,
        )
        .await?;
    }

    Ok(QianjiBpmnWorkflowTaskReleaseReport {
        checkpoint_store,
        checkpoint_sequence: checkpoint.sequence,
        instance: checkpoint.state,
        released_work: outcome.pending_host_work,
        changed: outcome.changed,
    })
}

pub(crate) async fn list_workflow_worklist(
    service: &QianjiBpmnWorkflowControlService,
    request: &QianjiBpmnWorkflowWorklistRequest,
) -> Result<QianjiBpmnWorkflowWorklistReport, QianjiBpmnWorkflowControlError> {
    let checkpoint_store = resolve_checkpoint_store(service, Some(&request.checkpoint_backend))?
        .ok_or_else(|| io::Error::other("workflow operation requires one checkpoint backend"))?;
    let claimant = request
        .claimant
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let routing = normalize_worklist_routing_filter(&request.routing);
    let work_items = checkpoint_store
        .list()?
        .iter()
        .flat_map(|checkpoint| {
            checkpoint
                .state
                .pending_host_work
                .iter()
                .filter_map(|pending| {
                    QianjiBpmnWorkflowWorklistItem::from_pending_host_work(checkpoint, pending)
                })
        })
        .filter(|item| match claimant {
            Some(claimant) => item
                .claim
                .as_ref()
                .is_none_or(|claim| claim.claimant == claimant),
            None => true,
        })
        .filter(|item| worklist_routing_matches(item, &routing))
        .collect();

    Ok(QianjiBpmnWorkflowWorklistReport {
        checkpoint_store,
        work_items,
    })
}

fn normalize_worklist_routing_filter(
    routing: &QianjiBpmnWorkflowWorklistRoutingFilter,
) -> QianjiBpmnWorkflowWorklistRoutingFilter {
    QianjiBpmnWorkflowWorklistRoutingFilter {
        assignment_resource: normalize_filter_value(routing.assignment_resource.as_deref()),
        lane: normalize_filter_value(routing.lane.as_deref()),
    }
}

fn normalize_filter_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn worklist_routing_matches(
    item: &QianjiBpmnWorkflowWorklistItem,
    routing: &QianjiBpmnWorkflowWorklistRoutingFilter,
) -> bool {
    let assignment_matches = match routing.assignment_resource.as_deref() {
        Some(resource) => item
            .assignment
            .as_ref()
            .is_some_and(|assignment| assignment_matches_resource(assignment, resource)),
        None => true,
    };
    let lane_matches = match routing.lane.as_deref() {
        Some(lane) => item
            .lane
            .as_ref()
            .is_some_and(|membership| lane_membership_matches(membership, lane)),
        None => true,
    };
    assignment_matches && lane_matches
}

fn assignment_matches_resource(assignment: &BpmnHumanTaskAssignmentSpec, resource: &str) -> bool {
    assignment
        .human_performers
        .iter()
        .chain(assignment.potential_owners.iter())
        .any(|role| resource_role_matches(role, resource))
}

fn resource_role_matches(role: &BpmnHumanTaskResourceRoleSpec, resource: &str) -> bool {
    role.name.as_deref() == Some(resource) || role.resource_ref.as_deref() == Some(resource)
}

fn lane_membership_matches(membership: &BpmnLaneMembershipSpec, lane: &str) -> bool {
    membership.id.as_deref() == Some(lane) || membership.name.as_deref() == Some(lane)
}

async fn save_claimed_checkpoint(
    service: &QianjiBpmnWorkflowControlService,
    checkpoint_backend: &QianjiBpmnWorkflowCheckpointBackend,
    checkpoint_store: &QianjiBpmnCheckpointStore,
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<(), QianjiBpmnWorkflowControlError> {
    match checkpoint_backend {
        QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey => {
            save_runtime_valkey_claimed_checkpoint(service, checkpoint_store, checkpoint).await
        }
        #[cfg(feature = "duckdb")]
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb => {
            checkpoint_store.save(checkpoint).await?;
            Ok(())
        }
    }
}

async fn save_runtime_valkey_claimed_checkpoint(
    service: &QianjiBpmnWorkflowControlService,
    checkpoint_store: &QianjiBpmnCheckpointStore,
    checkpoint: &BpmnCheckpointEnvelope,
) -> Result<(), QianjiBpmnWorkflowControlError> {
    let scheduler_identity = service.scheduler_identity.clone().unwrap_or_default();
    let lease = QianjiBpmnSchedulerLeaseConfig::from_scheduler_identity(
        &scheduler_identity,
        DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    )?;
    let owner_token = lease.owner_token().to_string();
    let acquired = checkpoint_store
        .try_acquire_lease(
            checkpoint.state.instance_id.as_ref(),
            owner_token.as_str(),
            lease.lease_ttl_ms(),
        )
        .await?;
    if !acquired {
        return Err(BpmnOrchestrationError::CheckpointLeaseConflict {
            instance_id: checkpoint.state.instance_id.as_ref().into(),
            owner_token: owner_token.into(),
        }
        .into());
    }

    let save_result = checkpoint_store
        .save_as_owner(checkpoint, lease.owner_token())
        .await;
    let release_result = checkpoint_store
        .release_lease(checkpoint.state.instance_id.as_ref(), lease.owner_token())
        .await;

    save_result?;
    let _released = release_result?;
    Ok(())
}
