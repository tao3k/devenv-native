use specta_typescript::{BigIntExportBehavior, Typescript};

use super::{studio_frontend_type_collection, studio_type_collection};

#[test]
fn studio_type_collection_exports_generic_plugin_artifact_types_only() {
    let exported = Typescript::new()
        .bigint(BigIntExportBehavior::Number)
        .export(&studio_type_collection())
        .unwrap_or_else(|error| panic!("export studio typescript bindings: {error}"));

    assert!(exported.contains("UiPluginArtifact"));
    assert!(exported.contains("UiPluginLaunchSpec"));
    assert!(!exported.contains("UiCompatDeploymentArtifact"));
    assert!(!exported.contains("UiJuliaDeploymentArtifact"));
}

#[test]
fn studio_frontend_type_collection_exports_frontend_runtime_types() {
    let exported = Typescript::new()
        .bigint(BigIntExportBehavior::Number)
        .export(&studio_frontend_type_collection())
        .unwrap_or_else(|error| panic!("export frontend studio typescript bindings: {error}"));

    assert!(exported.contains("UiCapabilities"));
    assert!(exported.contains("UiSearchContract"));
    assert!(exported.contains("UiConfig"));
    assert!(exported.contains("SearchResponse"));
    assert!(exported.contains("MarkdownAnalysisResponse"));
    assert!(exported.contains("CodeAstAnalysisResponse"));
    assert!(exported.contains("DocumentExtractResult"));
    assert!(exported.contains("DocumentExtractResource"));
    assert!(exported.contains("DocumentExtractJobSubmitRequest"));
    assert!(exported.contains("DocumentExtractJobStatus"));
    assert!(exported.contains("DocumentExtractJobsStatus"));
    assert!(exported.contains("maxRunningConversions"));
    assert!(exported.contains("queuedJobs"));
    assert!(exported.contains("sourceFormat"));
    assert!(exported.contains("totalResources"));
    assert!(exported.contains("totalPages"));
    assert!(!exported.contains("PdfExtractResult"));
    assert!(exported.contains("Topology3dPayload"));
    assert!(exported.contains("ApiError"));
}
