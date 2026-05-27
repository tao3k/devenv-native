use std::error::Error;

use xiuxian_qianji_control::{
    ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT, ActivityId, ActivityJournalWriteStatus,
    ActivityScheduleAdmissionExecutionFlags, ActivityScheduleAdmissionInputExecutionFlags,
    ActivityScheduleAdmissionKind, ActivityScheduleAdmissionPlanItem,
    ActivityScheduleAdmissionRuntimeExecutionFlags, ActivityScheduleAdmissionSafetyFlags,
    ActivityScheduleAdmissionStatus, ActivitySchedulePlanAdmissionRequest, ControlEvent,
    ControlEventKind, ControlLedger, InMemoryControlLedger, RunId, admit_activity_schedule_plan,
    parse_activity_schedule_plan_json,
};

use super::support::activity_task;

#[test]
fn activity_schedule_plan_admits_generic_activity_tasks() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("episteme.ontology.reasoning.test")?;
    let first_activity_id = ActivityId::new("activity.episteme.first")?;
    let second_activity_id = ActivityId::new("activity.episteme.second")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "admit generic schedule plan".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;

    let request = ActivitySchedulePlanAdmissionRequest::run(
        run_id.clone(),
        10,
        vec![
            plan_item("schedule.first", &run_id, first_activity_id.clone())?,
            plan_item("schedule.second", &run_id, second_activity_id)?,
        ],
    );
    let report = admit_activity_schedule_plan(&ledger, request.clone())?;
    let duplicate = admit_activity_schedule_plan(&ledger, request)?;

    assert_eq!(report.plan_item_count, 2);
    assert_eq!(report.appended_count, 2);
    assert_eq!(report.already_recorded_count, 0);
    assert_eq!(duplicate.appended_count, 0);
    assert_eq!(duplicate.already_recorded_count, 2);
    assert_eq!(
        duplicate.outcomes[0].status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );

    let queue = ledger.load_activity_queue_projection(&run_id, None)?;
    assert_eq!(queue.summary.scheduled, 2);
    assert!(
        queue
            .worker_tasks
            .iter()
            .any(|task| task.activity_id == first_activity_id)
    );

    Ok(())
}

#[test]
fn activity_schedule_plan_rejects_mutating_flags_before_ledger_write() -> Result<(), Box<dyn Error>>
{
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("episteme.ontology.reasoning.test")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "reject unsafe schedule plan".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    let mut item = plan_item(
        "schedule.unsafe",
        &run_id,
        ActivityId::new("activity.episteme.unsafe")?,
    )?;
    item.execution.runtime.qianji_ledger_mutated = true;

    let error = admit_activity_schedule_plan(
        &ledger,
        ActivitySchedulePlanAdmissionRequest::run(run_id.clone(), 10, vec![item]),
    )
    .err()
    .unwrap_or_else(|| panic!("unsafe schedule plan should fail"));

    assert!(
        error.to_string().contains("qianjiLedgerMutated=false"),
        "unexpected error: {error}"
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 1);

    Ok(())
}

#[test]
fn activity_schedule_plan_json_parser_preserves_activity_task_shape() -> Result<(), Box<dyn Error>>
{
    let run_id = RunId::new("episteme.ontology.reasoning.test")?;
    let item = plan_item(
        "schedule.json",
        &run_id,
        ActivityId::new("activity.episteme.json")?,
    )?;
    let json = serde_json::to_string(&vec![item])?;
    let parsed = parse_activity_schedule_plan_json(&json)?;

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].schedule_item_id, "schedule.json");
    assert_eq!(parsed[0].activity_task.task_queue.as_str(), "llm.openai");

    Ok(())
}

fn plan_item(
    schedule_item_id: &str,
    run_id: &RunId,
    activity_id: ActivityId,
) -> Result<ActivityScheduleAdmissionPlanItem, Box<dyn Error>> {
    Ok(ActivityScheduleAdmissionPlanItem {
        schedule_item_id: schedule_item_id.to_owned(),
        schedule_contract: ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT.to_owned(),
        admission_kind: ActivityScheduleAdmissionKind::QianjiActivityScheduleAdmissionCandidate,
        qianji_run_id: run_id.as_str().to_owned(),
        activity_task: activity_task(activity_id)?,
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
    })
}
