#[test]
fn selectors_keep_stable_julia_ids() {
    let provider = julia_rerank_provider_selector();
    let graph_structural = julia_graph_structural_provider_selector();
    let artifact = julia_deployment_artifact_selector();

    assert_eq!(provider.provider.0, JULIA_PLUGIN_ID);
    assert_eq!(provider.capability_id.0, JULIA_RERANK_CAPABILITY_ID);
    assert_eq!(graph_structural.provider.0, JULIA_PLUGIN_ID);
    assert_eq!(
        graph_structural.capability_id.0,
        JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID
    );
    assert_eq!(artifact.plugin_id.0, JULIA_PLUGIN_ID);
    assert_eq!(artifact.artifact_id.0, JULIA_DEPLOYMENT_ARTIFACT_ID);
}

#[test]
fn launch_manifest_round_trips_plugin_launch_spec() {
    let launch = LinkGraphJuliaSearchLaunchManifest {
        launcher_path: DEFAULT_JULIA_SEARCH_LAUNCHER_PATH.to_string(),
        args: vec!["--mode".to_string(), "stream".to_string()],
    };

    let spec: PluginLaunchSpec = launch.clone().into();
    let roundtrip = LinkGraphJuliaSearchLaunchManifest::from(spec);

    assert_eq!(roundtrip, launch);
}

#[test]
fn service_descriptor_builds_plugin_launch_spec() {
    let descriptor = LinkGraphJuliaSearchServiceDescriptor {
        service_mode: Some("stream".to_string()),
        search_config_path: Some(DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH.to_string()),
    };

    let spec = descriptor.plugin_launch_spec(DEFAULT_JULIA_SEARCH_LAUNCHER_PATH);

    assert_eq!(spec.launcher_path, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH);
    assert_eq!(
        spec.args,
        vec![
            "--mode",
            "stream",
            "--config",
            DEFAULT_JULIA_SEARCH_EXAMPLE_CONFIG_PATH,
        ]
    );
}
