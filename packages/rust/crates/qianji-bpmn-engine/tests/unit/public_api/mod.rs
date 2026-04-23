use crate::test_support::MustExt as _;
mod checkpoint;
mod dmn;
mod parser;

use qianji_bpmn_engine::BpmnSourceFile;

fn fixture_source(name: &str) -> BpmnSourceFile {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    BpmnSourceFile::new(name, contents)
}
