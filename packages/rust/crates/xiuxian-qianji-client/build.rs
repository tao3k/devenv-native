//! Qianji client build-time project harness gate.

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config()
        .with_cargo_check_advice_allow_explanation(
            "Qianji client build-time gate blocks warnings and errors while advisory API-shape migrations remain tracked separately.",
        );
    rust_lang_project_harness::assert_rust_project_harness_cargo_check_clean_from_env_with_config(
        &config,
    );
}
