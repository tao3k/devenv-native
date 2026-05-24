use std::{fs, path::Path};

use tempfile::tempdir;
use xiuxian_qianji_control::{ActivityTask, ArtifactRef};
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralFactsReasoningFillPlanRequest,
    EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralFactsReasoningPacketRequest,
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest,
    EpistemeOntologyStructuralFactsRequest, write_episteme_ontology_structural_facts,
    write_episteme_ontology_structural_facts_reasoning_fill_plan,
    write_episteme_ontology_structural_facts_reasoning_ledger_seed,
    write_episteme_ontology_structural_facts_reasoning_packet,
    write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan,
};

use super::fixtures::write_structural_facts_fixture;

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_writes_valid_activity_tasks()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_qianji_run_id("episteme.ontology.reasoning.test");
    let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.fill_item_count, 2);
    assert_eq!(report.object_schedule_item_count, 1);
    assert_eq!(report.relation_schedule_item_count, 1);
    assert_eq!(report.schedule_item_count, 2);
    assert!(report.context_evidence_run_ids.is_empty());
    assert_eq!(report.context_evidence_item_count, 0);
    assert_eq!(report.context_evidence_missing_item_count, 0);
    assert!(!report.execution.input.source_text_read);
    assert!(!report.execution.input.llm_executed);
    assert!(!report.execution.runtime.workflow_executed);
    assert!(!report.execution.runtime.qianji_ledger_mutated);
    assert!(!report.execution.runtime.hot_state_enqueued);
    assert!(!report.safety.source_mutation_allowed);
    assert!(!report.safety.rdf_mutation_allowed);
    assert!(!report.safety.ontology_truth);

    let items: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.qianji_schedule_plan_json)?)?;
    let rows = items
        .as_array()
        .ok_or("schedule plan JSON must be an array")?;
    assert_eq!(rows.len(), 2);
    for row in rows {
        let activity_task: ActivityTask = serde_json::from_value(row["activityTask"].clone())?;
        activity_task.validate()?;
        assert_eq!(
            activity_task.activity_type.as_str(),
            "episteme.ontology.reasoning_fill"
        );
        assert_eq!(
            activity_task.task_queue.as_str(),
            "episteme.ontology.reasoning"
        );
        assert!(activity_task.input_ref.is_some());
        let retry_policy = activity_task
            .retry_policy
            .as_ref()
            .ok_or("schedule task should carry provider retry policy")?;
        assert_eq!(retry_policy.max_attempts, 2);
        assert_eq!(retry_policy.initial_interval_ms, 1_000);
        assert!(
            activity_task
                .metadata
                .get("qianji_llm_activity_request")
                .is_none()
        );
    }

    Ok(())
}

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_writes_openai_prompt_audit()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;
    let extraction_run_root = write_context_evidence_cache_fixture(temp.path(), &fill_plan_json)?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_qianji_run_id("episteme.ontology.reasoning.test")
    .with_evidence_extraction_run_root(&extraction_run_root)
    .with_evidence_extraction_run_id("cache_run")
    .with_openai_compatible_prompt_audit("deepseek/deepseek-v4-pro", 768);
    let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;
    assert_eq!(report.context_evidence_run_ids, vec!["cache_run"]);
    assert_eq!(report.context_evidence_item_count, 2);
    assert_eq!(report.context_evidence_missing_item_count, 0);

    let items: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.qianji_schedule_plan_json)?)?;
    let rows = items
        .as_array()
        .ok_or("schedule plan JSON must be an array")?;
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_prompt_audit_activity_row(row)?;
    }

    Ok(())
}

fn assert_prompt_audit_activity_row(
    row: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let activity_task: ActivityTask = serde_json::from_value(row["activityTask"].clone())?;
    activity_task.validate()?;
    let input_ref = activity_task
        .input_ref
        .as_ref()
        .ok_or("prompt audit task must carry input_ref")?;
    assert_eq!(input_ref.artifact_kind.as_str(), "llm.prompt");
    assert!(Path::new(input_ref.uri.as_str()).exists());

    let audit = &activity_task.metadata["qianji_llm_activity_request"];
    assert_eq!(audit["schema"], "qianji.llm_activity_request_audit.v1");
    assert_eq!(audit["model"], "deepseek/deepseek-v4-pro");
    assert_eq!(audit["max_tokens"], 768);
    let prompt_ref: ArtifactRef = serde_json::from_value(audit["prompt_ref"].clone())?;
    assert_eq!(Some(&prompt_ref), activity_task.input_ref.as_ref());
    let retry_policy = activity_task
        .retry_policy
        .as_ref()
        .ok_or("prompt audit task should carry provider retry policy")?;
    assert_eq!(retry_policy.max_attempts, 2);
    assert_eq!(retry_policy.initial_interval_ms, 1_000);
    let context_ref: ArtifactRef = serde_json::from_value(audit["context_ref"].clone())?;
    assert_eq!(
        context_ref.artifact_kind.as_str(),
        "episteme.reasoning_fill_context"
    );
    assert_prompt_and_context(row, &activity_task, &prompt_ref, &context_ref)
}

fn assert_prompt_and_context(
    row: &serde_json::Value,
    activity_task: &ActivityTask,
    prompt_ref: &ArtifactRef,
    context_ref: &ArtifactRef,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(Path::new(context_ref.uri.as_str()).exists());
    let fill_item_id = row["fillItemId"].as_str().unwrap_or_default();
    let prompt_text = fs::read_to_string(prompt_ref.uri.as_str())?;
    assert!(prompt_text.contains("Return JSON only"));
    assert!(prompt_text.contains(fill_item_id));
    let context_text = fs::read_to_string(context_ref.uri.as_str())?;
    assert!(context_text.contains(fill_item_id));
    assert!(context_text.contains("contextEvidence"));
    assert!(context_text.contains("targetContract"));
    assert!(context_text.contains("Extension source evidence body"));
    let context_json: serde_json::Value = serde_json::from_str(&context_text)?;
    assert_target_contract(row, &context_json)?;
    assert_eq!(
        activity_task.metadata["sourceArtifactRef"]["artifact_kind"],
        "episteme.reasoning_fill_item"
    );
    Ok(())
}

fn assert_target_contract(
    row: &serde_json::Value,
    context_json: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        context_json["targetContract"]["schema"],
        "xiuxian.wendao.episteme.reasoning_target_contract.v1"
    );
    assert_eq!(
        context_json["targetContract"]["targetLedgerFieldGroup"],
        row["fieldGroup"]
    );
    assert_eq!(
        context_json["targetContract"]["objectModelCompatibility"],
        "foundry_style_object_model_v1"
    );
    assert_eq!(
        context_json["targetContract"]["operationalTargetLayer"],
        "object_model"
    );
    assert_eq!(
        context_json["targetContract"]["semanticSourceAuthority"],
        "rdf"
    );
    assert_eq!(
        context_json["targetContract"]["runtimeMutationAllowed"],
        false
    );
    assert_eq!(context_json["targetContract"]["rdfMutationAllowed"], false);
    assert_candidate_patch_shape(row, context_json)
}

fn assert_candidate_patch_shape(
    row: &serde_json::Value,
    context_json: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    match row["fieldGroup"].as_str().unwrap_or_default() {
        "object_proposal" => {
            assert_eq!(
                context_json["targetContract"]["patchKind"],
                "object_model_object_type_candidate"
            );
            assert!(
                context_json["targetContract"]["candidatePatchShape"]["objectType"]
                    .as_object()
                    .is_some()
            );
        }
        "relation_proposal" => {
            assert_eq!(
                context_json["targetContract"]["patchKind"],
                "object_model_link_type_candidate"
            );
            assert!(
                context_json["targetContract"]["candidatePatchShape"]["linkType"]
                    .as_object()
                    .is_some()
            );
        }
        other => return Err(format!("unexpected field group: {other}").into()),
    }
    assert_eq!(
        context_json["targetContract"]["candidatePatchShape"]["patchKind"],
        context_json["targetContract"]["patchKind"]
    );
    assert!(
        context_json["targetContract"]["candidatePatchShape"]
            .as_object()
            .is_some_and(|shape| shape.contains_key("sourceEvidence"))
    );
    Ok(())
}

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_emits_service_catalog_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;
    let mut fill_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fill_plan_json)?)?;
    fill_json[0]["seedKind"] = serde_json::json!("service_catalog_review_slot");
    fill_json[0]["evidenceTargetIntent"] = serde_json::json!("service_catalog_extraction");
    fill_json[0]["evidenceStructureHint"] = serde_json::json!("document_root:service_catalog");
    fill_json[0]["targetLedgerFieldGroup"] = serde_json::json!("service_catalog_review");
    fill_json
        .as_array_mut()
        .ok_or("fill plan must be an array")?
        .truncate(1);
    fs::write(&fill_plan_json, serde_json::to_string_pretty(&fill_json)?)?;
    let extraction_run_root = write_context_evidence_cache_fixture(temp.path(), &fill_plan_json)?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_qianji_run_id("episteme.ontology.reasoning.test")
    .with_evidence_extraction_run_root(&extraction_run_root)
    .with_evidence_extraction_run_id("cache_run")
    .with_openai_compatible_prompt_audit("deepseek/deepseek-v4-pro", 768);
    let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.object_schedule_item_count, 0);
    assert_eq!(report.relation_schedule_item_count, 0);
    assert_eq!(report.service_catalog_schedule_item_count, 1);
    assert_eq!(report.object_instance_schedule_item_count, 0);
    let items: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.qianji_schedule_plan_json)?)?;
    let row = &items.as_array().ok_or("schedule items must be an array")?[0];
    assert_eq!(row["fieldGroup"], "service_catalog_review");
    assert_eq!(row["evidenceTargetIntent"], "service_catalog_extraction");
    let context_ref =
        &row["activityTask"]["metadata"]["qianji_llm_activity_request"]["context_ref"]["uri"];
    let context_text = fs::read_to_string(context_ref.as_str().ok_or("context ref uri")?)?;
    let context_json: serde_json::Value = serde_json::from_str(&context_text)?;
    assert_eq!(
        context_json["targetContract"]["targetLedgerFieldGroup"],
        "service_catalog_review"
    );
    assert_eq!(
        context_json["targetContract"]["evidenceTargetIntent"],
        "service_catalog_extraction"
    );
    assert_eq!(
        context_json["targetContract"]["patchKind"],
        "object_candidate"
    );
    assert!(
        context_json["targetContract"]["candidatePatchShape"]["objectType"]
            .as_object()
            .is_none()
    );

    Ok(())
}

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_shards_service_catalog_table_context()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;
    let mut fill_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fill_plan_json)?)?;
    fill_json[0]["seedKind"] = serde_json::json!("service_catalog_review_slot");
    fill_json[0]["evidenceTargetIntent"] = serde_json::json!("service_catalog_extraction");
    fill_json[0]["evidenceStructureHint"] = serde_json::json!("document_root:service_catalog");
    fill_json[0]["targetLedgerFieldGroup"] = serde_json::json!("service_catalog_review");
    fill_json
        .as_array_mut()
        .ok_or("fill plan must be an array")?
        .truncate(1);
    fs::write(&fill_plan_json, serde_json::to_string_pretty(&fill_json)?)?;
    let extraction_run_root = write_context_evidence_cache_fixture(temp.path(), &fill_plan_json)?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_qianji_run_id("episteme.ontology.reasoning.test")
    .with_evidence_extraction_run_root(&extraction_run_root)
    .with_evidence_extraction_run_id("cache_run")
    .with_openai_compatible_prompt_audit("deepseek/deepseek-v4-pro", 768)
    .with_reasoning_context_shard_mode("service-catalog-table-rows")
    .with_reasoning_context_shard_row_limit(2);
    let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.fill_item_count, 1);
    assert_eq!(report.schedule_item_count, 3);
    assert_eq!(
        report.reasoning_context_shard_mode,
        "service-catalog-table-rows"
    );
    assert_eq!(report.reasoning_context_shard_row_limit, 2);
    assert_eq!(report.reasoning_context_shard_count, 3);
    assert_eq!(report.service_catalog_schedule_item_count, 3);
    let items: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.qianji_schedule_plan_json)?)?;
    let rows = items.as_array().ok_or("schedule items must be an array")?;
    let mut shard_ids = std::collections::BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let shard_id = row["reasoningContextShardId"]
            .as_str()
            .ok_or("missing reasoning context shard id")?;
        assert!(shard_ids.insert(shard_id.to_owned()));
        assert_eq!(
            row["reasoningContextShardIndex"],
            serde_json::json!(index + 1)
        );
        assert_eq!(row["reasoningContextShardCount"], serde_json::json!(3));
        assert_eq!(
            row["activityTask"]["metadata"]["reasoningContextShard"]["shardId"],
            shard_id
        );
        let context_ref =
            &row["activityTask"]["metadata"]["qianji_llm_activity_request"]["context_ref"]["uri"];
        let context_text = fs::read_to_string(context_ref.as_str().ok_or("context ref uri")?)?;
        let context_json: serde_json::Value = serde_json::from_str(&context_text)?;
        assert_eq!(context_json["reasoningContextShard"]["shardId"], shard_id);
        assert!(
            context_json["contextEvidence"][0]["extractedText"]
                .as_str()
                .unwrap_or_default()
                .contains("review only table data rows")
        );
        assert!(
            context_json["contextEvidence"][0]["extractedText"]
                .as_str()
                .unwrap_or_default()
                .contains("| service_item |")
        );
    }
    Ok(())
}

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_filters_target_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;
    let mut fill_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fill_plan_json)?)?;
    fill_json[0]["seedKind"] = serde_json::json!("service_catalog_review_slot");
    fill_json[0]["evidenceTargetIntent"] = serde_json::json!("service_catalog_extraction");
    fill_json[0]["evidenceStructureHint"] = serde_json::json!("document_root:service_catalog");
    fill_json[0]["targetLedgerFieldGroup"] = serde_json::json!("service_catalog_review");
    fs::write(&fill_plan_json, serde_json::to_string_pretty(&fill_json)?)?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_target_ledger_field_group("service_catalog_review")
    .with_evidence_target_intent("service_catalog_extraction")
    .with_limit(1);
    let report = write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.schedule_item_count, 1);
    assert_eq!(report.service_catalog_schedule_item_count, 1);
    assert_eq!(report.skipped_by_limit_count, 0);
    assert_eq!(report.skipped_by_filter_count, 1);
    assert_eq!(
        report.target_ledger_field_group.as_deref(),
        Some("service_catalog_review")
    );
    assert_eq!(
        report.evidence_target_intent.as_deref(),
        Some("service_catalog_extraction")
    );
    let items: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.qianji_schedule_plan_json)?)?;
    let row = &items.as_array().ok_or("schedule items must be an array")?[0];
    assert_eq!(row["fieldGroup"], "service_catalog_review");
    assert_eq!(row["evidenceTargetIntent"], "service_catalog_extraction");
    Ok(())
}

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_rejects_prompt_audit_without_evidence_runs()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_openai_compatible_prompt_audit("deepseek/deepseek-v4-pro", 768);
    let error = match write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    ) {
        Ok(report) => panic!("expected evidence run error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(
        error
            .to_string()
            .contains("require at least one evidence extraction run id")
    );
    Ok(())
}

#[test]
fn structural_facts_reasoning_qianji_schedule_plan_rejects_executed_fill_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;
    let mut fill_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fill_plan_json)?)?;
    fill_json[0]["workflowExecuted"] = serde_json::json!(true);
    fs::write(&fill_plan_json, serde_json::to_string_pretty(&fill_json)?)?;

    let request = EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    );
    let error = match write_episteme_ontology_structural_facts_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    ) {
        Ok(report) => panic!("expected workflow execution error, got report: {report:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("already executed workflow"));
    Ok(())
}

fn write_reasoning_fill_plan_fixture(
    temp_root: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let root = temp_root.join("episteme");
    let corpus_root = temp_root.join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_facts(
        &EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let packet_report = write_episteme_ontology_structural_facts_reasoning_packet(
        &EpistemeOntologyStructuralFactsReasoningPacketRequest::new(
            &structural_report.structural_facts_json,
            "reasoning_packet",
        ),
        root.join("runs/ontology-generation"),
    )?;
    let ledger_seed_report = write_episteme_ontology_structural_facts_reasoning_ledger_seed(
        &EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest::new(
            &packet_report.reasoning_packet_json,
            "reasoning_ledger_seed",
        ),
        root.join("runs/ontology-generation"),
    )?;
    let fill_plan_report = write_episteme_ontology_structural_facts_reasoning_fill_plan(
        &EpistemeOntologyStructuralFactsReasoningFillPlanRequest::new(
            &ledger_seed_report.reasoning_ledger_seed_json,
            "reasoning_fill_plan",
        ),
        root.join("runs/ontology-generation"),
    )?;
    Ok(fill_plan_report.reasoning_fill_plan_json)
}

fn write_context_evidence_cache_fixture(
    temp_root: &std::path::Path,
    fill_plan_json: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let extraction_run_root = temp_root.join("runs/extraction");
    let outputs = extraction_run_root.join("cache_run/outputs");
    fs::create_dir_all(&outputs)?;
    let rows: serde_json::Value = serde_json::from_str(&fs::read_to_string(fill_plan_json)?)?;
    let rows = rows.as_array().ok_or("fill plan must be an array")?;
    for row in rows {
        let file_id = row["fileId"].as_str().ok_or("fileId")?;
        let extracted_text =
            if row["targetLedgerFieldGroup"].as_str() == Some("service_catalog_review") {
                service_catalog_table_fixture()
            } else {
                format!("Extension source evidence body for {file_id}.")
            };
        let output = serde_json::json!({
            "status": "succeeded",
            "queue_id": format!("queue.{file_id}"),
            "file_id": file_id,
            "relative_path": row["relativePath"],
            "category": row["category"],
            "language": row["language"],
            "extraction_route": row["extractionRoute"],
            "source_sha256": row["sourceContentHash"],
            "text_sha256": "sha256:evidence-text",
            "text_char_count": extracted_text.chars().count(),
            "extracted_text": extracted_text,
            "ontology_truth": false,
            "raw_to_rdf_promotion_allowed": false
        });
        fs::write(
            outputs.join(format!("{file_id}.json").replace('/', "_")),
            serde_json::to_string_pretty(&output)?,
        )?;
    }
    Ok(extraction_run_root)
}

fn service_catalog_table_fixture() -> String {
    [
        "Private service catalog evidence.",
        "",
        "| service_item | category | description |",
        "| --- | --- | --- |",
        "| home nursing | nursing | home visit care |",
        "| rehab guidance | rehabilitation | rehabilitation guidance |",
        "| risk assessment | assessment | in-home risk review |",
        "| nutrition guidance | guidance | diet support |",
        "| medication reminder | nursing | medication support |",
    ]
    .join("\n")
}
