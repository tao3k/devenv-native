mod julia {
    use serde_yaml::Value;
    use xiuxian_wendao_julia::compatibility::link_graph::julia_rerank_provider_selector;

    use crate::resolve_builtin_rerank_runtime_projection_with_settings;

    fn julia_settings_fixture() -> Value {
        serde_yaml::from_str(
            r#"
link_graph:
  retrieval:
    julia_rerank:
      base_url: "http://127.0.0.1:8088"
      route: "/rerank"
      health_route: "/healthz"
      schema_version: "v1"
      timeout_secs: 15
      service_mode: "stream"
      search_config_path: "examples/julia.toml"
      vector_weight: 0.2
      similarity_weight: 0.8
"#,
        )
        .unwrap_or_else(|error| panic!("fixture should parse: {error}"))
    }

    #[test]
    fn resolve_builtin_rerank_runtime_projection_with_settings_projects_generic_fields() {
        let projection =
            resolve_builtin_rerank_runtime_projection_with_settings(&julia_settings_fixture());

        let Some(binding) = projection.binding else {
            panic!("binding should resolve");
        };
        let Some(score_weights) = projection.score_weights else {
            panic!("score weights should resolve");
        };

        assert_eq!(binding.selector, julia_rerank_provider_selector());
        assert_eq!(
            binding.endpoint.base_url.as_deref(),
            Some("http://127.0.0.1:8088")
        );
        assert_eq!(binding.endpoint.route.as_deref(), Some("/rerank"));
        assert_eq!(binding.endpoint.health_route.as_deref(), Some("/healthz"));
        assert_eq!(binding.endpoint.timeout_secs, Some(15));
        assert_eq!(projection.schema_version.as_deref(), Some("v1"));
        assert!((score_weights.vector_weight - 0.2).abs() < f64::EPSILON);
        assert!((score_weights.semantic_weight - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn resolve_builtin_rerank_runtime_projection_with_settings_returns_none_for_empty_weights() {
        let settings: Value = serde_yaml::from_str(
            r#"
link_graph:
  retrieval:
    julia_rerank:
      base_url: "http://127.0.0.1:8088"
      route: "/rerank"
"#,
        )
        .unwrap_or_else(|error| panic!("fixture should parse: {error}"));

        let projection = resolve_builtin_rerank_runtime_projection_with_settings(&settings);

        assert!(projection.score_weights.is_none());
        assert!(projection.schema_version.is_none());
    }
}
