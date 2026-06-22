use std::fs;

use crate::qianji_cli::test_exports::{ControlCliCommand, run_control_command};
use crate::qianji_cli::tests::control_cli::support::{append_empty_control_run, must_ok};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT, ActivityId, ActivityScheduleAdmissionExecutionFlags,
    ActivityScheduleAdmissionInputExecutionFlags, ActivityScheduleAdmissionKind,
    ActivityScheduleAdmissionPlanItem, ActivityScheduleAdmissionRuntimeExecutionFlags,
    ActivityScheduleAdmissionSafetyFlags, ActivityScheduleAdmissionStatus, ActivityTask,
    ActivityType, ArtifactId, ArtifactKind, ArtifactRef, ControlLedger, DuckDbControlLedger,
    IdempotencyKey, RunId, TaskQueue,
};

#[test]
fn run_control_activity_admit_plan_appends_scheduled_activity() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let plan_path = temp_dir.path().join("qianji_schedule_plan.json");
    write_schedule_plan(&plan_path, &run_id)?;

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityAdmitPlan {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: None,
            occurred_at_ms: 10,
            schedule_plan_json_path: plan_path,
            json: true,
        }),
        "control activity-admit-plan json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity admit-plan output should be valid json",
    );

    assert_eq!(json["planItemCount"], 1);
    assert_eq!(json["appendedCount"], 1);
    assert_eq!(json["alreadyRecordedCount"], 0);

    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "should load activity queue",
    );
    assert_eq!(queue.summary.scheduled, 1);
    assert_eq!(
        queue.worker_tasks[0].activity_id.as_str(),
        "activity.episteme.reasoning.test"
    );
    Ok(())
}

#[test]
fn run_control_activity_admit_plan_is_idempotent_for_exact_duplicate() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_empty_control_run(&ledger_path);
    let plan_path = temp_dir.path().join("qianji_schedule_plan.json");
    write_schedule_plan(&plan_path, &run_id)?;

    let command = ControlCliCommand::ActivityAdmitPlan {
        ledger_path: ledger_path.clone(),
        run_id: run_id.as_str().to_string(),
        step_id: None,
        occurred_at_ms: 10,
        schedule_plan_json_path: plan_path,
        json: true,
    };
    must_ok(
        run_control_command(&command),
        "first activity-admit-plan should append",
    );
    let duplicate = must_ok(
        run_control_command(&command),
        "duplicate activity-admit-plan should be idempotent",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&duplicate.rendered),
        "duplicate admit-plan output should be valid json",
    );

    assert_eq!(json["appendedCount"], 0);
    assert_eq!(json["alreadyRecordedCount"], 1);
    Ok(())
}

fn write_schedule_plan(path: &std::path::Path, run_id: &RunId) -> Result<(), String> {
    let item = ActivityScheduleAdmissionPlanItem {
        schedule_item_id: "idf.qianji_schedule_plan.test".to_owned(),
        schedule_contract: ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT.to_owned(),
        admission_kind: ActivityScheduleAdmissionKind::QianjiActivityScheduleAdmissionCandidate,
        qianji_run_id: run_id.as_str().to_owned(),
        activity_task: activity_task()?,
        execution: ActivityScheduleAdmissionExecutionFlags {
            input: ActivityScheduleAdmissionInputExecutionFlags {
                source_text_read: false,
                llm_executed: false,
            },
            runtime: ActivityScheduleAdmissionRuntimeExecutionFlags {
                workflow_executed: false,
                qianji_ledger_mutated: false,
                hot_state_enqueued: false,
            },
        },
        safety: ActivityScheduleAdmissionSafetyFlags {
            source_mutation_allowed: false,
            rdf_mutation_allowed: false,
            ontology_truth: false,
        },
        status: ActivityScheduleAdmissionStatus::PendingQianjiAdmission,
    };
    let content = serde_json::to_string_pretty(&vec![item]).map_err(|error| format!("{error}"))?;
    fs::write(path, content).map_err(|error| format!("should write schedule plan: {error}"))
}

fn activity_task() -> Result<ActivityTask, String> {
    Ok(ActivityTask::new(
        ActivityId::new("activity.episteme.reasoning.test").map_err(|error| format!("{error}"))?,
        ActivityType::new("episteme.ontology.reasoning_fill")
            .map_err(|error| format!("{error}"))?,
        TaskQueue::new("episteme.ontology.reasoning").map_err(|error| format!("{error}"))?,
        IdempotencyKey::new("episteme.ontology.reasoning.test/activity")
            .map_err(|error| format!("{error}"))?,
    )
    .with_input_ref(ArtifactRef {
        artifact_id: ArtifactId::new("artifact.episteme.reasoning.test")
            .map_err(|error| format!("{error}"))?,
        artifact_kind: ArtifactKind::new("episteme.reasoning_fill_item")
            .map_err(|error| format!("{error}"))?,
        uri: "reasoning_fill_plan.json#idf.reasoning_fill_plan.test".to_owned(),
        content_digest: Some("sha256:abc123".to_owned()),
        metadata: serde_json::Value::Null,
    }))
}
