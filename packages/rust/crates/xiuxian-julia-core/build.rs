//! Julia crate build-time project harness gate.

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config();
    rust_lang_project_harness::assert_rust_project_harness_build_clean_from_env_with_config(
        &config,
    );
}
