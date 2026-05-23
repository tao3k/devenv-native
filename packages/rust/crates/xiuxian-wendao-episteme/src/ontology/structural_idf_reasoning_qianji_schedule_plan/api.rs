use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use super::{
    input::{ReasoningFillPlanInputRow, read_reasoning_fill_plan_rows},
    types::{
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags,
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanItem,
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit,
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanReport,
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest,
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanSafetyFlags,
        QIANJI_SCHEDULE_PLAN_REPORT_SCHEMA_VERSION, QianjiActivityTaskShape,
        QianjiArtifactRefShape, QianjiSchedulePlanOutputPaths,
    },
    write::{write_json, write_schedule_plan_org, write_schedule_plan_tsv},
};

const SCHEDULE_CONTRACT: &str = "xiuxian.qianji.control.activity_schedule_admission_plan.v1";
const ADMISSION_KIND: &str = "qianji_activity_schedule_admission_candidate";
const ACTIVITY_TYPE: &str = "episteme.ontology.reasoning_fill";
const TASK_QUEUE: &str = "episteme.ontology.reasoning";
const INPUT_ARTIFACT_KIND: &str = "episteme.reasoning_fill_item";
const STATUS_PENDING: &str = "pending_qianji_admission";
const OBJECT_FIELD_GROUP: &str = "object_proposal";
const RELATION_FIELD_GROUP: &str = "relation_proposal";
const PROMPT_ARTIFACT_KIND: &str = "llm.prompt";
const CONTEXT_ARTIFACT_KIND: &str = "episteme.reasoning_fill_context";
const QIANJI_LLM_ACTIVITY_REQUEST_AUDIT_SCHEMA: &str = "qianji.llm_activity_request_audit.v1";

/// Compile a structural IDF reasoning fill plan into Qianji schedule inputs.
///
/// # Errors
///
/// Returns an error when the fill-plan artifact is missing, malformed, has no
/// selectable rows, attempts to mark ontology truth or mutation, contains
/// duplicate fill item ids, or output artifacts cannot be written.
pub fn write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan(
    request: &EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanReport> {
    validate_run_id(&request.run_id)?;
    if request.limit == 0 {
        bail!("Qianji schedule-plan limit must be greater than zero");
    }
    if let Some(prompt_audit) = &request.openai_compatible_prompt_audit {
        validate_prompt_audit(prompt_audit)?;
    }
    let qianji_run_id = request
        .qianji_run_id
        .clone()
        .unwrap_or_else(|| format!("episteme.ontology.reasoning.{}", request.run_id));
    validate_run_id(qianji_run_id.as_str())?;

    let fill_rows = read_reasoning_fill_plan_rows(request.reasoning_fill_plan_json.as_path())?;
    let paths = QianjiSchedulePlanOutputPaths::new(run_root.as_ref(), request.run_id.as_str());
    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;
    let (schedule_items, skipped_by_limit_count) = build_schedule_items(
        &fill_rows,
        request.run_id.as_str(),
        qianji_run_id.as_str(),
        request.limit,
        &paths,
        request.openai_compatible_prompt_audit.as_ref(),
    )?;
    write_schedule_plan_tsv(paths.schedule_plan_tsv.as_path(), &schedule_items)?;
    write_json(paths.schedule_plan_json.as_path(), &schedule_items)?;
    let report = build_report(
        request,
        qianji_run_id,
        &paths,
        &schedule_items,
        skipped_by_limit_count,
    );
    write_schedule_plan_org(paths.schedule_plan_org.as_path(), &report, &schedule_items)?;
    write_json(paths.report_json.as_path(), &report)?;
    Ok(report)
}

fn build_schedule_items(
    fill_rows: &[ReasoningFillPlanInputRow],
    schedule_run_id: &str,
    qianji_run_id: &str,
    limit: usize,
    paths: &QianjiSchedulePlanOutputPaths,
    prompt_audit: Option<&EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit>,
) -> Result<(
    Vec<EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanItem>,
    usize,
)> {
    let mut seen_fill_item_ids = BTreeSet::new();
    let mut seen_schedule_item_ids = BTreeSet::new();
    let mut schedule_items = Vec::new();
    let mut skipped_by_limit_count = 0;

    for fill in fill_rows {
        if !seen_fill_item_ids.insert(fill.fill_item_id.as_str()) {
            bail!(
                "duplicate reasoning fill-plan item id: {}",
                fill.fill_item_id
            );
        }
        if seen_fill_item_ids.len() > limit {
            skipped_by_limit_count += 1;
            continue;
        }
        let item = schedule_item(fill, schedule_run_id, qianji_run_id, paths, prompt_audit)?;
        if !seen_schedule_item_ids.insert(item.schedule_item_id.clone()) {
            bail!(
                "duplicate Qianji schedule-plan item id: {}",
                item.schedule_item_id
            );
        }
        schedule_items.push(item);
    }

    if schedule_items.is_empty() {
        bail!("Qianji schedule-plan selection produced no rows");
    }
    Ok((schedule_items, skipped_by_limit_count))
}

fn schedule_item(
    fill: &ReasoningFillPlanInputRow,
    schedule_run_id: &str,
    qianji_run_id: &str,
    paths: &QianjiSchedulePlanOutputPaths,
    prompt_audit: Option<&EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit>,
) -> Result<EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanItem> {
    let schedule_item_id = stable_schedule_item_id(schedule_run_id, fill.fill_item_id.as_str());
    let activity_id = stable_activity_id(qianji_run_id, fill.fill_item_id.as_str());
    let source_ref = qianji_input_ref(fill, schedule_run_id, schedule_item_id.as_str());
    let (input_ref, llm_request_audit) = if let Some(prompt_audit) = prompt_audit {
        let refs = write_prompt_audit_artifacts(
            fill,
            schedule_item_id.as_str(),
            paths,
            prompt_audit,
            &source_ref,
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
        metadata: qianji_task_metadata(
            fill,
            schedule_item_id.as_str(),
            &source_ref,
            llm_request_audit,
        ),
    };
    Ok(
        EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanItem {
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
            document_id: fill.document_id.clone(),
            document_anchor_id: fill.document_anchor_id.clone(),
            file_id: fill.file_id.clone(),
            evidence_id: fill.evidence_id.clone(),
            field_group: fill.target_ledger_field_group.clone(),
            activity_task,
            execution:
                EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags::inactive(),
            safety: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanSafetyFlags {
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
) -> QianjiArtifactRefShape {
    let digest = fill_item_digest(fill, schedule_run_id);
    QianjiArtifactRefShape {
        artifact_id: format!("artifact.{schedule_item_id}"),
        artifact_kind: INPUT_ARTIFACT_KIND.to_owned(),
        uri: format!("reasoning_fill_plan.json#{}", fill.fill_item_id),
        content_digest: format!("sha256:{digest}"),
        metadata: serde_json::json!({
            "fillItemId": fill.fill_item_id,
            "seedId": fill.seed_id,
            "packetId": fill.packet_id,
            "documentId": fill.document_id,
            "documentAnchorId": fill.document_anchor_id,
            "evidenceId": fill.evidence_id,
            "sourceContentHash": fill.source_content_hash,
        }),
    }
}

fn qianji_task_metadata(
    fill: &ReasoningFillPlanInputRow,
    schedule_item_id: &str,
    source_ref: &QianjiArtifactRefShape,
    llm_request_audit: Option<serde_json::Value>,
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
    prompt_audit: &EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit,
    source_ref: &QianjiArtifactRefShape,
) -> Result<PromptAuditArtifacts> {
    let context_text = serde_json::to_string_pretty(&reasoning_context_json(fill, source_ref))
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
) -> serde_json::Value {
    serde_json::json!({
        "schema": "xiuxian.wendao.episteme.reasoning_fill_context.v1",
        "sourceArtifactRef": source_ref,
        "fillItem": {
            "fillItemId": fill.fill_item_id,
            "workflowKey": fill.workflow_key,
            "activityKind": fill.activity_kind,
            "qianjiActivityContract": fill.qianji_activity_contract,
            "seedId": fill.seed_id,
            "seedKind": fill.seed_kind,
            "packetId": fill.packet_id,
            "reasoningTaskKind": fill.reasoning_task_kind,
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
        "safety": {
            "sourceTextRead": false,
            "sourceMutationAllowed": false,
            "rdfMutationAllowed": false,
            "ontologyTruth": false,
        },
    })
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
    prompt_audit: &EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit,
) -> Result<()> {
    if prompt_audit.model.trim().is_empty() {
        bail!("OpenAI-compatible prompt audit model must not be blank");
    }
    if prompt_audit.max_tokens == 0 {
        bail!("OpenAI-compatible prompt audit max tokens must be greater than zero");
    }
    Ok(())
}

fn build_report(
    request: &EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest,
    qianji_run_id: String,
    paths: &QianjiSchedulePlanOutputPaths,
    items: &[EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanItem],
    skipped_by_limit_count: usize,
) -> EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanReport {
    let object_schedule_item_count = items
        .iter()
        .filter(|item| item.field_group == OBJECT_FIELD_GROUP)
        .count();
    let relation_schedule_item_count = items
        .iter()
        .filter(|item| item.field_group == RELATION_FIELD_GROUP)
        .count();
    EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanReport {
        schema_version: QIANJI_SCHEDULE_PLAN_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        qianji_run_id,
        reasoning_fill_plan_json: request.reasoning_fill_plan_json.clone(),
        run_dir: paths.run_dir.clone(),
        qianji_schedule_plan_tsv: paths.schedule_plan_tsv.clone(),
        qianji_schedule_plan_json: paths.schedule_plan_json.clone(),
        qianji_schedule_plan_org: paths.schedule_plan_org.clone(),
        qianji_schedule_plan_report_json: paths.report_json.clone(),
        fill_item_count: items.len(),
        object_schedule_item_count,
        relation_schedule_item_count,
        schedule_item_count: items.len(),
        skipped_by_limit_count,
        execution: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags::inactive(
        ),
        safety: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanSafetyFlags {
            source_mutation_allowed: false,
            rdf_mutation_allowed: false,
            ontology_truth: false,
        },
    }
}

fn stable_schedule_item_id(schedule_run_id: &str, fill_item_id: &str) -> String {
    let digest = Sha256::digest(format!("{schedule_run_id}:{fill_item_id}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("idf.qianji_schedule_plan.{suffix}")
}

fn stable_activity_id(qianji_run_id: &str, fill_item_id: &str) -> String {
    let digest = Sha256::digest(format!("{qianji_run_id}:{fill_item_id}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("activity.episteme_ontology_reasoning_fill.{suffix}")
}

fn fill_item_digest(fill: &ReasoningFillPlanInputRow, schedule_run_id: &str) -> String {
    let payload = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        schedule_run_id,
        fill.fill_item_id,
        fill.seed_id,
        fill.packet_id,
        fill.document_id,
        fill.document_anchor_id,
        fill.evidence_id,
        fill.source_content_hash
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
