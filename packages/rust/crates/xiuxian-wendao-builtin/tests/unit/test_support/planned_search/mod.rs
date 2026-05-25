use xiuxian_julia_core::integration_support::{
    julia_planned_search_openai_runtime_config_toml,
    julia_planned_search_vector_store_runtime_config_toml,
};

use crate::test_support::{
    linked_builtin_julia_planned_search_openai_runtime_config_toml,
    linked_builtin_julia_planned_search_vector_store_runtime_config_toml,
};

#[test]
fn linked_builtin_planned_search_runtime_config_helpers_match_julia_source_of_truth() {
    assert_eq!(
        linked_builtin_julia_planned_search_openai_runtime_config_toml(
            "/tmp/vector-store",
            "http://127.0.0.1:9999",
            "http://127.0.0.1:8088",
        ),
        julia_planned_search_openai_runtime_config_toml(
            "/tmp/vector-store",
            "http://127.0.0.1:9999",
            "http://127.0.0.1:8088",
        )
    );
    assert_eq!(
        linked_builtin_julia_planned_search_vector_store_runtime_config_toml(
            "/tmp/vector-store",
            "http://127.0.0.1:8088",
        ),
        julia_planned_search_vector_store_runtime_config_toml(
            "/tmp/vector-store",
            "http://127.0.0.1:8088",
        )
    );
}
