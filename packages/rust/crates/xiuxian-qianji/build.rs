//! Qianji build-time project harness gate.

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(
            "Qianji still carries typed-DTO and workflow-source-admission advisory migrations; warning/error findings remain blocking while advisory cleanup continues through unit policy tests.",
        );
    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
