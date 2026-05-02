#[test]
fn enforce_rust_project_harness_gate() {
    let report = rust_lang_project_harness::run_rust_project_harness(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    report.assert_clean();
}
