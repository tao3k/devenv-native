use super::{StubHost, call_activity_child_process, call_activity_main_process};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnNodeKind, BpmnPackage, BpmnParseOptions, BpmnSourceFile, BpmnTaskIoSpec,
    BpmnTaskOutputBinding, parse_bpmn_package,
};

mod call_error;
mod call_mixed;
mod conditional_boundary;
mod core;
mod embedded_boundary;
mod embedded_error;
mod embedded_escalation;
mod embedded_mixed;
mod external_boundary;
mod nested;
mod transaction_cancel;
mod transaction_compensation;
mod transaction_completion;
mod transaction_error;
mod transaction_external;
mod transaction_mixed;
mod transaction_mixed_cancel;
mod transaction_mixed_cancel_error;

pub(super) const EMBEDDED_REVIEW_PROCESS_ID: &str =
    "__embedded_subprocess__::main_process::inline_review";
pub(super) const TRANSACTION_PROCESS_ID: &str = "__transaction__::main_process::payment_tx";

pub(super) fn parsed_fixture_package(name: &str) -> BpmnPackage {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    let mut package = parse_bpmn_package(
        &[BpmnSourceFile::new(name, contents)],
        &BpmnParseOptions::default(),
    )
    .must("fixture BPMN should parse");
    attach_optional_runtime_output_io(&mut package);
    package
}

fn attach_optional_runtime_output_io(package: &mut BpmnPackage) {
    for process in &mut package.processes {
        for node in &mut process.nodes {
            if is_host_task(&node.kind) && node.task_io.is_none() {
                node.task_io = Some(runtime_optional_output_io());
            }
        }
    }
}

fn is_host_task(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
            | BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    )
}

fn runtime_optional_output_io() -> BpmnTaskIoSpec {
    [
        "acknowledged",
        "answer",
        "approval",
        "approved",
        "captured",
        "completed_iteration",
        "done",
        "escalated",
        "handled",
        "last_completed",
        "payment_error",
        "refunded",
        "release_timestamp",
        "reserved",
        "released_capture",
        "released_reserve",
        "result",
        "reviewer",
        "timed_out",
        "winner",
    ]
    .into_iter()
    .fold(BpmnTaskIoSpec::new(), |task_io, name| {
        task_io.with_output(BpmnTaskOutputBinding::new(name, name).optional())
    })
}

pub(super) fn node_index(package: &BpmnPackage, process_id: &str, node_id: &str) -> u32 {
    package
        .find_process(process_id)
        .must("process should be present")
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == node_id)
        .must("node should be present")
        .index
}
