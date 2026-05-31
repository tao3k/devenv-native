//! Build-time project harness gate.

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(
            "The build-time rs-harness gate is active; existing advisory findings remain visible through rs-harness check while package-specific cleanup continues.",
        );
    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
