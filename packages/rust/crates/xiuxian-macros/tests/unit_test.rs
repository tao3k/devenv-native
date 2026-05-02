//! Cargo entry point for xiuxian-macros unit tests.

macro_rules! crate_testing_gate {
    () => {
        #[test]
        fn enforce_rust_project_harness_gate() {
            let report = rust_lang_project_harness::run_rust_project_harness(std::path::Path::new(
                env!("CARGO_MANIFEST_DIR"),
            ))
            .unwrap_or_else(|error| panic!("{error}"));
            report.assert_clean();
        }
    };
}

crate_testing_gate!();

#[path = "unit/macros.rs"]
mod macros;
#[path = "unit/xiuxian_config_api_key_policy.rs"]
mod xiuxian_config_api_key_policy;
