use crate::error::{BpmnEngineError, Result};
use crate::runtime::{BpmnInstanceState, PendingHostWorkClaim, PendingHostWorkKind};
use crate::runtime_claim_api::{
    PendingHumanTaskClaimOutcome, PendingHumanTaskClaimRequest, PendingHumanTaskReleaseOutcome,
    PendingHumanTaskReleaseRequest,
};
use std::borrow::Borrow;

pub(crate) fn claim_pending_human_task_impl(
    instance: &mut BpmnInstanceState,
    request: impl Borrow<PendingHumanTaskClaimRequest>,
) -> Result<PendingHumanTaskClaimOutcome> {
    let request = request.borrow();
    let claimant = request.claimant.trim();
    if claimant.is_empty() {
        return Err(BpmnEngineError::InvalidHumanTaskClaimant);
    }

    let pending_index = find_pending_human_work_index(
        instance,
        request.token_id,
        request.process_id.as_str(),
        request.activity_id.as_str(),
    )?;
    let pending = &mut instance.pending_host_work[pending_index];
    if let Some(existing) = pending.claim.as_ref() {
        if existing.claimant == claimant {
            return Ok(PendingHumanTaskClaimOutcome {
                pending_host_work: pending.clone(),
                changed: false,
            });
        }
        return Err(BpmnEngineError::PendingHostWorkAlreadyClaimed {
            token_id: request.token_id,
            claimed_by: existing.claimant.clone(),
        });
    }

    pending.claim = Some(PendingHostWorkClaim {
        claimant: claimant.to_string(),
        claimed_at_ms: request.claimed_at_ms,
    });
    let pending_host_work = pending.clone();
    instance.sequence += 1;
    instance.updated_at_ms = request.claimed_at_ms;

    Ok(PendingHumanTaskClaimOutcome {
        pending_host_work,
        changed: true,
    })
}

pub(crate) fn release_pending_human_task_impl(
    instance: &mut BpmnInstanceState,
    request: impl Borrow<PendingHumanTaskReleaseRequest>,
) -> Result<PendingHumanTaskReleaseOutcome> {
    let request = request.borrow();
    let claimant = request.claimant.trim();
    if claimant.is_empty() {
        return Err(BpmnEngineError::InvalidHumanTaskClaimant);
    }

    let pending_index = find_pending_human_work_index(
        instance,
        request.token_id,
        request.process_id.as_str(),
        request.activity_id.as_str(),
    )?;
    let pending = &mut instance.pending_host_work[pending_index];
    let Some(existing) = pending.claim.as_ref() else {
        return Err(BpmnEngineError::PendingHostWorkNotClaimed {
            token_id: request.token_id,
        });
    };
    if existing.claimant != claimant {
        return Err(BpmnEngineError::PendingHostWorkClaimReleaseMismatch {
            token_id: request.token_id,
            claimed_by: existing.claimant.clone(),
            requested_by: claimant.to_string(),
        });
    }

    pending.claim = None;
    let pending_host_work = pending.clone();
    instance.sequence += 1;
    instance.updated_at_ms = request.released_at_ms;

    Ok(PendingHumanTaskReleaseOutcome {
        pending_host_work,
        changed: true,
    })
}

fn find_pending_human_work_index(
    instance: &BpmnInstanceState,
    token_id: u64,
    expected_process_id: &str,
    expected_activity_id: &str,
) -> Result<usize> {
    let pending_index = instance
        .pending_host_work
        .iter()
        .position(|pending| pending.token_id == token_id)
        .ok_or_else(|| {
            if instance.pending_host_work.is_empty() {
                return BpmnEngineError::MissingPendingHostWork {
                    instance_id: instance.instance_id.to_string(),
                };
            }
            BpmnEngineError::MissingPendingHostWorkToken {
                instance_id: instance.instance_id.to_string(),
                token_id,
            }
        })?;

    let default_process_id = instance.process.process_id.to_string();
    let pending = &instance.pending_host_work[pending_index];
    let actual_process_id = pending
        .process_id
        .as_deref()
        .unwrap_or(default_process_id.as_str())
        .to_string();
    let actual_activity_id = pending
        .activity_id
        .clone()
        .unwrap_or_else(|| format!("node#{}", pending.node_index));

    if actual_process_id != expected_process_id || actual_activity_id != expected_activity_id {
        return Err(BpmnEngineError::pending_host_work_identity_mismatch(
            instance.instance_id.to_string(),
            token_id,
            expected_process_id.to_string(),
            expected_activity_id.to_string(),
            actual_process_id,
            actual_activity_id,
        ));
    }
    if !matches!(
        pending.kind,
        PendingHostWorkKind::User | PendingHostWorkKind::Manual
    ) {
        return Err(BpmnEngineError::PendingHostWorkNotHumanTask {
            token_id,
            node_index: pending.node_index,
            kind: pending_host_work_kind_name(&pending.kind).to_string(),
        });
    }

    Ok(pending_index)
}

fn pending_host_work_kind_name(kind: &PendingHostWorkKind) -> &'static str {
    match kind {
        PendingHostWorkKind::Send => "send",
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::Script => "script",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}
