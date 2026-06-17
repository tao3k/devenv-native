//! Gateway-owned model routing admission for document extraction.

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DocumentExtractModelRoute;

#[derive(Debug, Clone, Copy)]
pub(super) struct DocumentExtractRouteSourceIdentity<'a> {
    pub(super) path: &'a Path,
    pub(super) sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ImageDocumentExtractRouteConfig;

impl ImageDocumentExtractRouteConfig {
    pub(super) fn from_model_routing_config(model_routing: Option<&()>) -> Result<Self, String> {
        image_document_extract_route_config_with_model_routing(model_routing, &|_| None)
    }
}

pub(super) fn image_document_extract_route_config_with_model_routing(
    model_routing: Option<&()>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ImageDocumentExtractRouteConfig, String> {
    let _ = (model_routing, lookup);
    Ok(ImageDocumentExtractRouteConfig)
}

pub(super) async fn image_document_extract_model_route_with_config(
    source: &Path,
    profile: &str,
    config: ImageDocumentExtractRouteConfig,
) -> Result<Option<DocumentExtractModelRoute>, String> {
    let _ = (source, profile, config);
    Ok(None)
}

pub(super) async fn image_document_extract_model_route_for_source_identity(
    source: DocumentExtractRouteSourceIdentity<'_>,
    profile: &str,
    config: ImageDocumentExtractRouteConfig,
) -> Result<Option<DocumentExtractModelRoute>, String> {
    let _ = (source, profile, config);
    Ok(None)
}

pub(super) fn document_extract_route_manifest_matches(
    output_dir: &Path,
    model_route: Option<&DocumentExtractModelRoute>,
    profile: &str,
) -> bool {
    let _ = (output_dir, model_route, profile);
    true
}

pub(super) fn write_document_extract_route_manifest(
    output_dir: &Path,
    model_route: Option<&DocumentExtractModelRoute>,
    profile: &str,
) -> Result<(), String> {
    let _ = (output_dir, model_route, profile);
    Ok(())
}
