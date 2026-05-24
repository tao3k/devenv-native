use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::ontology::reasoning_context_shard::{
    EpistemeReasoningContextShard, EpistemeReasoningContextShardSource,
    REASONING_CONTEXT_SHARD_MODE_DISABLED, plan_episteme_reasoning_context_shard_texts,
    validate_reasoning_context_shard_mode,
};
use crate::ontology::reasoning_target::{
    OBJECT_FIELD_GROUP, OBJECT_INSTANCE_FIELD_GROUP, RELATION_FIELD_GROUP,
    SERVICE_CATALOG_FIELD_GROUP,
};

use super::{
    evidence::{ContextEvidenceByFileId, ContextEvidenceRow, read_context_evidence_by_file_id},
    input::{ReasoningFillPlanInputRow, read_reasoning_fill_plan_rows},
    types::{
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags,
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem,
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit,
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport,
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanSafetyFlags,
        QIANJI_SCHEDULE_PLAN_REPORT_SCHEMA_VERSION, QianjiActivityRetryPolicyShape,
        QianjiActivityTaskShape, QianjiArtifactRefShape, QianjiSchedulePlanOutputPaths,
    },
    write::{write_json, write_schedule_plan_org, write_schedule_plan_tsv},
};

const SCHEDULE_CONTRACT: &str = "xiuxian.qianji.control.activity_schedule_admission_plan.v1";
const ADMISSION_KIND: &str = "qianji_activity_schedule_admission_candidate";
const ACTIVITY_TYPE: &str = "episteme.ontology.reasoning_fill";
const TASK_QUEUE: &str = "episteme.ontology.reasoning";
const INPUT_ARTIFACT_KIND: &str = "episteme.reasoning_fill_item";
const STATUS_PENDING: &str = "pending_qianji_admission";
const PROMPT_ARTIFACT_KIND: &str = "llm.prompt";
const CONTEXT_ARTIFACT_KIND: &str = "episteme.reasoning_fill_context";
const QIANJI_LLM_ACTIVITY_REQUEST_AUDIT_SCHEMA: &str = "qianji.llm_activity_request_audit.v1";
const TARGET_CONTRACT_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_target_contract.v1";
const OBJECT_MODEL_SCHEMA_REF: &str =
    "https://wendao.ai/schema/episteme/object-model-v1.schema.json";
const OBJECT_MODEL_COMPATIBILITY: &str = "foundry_style_object_model_v1";
const OBJECT_MODEL_TARGET_LAYER: &str = "object_model";
const RDF_SOURCE_AUTHORITY: &str = "rdf";
const OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND: &str = "object_model_object_type_candidate";
const OBJECT_MODEL_LINK_TYPE_PATCH_KIND: &str = "object_model_link_type_candidate";

/// Compile a structural facts reasoning fill plan into Qianji schedule inputs.
///
/// # Errors
///
/// Returns an error when the fill-plan artifact is missing, malformed, has no
/// selectable rows, attempts to mark ontology truth or mutation, contains
/// duplicate fill item ids, or output artifacts cannot be written.
pub fn write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
    request: &EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport> {
    validate_run_id(&request.run_id)?;
    if request.limit == 0 {
        bail!("Qianji schedule-plan limit must be greater than zero");
    }
    validate_reasoning_context_shard_mode(request.reasoning_context_shard_mode.as_str())?;
    if request.reasoning_context_shard_row_limit == 0 {
        bail!("Qianji schedule-plan reasoning context shard row limit must be greater than zero");
    }
    validate_optional_filter(
        "target ledger field group",
        request.target_ledger_field_group.as_deref(),
    )?;
    validate_optional_filter(
        "evidence target intent",
        request.evidence_target_intent.as_deref(),
    )?;
    if let Some(prompt_audit) = &request.openai_compatible_prompt_audit {
        validate_prompt_audit(prompt_audit)?;
        if request.evidence_extraction_run_ids.is_empty() {
            bail!(
                "OpenAI-compatible Episteme reasoning schedule plans require at least one evidence extraction run id"
            );
        }
    }
    let qianji_run_id = request
        .qianji_run_id
        .clone()
        .unwrap_or_else(|| format!("episteme.ontology.reasoning.{}", request.run_id));
    validate_run_id(qianji_run_id.as_str())?;

    let fill_rows = read_reasoning_fill_plan_rows(request.reasoning_fill_plan_json.as_path())?;
    let context_evidence_by_file_id = load_context_evidence(request, &fill_rows)?;
    let paths = QianjiSchedulePlanOutputPaths::new(run_root.as_ref(), request.run_id.as_str());
    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;
    let build_options = ScheduleBuildOptions {
        schedule_run_id: request.run_id.as_str(),
        qianji_run_id: qianji_run_id.as_str(),
        limit: request.limit,
        target_ledger_field_group: request.target_ledger_field_group.as_deref(),
        evidence_target_intent: request.evidence_target_intent.as_deref(),
        reasoning_context_shard_mode: request.reasoning_context_shard_mode.as_str(),
        reasoning_context_shard_row_limit: request.reasoning_context_shard_row_limit,
        paths: &paths,
        prompt_audit: request.openai_compatible_prompt_audit.as_ref(),
        context_evidence_by_file_id: &context_evidence_by_file_id,
    };
    let selection = build_schedule_items(&fill_rows, &build_options)?;
    write_schedule_plan_tsv(paths.schedule_plan_tsv.as_path(), &selection.items)?;
    write_json(paths.schedule_plan_json.as_path(), &selection.items)?;
    let report = build_report(
        request,
        &ReportBuildContext {
            qianji_run_id,
            paths: &paths,
            items: &selection.items,
            selected_fill_item_count: selection.selected_fill_item_count,
            skipped_by_limit_count: selection.skipped_by_limit_count,
            skipped_by_filter_count: selection.skipped_by_filter_count,
            context_evidence_by_file_id: &context_evidence_by_file_id,
        },
    );
    write_schedule_plan_org(paths.schedule_plan_org.as_path(), &report, &selection.items)?;
    write_json(paths.report_json.as_path(), &report)?;
    Ok(report)
}

struct ScheduleItemSelection {
    items: Vec<EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem>,
    selected_fill_item_count: usize,
    skipped_by_limit_count: usize,
    skipped_by_filter_count: usize,
}

struct ScheduleBuildOptions<'a> {
    schedule_run_id: &'a str,
    qianji_run_id: &'a str,
    limit: usize,
    target_ledger_field_group: Option<&'a str>,
    evidence_target_intent: Option<&'a str>,
    reasoning_context_shard_mode: &'a str,
    reasoning_context_shard_row_limit: usize,
    paths: &'a QianjiSchedulePlanOutputPaths,
    prompt_audit: Option<&'a EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit>,
    context_evidence_by_file_id: &'a ContextEvidenceByFileId,
}

fn build_schedule_items(
    fill_rows: &[ReasoningFillPlanInputRow],
    options: &ScheduleBuildOptions<'_>,
) -> Result<ScheduleItemSelection> {
    let mut seen_fill_item_ids = BTreeSet::new();
    let mut seen_schedule_item_ids = BTreeSet::new();
    let mut schedule_items = Vec::new();
    let mut selected_fill_item_count = 0;
    let mut skipped_by_limit_count = 0;
    let mut skipped_by_filter_count = 0;

    for fill in fill_rows {
        if !seen_fill_item_ids.insert(fill.fill_item_id.as_str()) {
            bail!(
                "duplicate reasoning fill-plan item id: {}",
                fill.fill_item_id
            );
        }
        if !fill_matches_filters(
            fill,
            options.target_ledger_field_group,
            options.evidence_target_intent,
        ) {
            skipped_by_filter_count += 1;
            continue;
        }
        if schedule_items.len() >= options.limit {
            skipped_by_limit_count += 1;
            continue;
        }
        let item_contexts = schedule_item_contexts(
            fill,
            options.prompt_audit,
            options.context_evidence_by_file_id,
            options.reasoning_context_shard_mode,
            options.reasoning_context_shard_row_limit,
        )?;
        selected_fill_item_count += 1;
        for item_context in item_contexts {
            if schedule_items.len() >= options.limit {
                skipped_by_limit_count += 1;
                continue;
            }
            let item = schedule_item(
                fill,
                options.schedule_run_id,
                options.qianji_run_id,
                options.paths,
                options.prompt_audit,
                &item_context,
            )?;
            if !seen_schedule_item_ids.insert(item.schedule_item_id.clone()) {
                bail!(
                    "duplicate Qianji schedule-plan item id: {}",
                    item.schedule_item_id
                );
            }
            schedule_items.push(item);
        }
    }

    if schedule_items.is_empty() {
        bail!("Qianji schedule-plan selection produced no rows");
    }
    Ok(ScheduleItemSelection {
        items: schedule_items,
        selected_fill_item_count,
        skipped_by_limit_count,
        skipped_by_filter_count,
    })
}

fn fill_matches_filters(
    fill: &ReasoningFillPlanInputRow,
    target_ledger_field_group: Option<&str>,
    evidence_target_intent: Option<&str>,
) -> bool {
    if let Some(expected) = target_ledger_field_group
        && fill.target_ledger_field_group != expected
    {
        return false;
    }
    if let Some(expected) = evidence_target_intent
        && fill.evidence_target_intent != expected
    {
        return false;
    }
    true
}

struct ScheduleItemContext {
    reasoning_context_shard: Option<EpistemeReasoningContextShard>,
    context_evidence: Vec<ContextEvidenceRow>,
}

fn schedule_item_contexts(
    fill: &ReasoningFillPlanInputRow,
    prompt_audit: Option<&EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit>,
    context_evidence_by_file_id: &ContextEvidenceByFileId,
    reasoning_context_shard_mode: &str,
    reasoning_context_shard_row_limit: usize,
) -> Result<Vec<ScheduleItemContext>> {
    if prompt_audit.is_none() {
        return Ok(vec![ScheduleItemContext {
            reasoning_context_shard: None,
            context_evidence: Vec::new(),
        }]);
    }
    let context_evidence = context_evidence_for_fill(fill, context_evidence_by_file_id)?;
    if reasoning_context_shard_mode == REASONING_CONTEXT_SHARD_MODE_DISABLED
        || fill.target_ledger_field_group != SERVICE_CATALOG_FIELD_GROUP
    {
        return Ok(vec![ScheduleItemContext {
            reasoning_context_shard: None,
            context_evidence: context_evidence.to_vec(),
        }]);
    }

    let mut contexts = Vec::new();
    for row in context_evidence {
        let source = EpistemeReasoningContextShardSource {
            subject_id: fill.fill_item_id.as_str(),
            context_id: row.queue_id.as_str(),
            target_field_group: fill.target_ledger_field_group.as_str(),
            service_catalog_field_group: SERVICE_CATALOG_FIELD_GROUP,
            extracted_text: row.extracted_text.as_str(),
        };
        for shard_text in plan_episteme_reasoning_context_shard_texts(
            &source,
            reasoning_context_shard_mode,
            reasoning_context_shard_row_limit,
        )? {
            let mut sharded_row = row.clone();
            if let Some(shard) = &shard_text.shard {
                sharded_row.cache_output_path =
                    format!("{}#{}", sharded_row.cache_output_path, shard.shard_id);
            }
            sharded_row.extracted_text = shard_text.extracted_text;
            sharded_row.text_sha256 = shard_text.text_sha256;
            sharded_row.text_char_count = shard_text.text_char_count;
            contexts.push(ScheduleItemContext {
                reasoning_context_shard: shard_text.shard,
                context_evidence: vec![sharded_row],
            });
        }
    }
    Ok(contexts)
}

fn schedule_item(
    fill: &ReasoningFillPlanInputRow,
    schedule_run_id: &str,
    qianji_run_id: &str,
    paths: &QianjiSchedulePlanOutputPaths,
    prompt_audit: Option<&EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit>,
    item_context: &ScheduleItemContext,
) -> Result<EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem> {
    let schedule_key = schedule_key(fill, item_context.reasoning_context_shard.as_ref());
    let schedule_item_id = stable_schedule_item_id(schedule_run_id, schedule_key.as_str());
    let activity_id = stable_activity_id(qianji_run_id, schedule_key.as_str());
    let source_ref = qianji_input_ref(
        fill,
        schedule_run_id,
        schedule_item_id.as_str(),
        item_context.reasoning_context_shard.as_ref(),
    );
    let (input_ref, llm_request_audit) = if let Some(prompt_audit) = prompt_audit {
        let refs = write_prompt_audit_artifacts(
            fill,
            schedule_item_id.as_str(),
            paths,
            prompt_audit,
            &source_ref,
            item_context.context_evidence.as_slice(),
            item_context.reasoning_context_shard.as_ref(),
        )?;
        (refs.prompt_ref, Some(refs.request_audit))
    } else {
        (source_ref.clone(), None)
    };
    let activity_task = QianjiActivityTaskShape {
        activity_id: activity_id.clone(),
        activity_type: ACTIVITY_TYPE.to_owned(),
        task_queue: TASK_QUEUE.to_owned(),
        input_ref,
        idempotency_key: format!("{qianji_run_id}/{activity_id}"),
        retry_policy: Some(QianjiActivityRetryPolicyShape::llm_provider_default()),
        metadata: qianji_task_metadata(
            fill,
            schedule_item_id.as_str(),
            &source_ref,
            llm_request_audit,
            item_context.reasoning_context_shard.as_ref(),
        ),
    };
    let reasoning_context_shard = item_context.reasoning_context_shard.as_ref();
    Ok(
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem {
            schedule_item_id,
            schedule_contract: SCHEDULE_CONTRACT,
            admission_kind: ADMISSION_KIND,
            qianji_run_id: qianji_run_id.to_owned(),
            fill_item_id: fill.fill_item_id.clone(),
            workflow_key: fill.workflow_key.clone(),
            activity_kind: fill.activity_kind.clone(),
            seed_id: fill.seed_id.clone(),
            seed_kind: fill.seed_kind.clone(),
            packet_id: fill.packet_id.clone(),
            evidence_target_intent: fill.evidence_target_intent.clone(),
            evidence_anchor_kind: fill.evidence_anchor_kind.clone(),
            evidence_structure_hint: fill.evidence_structure_hint.clone(),
            document_id: fill.document_id.clone(),
            document_anchor_id: fill.document_anchor_id.clone(),
            file_id: fill.file_id.clone(),
            evidence_id: fill.evidence_id.clone(),
            field_group: fill.target_ledger_field_group.clone(),
            reasoning_context_shard_id: reasoning_context_shard.map(|shard| shard.shard_id.clone()),
            reasoning_context_shard_index: reasoning_context_shard.map(|shard| shard.shard_index),
            reasoning_context_shard_count: reasoning_context_shard.map(|shard| shard.shard_count),
            reasoning_context_shard_row_start: reasoning_context_shard.map(|shard| shard.row_start),
            reasoning_context_shard_row_end: reasoning_context_shard.map(|shard| shard.row_end),
            activity_task,
            execution:
                EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags::inactive(),
            safety: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanSafetyFlags {
                source_mutation_allowed: false,
                rdf_mutation_allowed: false,
                ontology_truth: false,
            },
            status: STATUS_PENDING,
        },
    )
}

fn qianji_input_ref(
    fill: &ReasoningFillPlanInputRow,
    schedule_run_id: &str,
    schedule_item_id: &str,
    reasoning_context_shard: Option<&EpistemeReasoningContextShard>,
) -> QianjiArtifactRefShape {
    let digest = fill_item_digest(fill, schedule_run_id, reasoning_context_shard);
    let uri = reasoning_context_shard.map_or_else(
        || format!("reasoning_fill_plan.json#{}", fill.fill_item_id),
        |shard| {
            format!(
                "reasoning_fill_plan.json#{}#{}",
                fill.fill_item_id, shard.shard_id
            )
        },
    );
    QianjiArtifactRefShape {
        artifact_id: format!("artifact.{schedule_item_id}"),
        artifact_kind: INPUT_ARTIFACT_KIND.to_owned(),
        uri,
        content_digest: format!("sha256:{digest}"),
        metadata: serde_json::json!({
            "fillItemId": fill.fill_item_id,
            "seedId": fill.seed_id,
            "packetId": fill.packet_id,
            "evidenceTargetIntent": fill.evidence_target_intent,
            "evidenceAnchorKind": fill.evidence_anchor_kind,
            "evidenceStructureHint": fill.evidence_structure_hint,
            "documentId": fill.document_id,
            "documentAnchorId": fill.document_anchor_id,
            "evidenceId": fill.evidence_id,
            "sourceContentHash": fill.source_content_hash,
            "reasoningContextShard": reasoning_context_shard,
        }),
    }
}

fn qianji_task_metadata(
    fill: &ReasoningFillPlanInputRow,
    schedule_item_id: &str,
    source_ref: &QianjiArtifactRefShape,
    llm_request_audit: Option<serde_json::Value>,
    reasoning_context_shard: Option<&EpistemeReasoningContextShard>,
) -> serde_json::Value {
    let mut metadata = serde_json::json!({
        "scheduleItemId": schedule_item_id,
        "workflowKey": fill.workflow_key,
        "activityKind": fill.activity_kind,
        "qianjiActivityContract": fill.qianji_activity_contract,
        "seedId": fill.seed_id,
        "seedKind": fill.seed_kind,
        "packetId": fill.packet_id,
        "reasoningTaskKind": fill.reasoning_task_kind,
        "evidenceTargetIntent": fill.evidence_target_intent,
        "evidenceAnchorKind": fill.evidence_anchor_kind,
        "evidenceStructureHint": fill.evidence_structure_hint,
        "documentId": fill.document_id,
        "documentAnchorId": fill.document_anchor_id,
        "fileId": fill.file_id,
        "domainId": fill.domain_id,
        "sourceContractId": fill.source_contract_id,
        "relativePath": fill.relative_path,
        "category": fill.category,
        "language": fill.language,
        "extractionRoute": fill.extraction_route,
        "sourceContentHash": fill.source_content_hash,
        "evidenceId": fill.evidence_id,
        "targetLedgerFieldGroup": fill.target_ledger_field_group,
        "outputContract": fill.output_contract,
        "reviewDecisionRequired": fill.review_decision_required,
        "promotionDecisionRequired": fill.promotion_decision_required,
        "sourceArtifactRef": source_ref,
        "reasoningContextShard": reasoning_context_shard,
    });
    if let Some(llm_request_audit) = llm_request_audit {
        metadata["qianji_llm_activity_request"] = llm_request_audit;
    }
    metadata
}

struct PromptAuditArtifacts {
    prompt_ref: QianjiArtifactRefShape,
    request_audit: serde_json::Value,
}

fn write_prompt_audit_artifacts(
    fill: &ReasoningFillPlanInputRow,
    schedule_item_id: &str,
    paths: &QianjiSchedulePlanOutputPaths,
    prompt_audit: &EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit,
    source_ref: &QianjiArtifactRefShape,
    context_evidence: &[ContextEvidenceRow],
    reasoning_context_shard: Option<&EpistemeReasoningContextShard>,
) -> Result<PromptAuditArtifacts> {
    let context_text = serde_json::to_string_pretty(&reasoning_context_json(
        fill,
        source_ref,
        context_evidence,
        reasoning_context_shard,
    ))
    .context("failed to serialize reasoning context artifact")?;
    let context_ref = write_local_text_artifact(
        paths.context_artifact_dir.as_path(),
        schedule_item_id,
        "context.json",
        CONTEXT_ARTIFACT_KIND,
        context_text.as_str(),
        serde_json::json!({
            "scheduleItemId": schedule_item_id,
            "fillItemId": fill.fill_item_id,
            "sourceArtifactId": source_ref.artifact_id,
            "reasoningContextShard": reasoning_context_shard,
        }),
    )?;

    let prompt_text = reasoning_prompt_text(fill);
    let prompt_ref = write_local_text_artifact(
        paths.prompt_artifact_dir.as_path(),
        schedule_item_id,
        "prompt.txt",
        PROMPT_ARTIFACT_KIND,
        prompt_text.as_str(),
        serde_json::json!({
            "scheduleItemId": schedule_item_id,
            "fillItemId": fill.fill_item_id,
            "contextArtifactId": context_ref.artifact_id,
            "sourceArtifactId": source_ref.artifact_id,
            "reasoningContextShard": reasoning_context_shard,
        }),
    )?;

    Ok(PromptAuditArtifacts {
        request_audit: serde_json::json!({
            "schema": QIANJI_LLM_ACTIVITY_REQUEST_AUDIT_SCHEMA,
            "model": prompt_audit.model.as_str(),
            "prompt_ref": prompt_ref.clone(),
            "context_ref": context_ref,
            "temperature_millis": 0,
            "max_tokens": prompt_audit.max_tokens,
            "response_schema_ref": null,
            "request_metadata": {
                "activityType": ACTIVITY_TYPE,
                "taskQueue": TASK_QUEUE,
                "fillItemId": fill.fill_item_id,
                "reasoningContextShard": reasoning_context_shard,
                "reviewOnly": true,
                "rdfMutationAllowed": false,
            },
        }),
        prompt_ref,
    })
}

fn reasoning_context_json(
    fill: &ReasoningFillPlanInputRow,
    source_ref: &QianjiArtifactRefShape,
    context_evidence: &[ContextEvidenceRow],
    reasoning_context_shard: Option<&EpistemeReasoningContextShard>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "xiuxian.wendao.episteme.reasoning_fill_context.v1",
        "sourceArtifactRef": source_ref,
        "reasoningContextShard": reasoning_context_shard,
        "fillItem": {
            "fillItemId": fill.fill_item_id,
            "workflowKey": fill.workflow_key,
            "activityKind": fill.activity_kind,
            "qianjiActivityContract": fill.qianji_activity_contract,
            "seedId": fill.seed_id,
            "seedKind": fill.seed_kind,
            "packetId": fill.packet_id,
            "reasoningTaskKind": fill.reasoning_task_kind,
            "evidenceTargetIntent": fill.evidence_target_intent,
            "evidenceAnchorKind": fill.evidence_anchor_kind,
            "evidenceStructureHint": fill.evidence_structure_hint,
            "documentId": fill.document_id,
            "documentAnchorId": fill.document_anchor_id,
            "fileId": fill.file_id,
            "domainId": fill.domain_id,
            "sourceContractId": fill.source_contract_id,
            "relativePath": fill.relative_path,
            "category": fill.category,
            "language": fill.language,
            "extractionRoute": fill.extraction_route,
            "sourceContentHash": fill.source_content_hash,
            "evidenceId": fill.evidence_id,
            "targetLedgerFieldGroup": fill.target_ledger_field_group,
            "outputContract": fill.output_contract,
            "reviewDecisionRequired": fill.review_decision_required,
            "promotionDecisionRequired": fill.promotion_decision_required,
        },
        "targetContract": reasoning_target_contract_json(fill),
        "contextEvidence": context_evidence,
        "safety": {
            "sourceTextRead": false,
            "sourceMutationAllowed": false,
            "rdfMutationAllowed": false,
            "ontologyTruth": false,
        },
    })
}

fn reasoning_target_contract_json(fill: &ReasoningFillPlanInputRow) -> serde_json::Value {
    let (patch_kind, candidate_patch_shape) =
        target_candidate_patch_contract(fill.target_ledger_field_group.as_str(), fill);
    serde_json::json!({
        "schema": TARGET_CONTRACT_SCHEMA,
        "objectModelSchemaRef": OBJECT_MODEL_SCHEMA_REF,
        "objectModelCompatibility": OBJECT_MODEL_COMPATIBILITY,
        "operationalTargetLayer": OBJECT_MODEL_TARGET_LAYER,
        "semanticSourceAuthority": RDF_SOURCE_AUTHORITY,
        "targetLedgerFieldGroup": fill.target_ledger_field_group,
        "evidenceTargetIntent": fill.evidence_target_intent,
        "evidenceStructureHint": fill.evidence_structure_hint,
        "patchKind": patch_kind,
        "allowedPatchKinds": [patch_kind],
        "reviewMode": "proposal_patch_only",
        "runtimeMutationAllowed": false,
        "rdfMutationAllowed": false,
        "rules": [
            "Use the targetContract candidatePatchShape when evidence supports an object model candidate.",
            "Honor evidenceTargetIntent and evidenceStructureHint before proposing a candidate.",
            "For service_catalog_review or object_instance_review, do not propose an object model type; propose concrete object candidates or return blockers.",
            "Do not block solely because an external proposal schema is absent.",
            "Every candidate patch must cite contextEvidence by fileId, relativePath, and a short quote.",
            "Return review-only object model proposal patches; do not mutate source files, runtime objects, or RDF."
        ],
        "requiredResponseFields": [
            "schema",
            "status",
            "fillItemId",
            "targetLedgerFieldGroup",
            "reviewSummary",
            "candidatePatchCount",
            "candidatePatches",
            "blockers",
            "rdfMutation"
        ],
        "candidatePatchShape": candidate_patch_shape,
    })
}

fn target_candidate_patch_contract(
    field_group: &str,
    fill: &ReasoningFillPlanInputRow,
) -> (&'static str, serde_json::Value) {
    match field_group {
        OBJECT_FIELD_GROUP => object_type_patch_contract(fill),
        RELATION_FIELD_GROUP => link_type_patch_contract(fill),
        SERVICE_CATALOG_FIELD_GROUP => service_catalog_patch_contract(fill),
        OBJECT_INSTANCE_FIELD_GROUP => object_instance_patch_contract(fill),
        _ => ledger_patch_contract(field_group, fill),
    }
}

fn object_type_patch_contract(
    fill: &ReasoningFillPlanInputRow,
) -> (&'static str, serde_json::Value) {
    (
        OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND,
        serde_json::json!({
            "patchKind": OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND,
            "fillItemId": fill.fill_item_id,
            "targetLedgerFieldGroup": OBJECT_FIELD_GROUP,
            "objectType": {
                "domain": "episteme://extension-or-common-domain",
                "apiName": "PascalCaseObjectType",
                "displayName": "human-facing label grounded in evidence",
                "pluralDisplayName": "human-facing plural label grounded in evidence",
                "status": "preview",
                "rdfClass": "stable RDF class IRI suggestion; review assigns final source truth",
                "primaryKey": ["sourceId"],
                "displayNameProperty": "name",
                "titleProperty": "name",
                "interfaces": [],
                "visibility": "private"
            },
            "propertyTypes": [
                {
                    "apiName": "name",
                    "displayName": "Name",
                    "valueType": "string",
                    "required": true,
                    "indexed": true,
                    "searchPolicy": "full_text"
                }
            ],
            "sourceEvidence": [source_evidence_shape(fill, "why the quote supports this candidate")],
            "confidence": "low|medium|high",
            "reviewNotes": "ambiguities, rejected interpretations, and RDF promotion questions"
        }),
    )
}

fn link_type_patch_contract(fill: &ReasoningFillPlanInputRow) -> (&'static str, serde_json::Value) {
    (
        OBJECT_MODEL_LINK_TYPE_PATCH_KIND,
        serde_json::json!({
            "patchKind": OBJECT_MODEL_LINK_TYPE_PATCH_KIND,
            "fillItemId": fill.fill_item_id,
            "targetLedgerFieldGroup": RELATION_FIELD_GROUP,
            "linkType": {
                "domain": "episteme://extension-or-common-domain",
                "apiName": "sourceObjectToTargetObject",
                "displayName": "evidence-grounded relation label",
                "status": "preview",
                "rdfProperty": "stable RDF property IRI suggestion; review assigns final source truth",
                "fromObjectType": "SourceObjectType",
                "toObjectType": "TargetObjectType",
                "cardinality": "many_to_many",
                "fromApiName": "sourceObjects",
                "toApiName": "targetObjects",
                "inverseApiName": "targetObjectToSourceObject",
                "foreignKeyProperty": "sourceObjectId"
            },
            "endpointObjectTypes": [
                {
                    "apiName": "SourceObjectType",
                    "displayName": "evidence-grounded source object label",
                    "rdfClass": "stable RDF class IRI suggestion"
                },
                {
                    "apiName": "TargetObjectType",
                    "displayName": "evidence-grounded target object label",
                    "rdfClass": "stable RDF class IRI suggestion"
                }
            ],
            "sourceEvidence": [
                source_evidence_shape(fill, "why the quote supports this candidate relation")
            ],
            "confidence": "low|medium|high",
            "reviewNotes": "ambiguities, rejected interpretations, and RDF promotion questions"
        }),
    )
}

fn service_catalog_patch_contract(
    fill: &ReasoningFillPlanInputRow,
) -> (&'static str, serde_json::Value) {
    object_candidate_patch_contract(
        fill,
        SERVICE_CATALOG_FIELD_GROUP,
        "stable service item key grounded in the source",
        "ltc.service_item or another reviewed class key",
        "service item label grounded in evidence",
        "why the quote supports this service catalog object candidate",
        "catalog row boundaries, rejected interpretations, and RDF promotion questions",
    )
}

fn object_instance_patch_contract(
    fill: &ReasoningFillPlanInputRow,
) -> (&'static str, serde_json::Value) {
    object_candidate_patch_contract(
        fill,
        OBJECT_INSTANCE_FIELD_GROUP,
        "stable object instance key grounded in the source",
        "reviewed class key for this instance",
        "object instance label grounded in evidence",
        "why the quote supports this object instance candidate",
        "instance boundaries, rejected interpretations, and RDF promotion questions",
    )
}

fn object_candidate_patch_contract(
    fill: &ReasoningFillPlanInputRow,
    field_group: &str,
    provisional_key: &str,
    ontology_class_key: &str,
    label: &str,
    evidence_reason: &str,
    review_notes: &str,
) -> (&'static str, serde_json::Value) {
    (
        "object_candidate",
        serde_json::json!({
            "patchKind": "object_candidate",
            "fillItemId": fill.fill_item_id,
            "targetLedgerFieldGroup": field_group,
            "provisionalObjectKey": provisional_key,
            "ontologyClassKey": ontology_class_key,
            "label": label,
            "sourceEvidence": [source_evidence_shape(fill, evidence_reason)],
            "confidence": "low|medium|high",
            "reviewNotes": review_notes
        }),
    )
}

fn ledger_patch_contract(
    field_group: &str,
    fill: &ReasoningFillPlanInputRow,
) -> (&'static str, serde_json::Value) {
    (
        "ledger_candidate_patch",
        serde_json::json!({
            "patchKind": "ledger_candidate",
            "fillItemId": fill.fill_item_id,
            "targetLedgerFieldGroup": field_group,
            "proposal": "evidence-grounded review-only proposal",
            "sourceEvidence": [source_evidence_shape(fill, "why the quote supports this candidate")],
            "confidence": "low|medium|high",
            "reviewNotes": "ambiguities or rejected interpretations"
        }),
    )
}

fn source_evidence_shape(fill: &ReasoningFillPlanInputRow, reason: &str) -> serde_json::Value {
    serde_json::json!({
        "fileId": fill.file_id,
        "relativePath": fill.relative_path,
        "quote": "short verbatim evidence quote",
        "reason": reason
    })
}

fn load_context_evidence(
    request: &EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    fill_rows: &[ReasoningFillPlanInputRow],
) -> Result<ContextEvidenceByFileId> {
    if request.evidence_extraction_run_ids.is_empty() {
        return Ok(ContextEvidenceByFileId::new());
    }
    let extraction_run_root = request
        .evidence_extraction_run_root
        .as_ref()
        .with_context(|| "Qianji schedule-plan context evidence requires an extraction run root")?;
    read_context_evidence_by_file_id(
        extraction_run_root.as_path(),
        &request.evidence_extraction_run_ids,
        fill_rows,
    )
}

fn context_evidence_for_fill<'a>(
    fill: &ReasoningFillPlanInputRow,
    context_evidence_by_file_id: &'a ContextEvidenceByFileId,
) -> Result<&'a [ContextEvidenceRow]> {
    let rows = context_evidence_by_file_id
        .get(fill.file_id.as_str())
        .map_or([].as_slice(), Vec::as_slice);
    if rows.is_empty() {
        bail!(
            "reasoning fill item `{}` has no materialized context evidence for file `{}`",
            fill.fill_item_id,
            fill.file_id
        );
    }
    for row in rows {
        if row.source_sha256 != fill.source_content_hash {
            bail!(
                "reasoning fill item `{}` context evidence `{}` source hash mismatch",
                fill.fill_item_id,
                row.queue_id
            );
        }
        if row.relative_path != fill.relative_path {
            bail!(
                "reasoning fill item `{}` context evidence `{}` relative path mismatch",
                fill.fill_item_id,
                row.queue_id
            );
        }
    }
    Ok(rows)
}

fn reasoning_prompt_text(fill: &ReasoningFillPlanInputRow) -> String {
    format!(
        r#"You are executing an Episteme ontology reasoning review task.

Return JSON only. Do not include Markdown fences or prose outside JSON.

Required output shape:
{{
  "schema": "xiuxian.wendao.episteme.reasoning_fill_review.v1",
  "status": "review_only",
  "fillItemId": "{fill_item_id}",
  "targetLedgerFieldGroup": "{field_group}",
  "reviewSummary": "...",
  "candidatePatchCount": 0,
  "candidatePatches": [],
  "blockers": [],
  "rdfMutation": false
}}

Rules:
- Treat the attached context as a review request, not ontology truth.
- Read targetContract from the attached context and use its candidatePatchShape for any supported proposal.
- If reasoningContextShard is present in the attached context, review only that shard and do not emit candidates outside its row_start..row_end window.
- For object_proposal or relation_proposal, do not block solely because another schema is missing; targetContract is the active patch contract for this review.
- Set candidatePatchCount to the exact length of candidatePatches.
- Do not claim source-file mutation or RDF mutation.
- If evidence is insufficient, return an empty candidatePatches array and explain blockers.
- Preserve the exact fillItemId from the context.
"#,
        fill_item_id = fill.fill_item_id,
        field_group = fill.target_ledger_field_group,
    )
}

fn write_local_text_artifact(
    artifact_dir: &Path,
    schedule_item_id: &str,
    filename: &str,
    artifact_kind: &str,
    content: &str,
    metadata: serde_json::Value,
) -> Result<QianjiArtifactRefShape> {
    fs::create_dir_all(artifact_dir)
        .with_context(|| format!("failed to create `{}`", artifact_dir.display()))?;
    let path = artifact_dir.join(format!("{schedule_item_id}.{filename}"));
    fs::write(path.as_path(), content)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    let digest = sha256_hex(content);
    Ok(QianjiArtifactRefShape {
        artifact_id: format!("artifact.{schedule_item_id}.{filename}"),
        artifact_kind: artifact_kind.to_owned(),
        uri: path.display().to_string(),
        content_digest: format!("sha256:{digest}"),
        metadata,
    })
}

fn validate_prompt_audit(
    prompt_audit: &EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit,
) -> Result<()> {
    if prompt_audit.model.trim().is_empty() {
        bail!("OpenAI-compatible prompt audit model must not be blank");
    }
    if prompt_audit.max_tokens == 0 {
        bail!("OpenAI-compatible prompt audit max tokens must be greater than zero");
    }
    Ok(())
}

fn validate_optional_filter(name: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value
        && value.trim().is_empty()
    {
        bail!("Qianji schedule-plan {name} filter must not be blank");
    }
    Ok(())
}

struct ReportBuildContext<'a> {
    qianji_run_id: String,
    paths: &'a QianjiSchedulePlanOutputPaths,
    items: &'a [EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem],
    selected_fill_item_count: usize,
    skipped_by_limit_count: usize,
    skipped_by_filter_count: usize,
    context_evidence_by_file_id: &'a ContextEvidenceByFileId,
}

fn build_report(
    request: &EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    context: &ReportBuildContext<'_>,
) -> EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport {
    let object_schedule_item_count = context
        .items
        .iter()
        .filter(|item| item.field_group == OBJECT_FIELD_GROUP)
        .count();
    let relation_schedule_item_count = context
        .items
        .iter()
        .filter(|item| item.field_group == RELATION_FIELD_GROUP)
        .count();
    let service_catalog_schedule_item_count = context
        .items
        .iter()
        .filter(|item| item.field_group == SERVICE_CATALOG_FIELD_GROUP)
        .count();
    let object_instance_schedule_item_count = context
        .items
        .iter()
        .filter(|item| item.field_group == OBJECT_INSTANCE_FIELD_GROUP)
        .count();
    let reasoning_context_shard_count = context
        .items
        .iter()
        .filter(|item| item.reasoning_context_shard_id.is_some())
        .count();
    let context_evidence_item_count = if request.evidence_extraction_run_ids.is_empty() {
        0
    } else {
        context
            .items
            .iter()
            .map(|item| {
                context
                    .context_evidence_by_file_id
                    .get(item.file_id.as_str())
                    .map_or(0, Vec::len)
            })
            .sum()
    };
    let context_evidence_missing_item_count = if request.evidence_extraction_run_ids.is_empty() {
        0
    } else {
        context
            .items
            .iter()
            .filter(|item| {
                !context
                    .context_evidence_by_file_id
                    .contains_key(item.file_id.as_str())
            })
            .count()
    };
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport {
        schema_version: QIANJI_SCHEDULE_PLAN_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        qianji_run_id: context.qianji_run_id.clone(),
        reasoning_fill_plan_json: request.reasoning_fill_plan_json.clone(),
        run_dir: context.paths.run_dir.clone(),
        qianji_schedule_plan_tsv: context.paths.schedule_plan_tsv.clone(),
        qianji_schedule_plan_json: context.paths.schedule_plan_json.clone(),
        qianji_schedule_plan_org: context.paths.schedule_plan_org.clone(),
        qianji_schedule_plan_report_json: context.paths.report_json.clone(),
        fill_item_count: context.selected_fill_item_count,
        object_schedule_item_count,
        relation_schedule_item_count,
        service_catalog_schedule_item_count,
        object_instance_schedule_item_count,
        schedule_item_count: context.items.len(),
        skipped_by_limit_count: context.skipped_by_limit_count,
        skipped_by_filter_count: context.skipped_by_filter_count,
        reasoning_context_shard_mode: request.reasoning_context_shard_mode.clone(),
        reasoning_context_shard_row_limit: request.reasoning_context_shard_row_limit,
        reasoning_context_shard_count,
        target_ledger_field_group: request.target_ledger_field_group.clone(),
        evidence_target_intent: request.evidence_target_intent.clone(),
        context_evidence_run_ids: request.evidence_extraction_run_ids.clone(),
        context_evidence_item_count,
        context_evidence_missing_item_count,
        execution:
            EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags::inactive(),
        safety: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanSafetyFlags {
            source_mutation_allowed: false,
            rdf_mutation_allowed: false,
            ontology_truth: false,
        },
    }
}

fn schedule_key(
    fill: &ReasoningFillPlanInputRow,
    reasoning_context_shard: Option<&EpistemeReasoningContextShard>,
) -> String {
    reasoning_context_shard.map_or_else(
        || fill.fill_item_id.clone(),
        |shard| format!("{}:{}", fill.fill_item_id, shard.shard_id),
    )
}

fn stable_schedule_item_id(schedule_run_id: &str, schedule_key: &str) -> String {
    let digest = Sha256::digest(format!("{schedule_run_id}:{schedule_key}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("structural_facts.qianji_schedule_plan.{suffix}")
}

fn stable_activity_id(qianji_run_id: &str, schedule_key: &str) -> String {
    let digest = Sha256::digest(format!("{qianji_run_id}:{schedule_key}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("activity.episteme_ontology_reasoning_fill.{suffix}")
}

fn fill_item_digest(
    fill: &ReasoningFillPlanInputRow,
    schedule_run_id: &str,
    reasoning_context_shard: Option<&EpistemeReasoningContextShard>,
) -> String {
    let payload = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        schedule_run_id,
        fill.fill_item_id,
        fill.seed_id,
        fill.packet_id,
        fill.document_id,
        fill.document_anchor_id,
        fill.evidence_id,
        fill.source_content_hash,
        reasoning_context_shard.map_or("unsharded", |shard| shard.shard_id.as_str())
    );
    sha256_hex(payload.as_str())
}

fn sha256_hex(payload: &str) -> String {
    format!("{:x}", Sha256::digest(payload.as_bytes()))
}

fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("invalid run id `{run_id}`; use ASCII letters, digits, '.', '_', or '-'");
    }
    Ok(())
}
