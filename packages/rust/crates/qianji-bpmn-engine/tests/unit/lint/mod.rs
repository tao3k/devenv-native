use crate::test_support::MustExt as _;

pub(super) use qianji_bpmn_engine::{
    BpmnSourceFile, DmnSourceFile, LintDomain, lint_bpmn_source, lint_dmn_source,
};

mod bpmn_core;
mod bpmn_loops;
mod bpmn_tasks;
mod compensation;
mod dmn;
mod smoke;
mod transaction;

pub(super) fn bpmn_fixture_source(name: &str) -> BpmnSourceFile {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    BpmnSourceFile::new(name, contents)
}

pub(super) fn dmn_fixture_source(name: &str) -> DmnSourceFile {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    DmnSourceFile::new(name, contents)
}

pub(super) fn assert_lint_json_snapshot(name: &str, value: impl serde::Serialize) {
    insta::with_settings!({
        snapshot_path => "../../snapshots",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}
