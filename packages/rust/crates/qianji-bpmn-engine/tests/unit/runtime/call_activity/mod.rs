use super::{StubHost, call_activity_child_process, call_activity_main_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnPackage, BpmnParseOptions, BpmnSourceFile, parse_bpmn_package};

mod core;
mod nested;
mod transaction_cancel;
mod transaction_completion;
mod transaction_error;

pub(super) const EMBEDDED_REVIEW_PROCESS_ID: &str =
    "__embedded_subprocess__::main_process::inline_review";
pub(super) const TRANSACTION_PROCESS_ID: &str = "__transaction__::main_process::payment_tx";

pub(super) fn parsed_fixture_package(name: &str) -> BpmnPackage {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    parse_bpmn_package(
        &[BpmnSourceFile::new(name, contents)],
        &BpmnParseOptions::default(),
    )
    .must("fixture BPMN should parse")
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
