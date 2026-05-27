//! Gateway-owned model routing admission for document extraction.

use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};
use xiuxian_llm::model_routing::{
    WendaoAttachmentRouteConfig, WendaoAttachmentRouteInput, WendaoModelDecision,
    WendaoModelRoutingMode, WendaoModelRoutingTomlConfig, WendaoRouteIntent,
    wendao_attachment_model_route_decision,
    wendao_image_extract_route_config_with_model_routing_config,
};
use xiuxian_wendao_server::transport::DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE;

use super::route::is_image_source_path;

const DOCUMENT_EXTRACT_ROUTE_MANIFEST_NAME: &str = "_document_extract_model_route_manifest.json";
const DOCUMENT_EXTRACT_ROUTE_MANIFEST_SCHEMA: &str =
    "xiuxian_wendao.document_extract_model_route_manifest.v1";
const IMAGE_ROUTE_TASK_KIND: &str = "attachment-extract";
const IMAGE_ROUTE_MODALITY: &str = "image";
const IMAGE_ROUTE_SOURCE_KIND: &str = "attachment";
const IMAGE_ROUTE_PRECISION_TIER: &str = "high";
const IMAGE_ROUTE_PRIVACY_TIER: &str = "private";
const IMAGE_ROUTE_EVIDENCE_PROFILE: &str = "image-document-markdown";
const IMAGE_ROUTE_LATENCY_BUDGET_MS: u64 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DocumentExtractModelRoute {
    pub(super) intent: WendaoRouteIntent,
    pub(super) decision: WendaoModelDecision,
    pub(super) source_sha256: String,
    pub(super) model_routing_mode: WendaoModelRoutingMode,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DocumentExtractRouteSourceIdentity<'a> {
    pub(super) path: &'a Path,
    pub(super) sha256: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ImageDocumentExtractRouteConfig {
    pub(super) route_provider: Option<String>,
    pub(super) route_model: String,
    pub(super) model_routing_mode: WendaoModelRoutingMode,
    pub(super) vllm_sr_base_url: String,
}

impl ImageDocumentExtractRouteConfig {
    pub(super) fn from_model_routing_config(
        model_routing: Option<&WendaoModelRoutingTomlConfig>,
    ) -> Result<Self, String> {
        image_document_extract_route_config_with_model_routing(model_routing, &|key| {
            std::env::var(key).ok()
        })
    }
}

pub(super) fn image_document_extract_route_config_with_model_routing(
    model_routing: Option<&WendaoModelRoutingTomlConfig>,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ImageDocumentExtractRouteConfig, String> {
    let shared =
        wendao_image_extract_route_config_with_model_routing_config(model_routing, lookup)?;
    Ok(ImageDocumentExtractRouteConfig {
        route_provider: shared.route_provider,
        route_model: shared.route_model,
        model_routing_mode: shared.model_routing_mode,
        vllm_sr_base_url: shared.vllm_sr_base_url,
    })
}

pub(super) async fn image_document_extract_model_route_with_config(
    source: &Path,
    profile: &str,
    config: ImageDocumentExtractRouteConfig,
) -> Result<Option<DocumentExtractModelRoute>, String> {
    if !is_image_route_candidate(source, profile) {
        return Ok(None);
    }
    let source_sha256 = source_sha256_hex(source)
        .map_err(|error| format!("image route source hash resolution failed: {error}"))?;
    image_document_extract_model_route_for_source_identity(
        DocumentExtractRouteSourceIdentity {
            path: source,
            sha256: source_sha256.as_str(),
        },
        profile,
        config,
    )
    .await
}

pub(super) async fn image_document_extract_model_route_for_source_identity(
    source: DocumentExtractRouteSourceIdentity<'_>,
    profile: &str,
    config: ImageDocumentExtractRouteConfig,
) -> Result<Option<DocumentExtractModelRoute>, String> {
    if !is_image_route_candidate(source.path, profile) {
        return Ok(None);
    }
    let source_sha256 = normalized_source_sha256(source.sha256)?;
    let route_config = WendaoAttachmentRouteConfig {
        route_provider: config.route_provider.clone(),
        route_model: config.route_model.clone(),
        backend_profile: DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE.to_owned(),
        model_routing_mode: config.model_routing_mode,
        vllm_sr_base_url: config.vllm_sr_base_url.clone(),
    };
    let route_input = image_route_input(profile, source.path, source_sha256.as_str());
    let (intent, decision) = wendao_attachment_model_route_decision(&route_config, &route_input)
        .await
        .map_err(|error| format!("image model route admission failed: {error}"))?;
    Ok(Some(DocumentExtractModelRoute {
        intent,
        decision,
        source_sha256,
        model_routing_mode: config.model_routing_mode,
    }))
}

pub(super) fn document_extract_route_manifest_matches(
    output_dir: &Path,
    model_route: Option<&DocumentExtractModelRoute>,
    profile: &str,
) -> bool {
    let Some(model_route) = model_route else {
        return true;
    };
    let manifest_path = output_dir.join(DOCUMENT_EXTRACT_ROUTE_MANIFEST_NAME);
    let Ok(payload) = std::fs::read(manifest_path.as_path()) else {
        return false;
    };
    let Ok(actual) = serde_json::from_slice::<serde_json::Value>(payload.as_slice()) else {
        return false;
    };
    actual == document_extract_route_manifest(model_route, profile)
}

pub(super) fn write_document_extract_route_manifest(
    output_dir: &Path,
    model_route: Option<&DocumentExtractModelRoute>,
    profile: &str,
) -> Result<(), String> {
    let Some(model_route) = model_route else {
        return Ok(());
    };
    let manifest_path = output_dir.join(DOCUMENT_EXTRACT_ROUTE_MANIFEST_NAME);
    let payload = serde_json::to_vec_pretty(&document_extract_route_manifest(model_route, profile))
        .map_err(|error| format!("serialize document extract route manifest: {error}"))?;
    std::fs::write(manifest_path.as_path(), payload).map_err(|error| {
        format!(
            "write document extract route manifest `{}`: {error}",
            manifest_path.display()
        )
    })
}

fn is_image_route_candidate(source: &Path, profile: &str) -> bool {
    source.exists()
        && profile == DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE
        && is_image_source_path(source)
}

fn image_route_input(
    profile: &str,
    source: &Path,
    source_sha256: &str,
) -> WendaoAttachmentRouteInput {
    let source_suffix = source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    WendaoAttachmentRouteInput {
        task_kind: IMAGE_ROUTE_TASK_KIND.to_owned(),
        modality: IMAGE_ROUTE_MODALITY.to_owned(),
        source_kind: IMAGE_ROUTE_SOURCE_KIND.to_owned(),
        precision_tier: IMAGE_ROUTE_PRECISION_TIER.to_owned(),
        privacy_tier: IMAGE_ROUTE_PRIVACY_TIER.to_owned(),
        latency_budget_ms: IMAGE_ROUTE_LATENCY_BUDGET_MS,
        evidence_profile: IMAGE_ROUTE_EVIDENCE_PROFILE.to_owned(),
        artifact_refs: vec![
            format!("source-sha256:{source_sha256}"),
            format!("source-suffix:{source_suffix}"),
            format!("backend-profile:{profile}"),
        ],
    }
}

fn document_extract_route_manifest(
    model_route: &DocumentExtractModelRoute,
    profile: &str,
) -> serde_json::Value {
    json!({
        "schema": DOCUMENT_EXTRACT_ROUTE_MANIFEST_SCHEMA,
        "sourceSha256": model_route.source_sha256.as_str(),
        "profile": profile,
        "modelRoutingMode": image_model_routing_mode_name(model_route.model_routing_mode),
        "routeSelectedProvider": model_route.decision.selected_provider.as_str(),
        "routeSelectedModel": model_route.decision.selected_model.as_str(),
        "routeSelectedBackendProfile": model_route.decision.selected_backend_profile.as_str(),
    })
}

fn source_sha256_hex(source: &Path) -> Result<String, String> {
    let bytes = std::fs::read(source)
        .map_err(|error| format!("read image route source `{}`: {error}", source.display()))?;
    Ok(hex_lower(&Sha256::digest(bytes.as_slice())))
}

fn normalized_source_sha256(source_sha256: &str) -> Result<String, String> {
    let normalized = source_sha256.trim();
    if normalized.is_empty() {
        Err("image route source sha256 must be non-empty".to_owned())
    } else {
        Ok(normalized.to_owned())
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn image_model_routing_mode_name(mode: WendaoModelRoutingMode) -> &'static str {
    match mode {
        WendaoModelRoutingMode::VllmSr => "vllm-sr",
        WendaoModelRoutingMode::Deterministic => "deterministic",
    }
}
