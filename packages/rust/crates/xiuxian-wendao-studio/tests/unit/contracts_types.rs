use specta_typescript::{BigIntExportBehavior, Typescript};
use xiuxian_wendao_studio::contracts::studio_type_collection;

#[test]
fn contracts_feature_exports_plugin_artifact_type_collection() {
    let exported = Typescript::new()
        .bigint(BigIntExportBehavior::Number)
        .export(&studio_type_collection())
        .unwrap_or_else(|error| panic!("export studio contracts typescript bindings: {error}"));

    assert!(exported.contains("UiPluginArtifact"));
    assert!(exported.contains("UiPluginLaunchSpec"));
    assert!(exported.contains("UiPluginTransportKind"));
}
