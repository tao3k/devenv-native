use std::{fs, path::Path};

use tempfile::tempdir;
use xiuxian_qianji_control::{ActivityTask, ArtifactRef};
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralIdfReasoningFillPlanRequest,
    EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest,
    EpistemeOntologyStructuralIdfReasoningPacketRequest,
    EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest,
    EpistemeOntologyStructuralIdfRequest, write_episteme_ontology_structural_idf,
    write_episteme_ontology_structural_idf_reasoning_fill_plan,
    write_episteme_ontology_structural_idf_reasoning_ledger_seed,
    write_episteme_ontology_structural_idf_reasoning_packet,
    write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan,
};

use super::fixtures::write_structural_idf_fixture;

#[test]
fn structural_idf_reasoning_qianji_schedule_plan_writes_valid_activity_tasks()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;

    let request = EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_qianji_run_id("episteme.ontology.reasoning.test");
    let report = write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    assert_eq!(report.fill_item_count, 2);
    assert_eq!(report.object_schedule_item_count, 1);
    assert_eq!(report.relation_schedule_item_count, 1);
    assert_eq!(report.schedule_item_count, 2);
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
fn structural_idf_reasoning_qianji_schedule_plan_writes_openai_prompt_audit()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;

    let request = EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    )
    .with_qianji_run_id("episteme.ontology.reasoning.test")
    .with_openai_compatible_prompt_audit("openrouter/deepseek/deepseek-chat-v3.1", 768);
    let report = write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan(
        &request,
        temp.path().join("runs/ontology-generation"),
    )?;

    let items: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report.qianji_schedule_plan_json)?)?;
    let rows = items
        .as_array()
        .ok_or("schedule plan JSON must be an array")?;
    assert_eq!(rows.len(), 2);
    for row in rows {
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
        assert_eq!(audit["model"], "openrouter/deepseek/deepseek-chat-v3.1");
        assert_eq!(audit["max_tokens"], 768);
        let prompt_ref: ArtifactRef = serde_json::from_value(audit["prompt_ref"].clone())?;
        assert_eq!(Some(&prompt_ref), activity_task.input_ref.as_ref());
        let context_ref: ArtifactRef = serde_json::from_value(audit["context_ref"].clone())?;
        assert_eq!(
            context_ref.artifact_kind.as_str(),
            "episteme.reasoning_fill_context"
        );
        assert!(Path::new(context_ref.uri.as_str()).exists());
        let prompt_text = fs::read_to_string(prompt_ref.uri.as_str())?;
        assert!(prompt_text.contains("Return JSON only"));
        assert!(prompt_text.contains(row["fillItemId"].as_str().unwrap_or_default()));
        let context_text = fs::read_to_string(context_ref.uri.as_str())?;
        assert!(context_text.contains(row["fillItemId"].as_str().unwrap_or_default()));
        assert_eq!(
            activity_task.metadata["sourceArtifactRef"]["artifact_kind"],
            "episteme.reasoning_fill_item"
        );
    }

    Ok(())
}

#[test]
fn structural_idf_reasoning_qianji_schedule_plan_rejects_executed_fill_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let fill_plan_json = write_reasoning_fill_plan_fixture(temp.path())?;
    let mut fill_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fill_plan_json)?)?;
    fill_json[0]["workflowExecuted"] = serde_json::json!(true);
    fs::write(&fill_plan_json, serde_json::to_string_pretty(&fill_json)?)?;

    let request = EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest::new(
        &fill_plan_json,
        "qianji_schedule_plan",
    );
    let error = match write_episteme_ontology_structural_idf_reasoning_qianji_schedule_plan(
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
    write_structural_idf_fixture(&root, &corpus_root, "expected")?;
    let structural_report = write_episteme_ontology_structural_idf(
        &EpistemeOntologyStructuralIdfRequest::new(&root, &corpus_root, "structural_seed"),
        root.join("runs/structure"),
    )?;
    let packet_report = write_episteme_ontology_structural_idf_reasoning_packet(
        &EpistemeOntologyStructuralIdfReasoningPacketRequest::new(
            &structural_report.structural_idf_json,
            "reasoning_packet",
        ),
        root.join("runs/ontology-generation"),
    )?;
    let ledger_seed_report = write_episteme_ontology_structural_idf_reasoning_ledger_seed(
        &EpistemeOntologyStructuralIdfReasoningLedgerSeedRequest::new(
            &packet_report.reasoning_packet_json,
            "reasoning_ledger_seed",
        ),
        root.join("runs/ontology-generation"),
    )?;
    let fill_plan_report = write_episteme_ontology_structural_idf_reasoning_fill_plan(
        &EpistemeOntologyStructuralIdfReasoningFillPlanRequest::new(
            &ledger_seed_report.reasoning_ledger_seed_json,
            "reasoning_fill_plan",
        ),
        root.join("runs/ontology-generation"),
    )?;
    Ok(fill_plan_report.reasoning_fill_plan_json)
}
