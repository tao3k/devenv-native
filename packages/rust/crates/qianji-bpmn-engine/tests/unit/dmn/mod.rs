use crate::test_support::MustExt as _;
use qianji_bpmn_engine::DmnSourceFile;

mod core;
mod date;
mod datetime;
mod document;
mod numeric;
mod snapshot;
mod time;

fn fixture_source(name: &str) -> DmnSourceFile {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    DmnSourceFile::new(name, contents)
}

fn assert_dmn_json_snapshot(name: &str, value: impl serde::Serialize) {
    insta::with_settings!({
        snapshot_path => "../../snapshots",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}
