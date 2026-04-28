use super::*;

#[cfg(feature = "duckdb")]
use crate::test_exports::{
    BpmnTaskClaimCliCommand, BpmnTaskReleaseCliCommand, BpmnTaskWorklistCliCommand,
};

#[cfg(feature = "duckdb")]
#[test]
fn parse_bpmn_command_accepts_tasks_claim_release_and_worklist() {
    let claim = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "tasks",
                "claim",
                "--instance-id",
                "wf_review",
                "--token-id",
                "7",
                "--process-id",
                "review",
                "--activity-id",
                "review_task",
                "--claimant",
                "alice",
            ])),
            "bpmn tasks claim parse should accept explicit identity",
        ),
        "bpmn tasks claim command should be detected",
    );
    assert_eq!(
        claim,
        BpmnCliCommand::TaskClaim(BpmnTaskClaimCliCommand {
            instance_id: "wf_review".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            token_id: 7,
            process_id: "review".to_string(),
            activity_id: "review_task".to_string(),
            claimant: "alice".to_string(),
        })
    );

    let release = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "tasks",
                "release",
                "--instance-id",
                "wf_review",
                "--token-id",
                "7",
                "--process-id",
                "review",
                "--activity-id",
                "review_task",
                "--claimant",
                "alice",
            ])),
            "bpmn tasks release parse should accept explicit identity",
        ),
        "bpmn tasks release command should be detected",
    );
    assert_eq!(
        release,
        BpmnCliCommand::TaskRelease(BpmnTaskReleaseCliCommand {
            instance_id: "wf_review".to_string(),
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            token_id: 7,
            process_id: "review".to_string(),
            activity_id: "review_task".to_string(),
            claimant: "alice".to_string(),
        })
    );

    let worklist = must_some(
        must_ok(
            parse_bpmn_command(&to_args(&[
                "qianji",
                "bpmn",
                "tasks",
                "worklist",
                "--claimant",
                "alice",
                "--assignment-resource",
                "reviewers",
                "--lane",
                "Reviewer Lane",
            ])),
            "bpmn tasks worklist parse should accept claimant and routing filters",
        ),
        "bpmn tasks worklist command should be detected",
    );
    assert_eq!(
        worklist,
        BpmnCliCommand::TaskWorklist(BpmnTaskWorklistCliCommand {
            checkpoint_backend: BpmnCliCheckpointBackend::LocalDuckDb,
            claimant: Some("alice".to_string()),
            assignment_resource: Some("reviewers".to_string()),
            lane: Some("Reviewer Lane".to_string()),
        })
    );
}
