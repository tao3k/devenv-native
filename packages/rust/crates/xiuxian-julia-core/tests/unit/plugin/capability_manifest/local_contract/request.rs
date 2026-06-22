#[test]
fn capability_manifest_request_batch_materializes_rows() {
    let batch = build_julia_plugin_capability_manifest_request_batch(&[
        JuliaPluginCapabilityManifestRequestRow {
            plugin_id: "xiuxian-julia-core".into(),
            repository_id: "repo-julia".into(),
            capability_filter: Some("graph-structural".into()),
            include_disabled: true.into(),
        },
    ])
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().fields().len(), 4);
    assert_eq!(
        batch.schema().field(0).name(),
        JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(2).name(),
        JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN
    );
    assert!(batch.schema().field(2).is_nullable());
}
