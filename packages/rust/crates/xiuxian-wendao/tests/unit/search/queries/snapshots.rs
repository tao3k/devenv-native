use serde::Serialize;

pub fn assert_query_json_snapshot(name: &str, value: impl Serialize) {
    insta::with_settings!({
        snapshot_path => concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/search/queries"),
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}
