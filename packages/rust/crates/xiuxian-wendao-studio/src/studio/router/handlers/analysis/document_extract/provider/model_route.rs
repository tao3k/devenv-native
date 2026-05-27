//! Gateway-owned model routing admission for document extraction.

use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};
use xiuxian_llm::model_routing::{
    VllmSrRouteDecisionClient, WendaoModelDecision, WendaoModelRoutingMode, WendaoRouteIntent,
    WendaoRouteSourceKind, WendaoRouteTaskKind, wendao_model_routing_mode_with_lookup,
    wendao_vllm_sr_base_url_with_lookup,
};
use xiuxian_wendao_server::transport::DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE;

use super::route::is_image_source_path;

pub(super) const DOCUMENT_EXTRACT_IMAGE_ROUTE_PROVIDER_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_IMAGE_ROUTE_PROVIDER";

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ImageDocumentExtractRouteConfig {
    pub(super) route_provider: Option<String>,
    pub(super) model_routing_mode: WendaoModelRoutingMode,
    pub(super) vllm_sr_base_url: String,
}

impl ImageDocumentExtractRouteConfig {
    pub(super) fn from_env() -> Result<Self, String> {
        image_document_extract_route_config(&|key| std::env::var(key).ok())
    }
}

pub(super) fn image_document_extract_route_config(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ImageDocumentExtractRouteConfig, String> {
    Ok(ImageDocumentExtractRouteConfig {
        route_provider: optional_string_value(lookup, DOCUMENT_EXTRACT_IMAGE_ROUTE_PROVIDER_ENV),
        model_routing_mode: wendao_model_routing_mode_with_lookup(lookup)?,
        vllm_sr_base_url: wendao_vllm_sr_base_url_with_lookup(lookup),
    })
}

pub(super) async fn image_document_extract_model_route(
    source: &Path,
    profile: &str,
) -> Result<Option<DocumentExtractModelRoute>, String> {
    image_document_extract_model_route_with_config(
        source,
        profile,
        ImageDocumentExtractRouteConfig::from_env()?,
    )
    .await
}

pub(super) async fn image_document_extract_model_route_with_config(
    source: &Path,
    profile: &str,
    config: ImageDocumentExtractRouteConfig,
) -> Result<Option<DocumentExtractModelRoute>, String> {
    if !is_image_route_candidate(source, profile) {
        return Ok(None);
    }
    match config.model_routing_mode {
        WendaoModelRoutingMode::Deterministic => Ok(None),
        WendaoModelRoutingMode::VllmSr => {
            let provider = image_route_provider_hint(&config)?;
            let source_sha256 = source_sha256_hex(source)?;
            let intent = image_route_intent(profile, source, source_sha256.as_str());
            let decision = VllmSrRouteDecisionClient::new(config.vllm_sr_base_url.as_str())
                .decide(
                    &intent,
                    provider.as_str(),
                    DOCUMENT_EXTRACT_HOSTED_VLM_IMAGE_PROFILE,
                )
                .await
                .map_err(|error| format!("image model route admission failed: {error}"))?;
            Ok(Some(DocumentExtractModelRoute {
                intent,
                decision,
                source_sha256,
            }))
        }
    }
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

fn image_route_provider_hint(config: &ImageDocumentExtractRouteConfig) -> Result<String, String> {
    config
        .route_provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "WENDAO_MODEL_ROUTING_MODE=vllm-sr requires {DOCUMENT_EXTRACT_IMAGE_ROUTE_PROVIDER_ENV}"
            )
        })
}

fn image_route_intent(profile: &str, source: &Path, source_sha256: &str) -> WendaoRouteIntent {
    let source_suffix = source
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .unwrap_or_default();
    WendaoRouteIntent {
        task_kind: WendaoRouteTaskKind::new(IMAGE_ROUTE_TASK_KIND),
        modality: IMAGE_ROUTE_MODALITY.to_owned(),
        source_kind: WendaoRouteSourceKind::new(IMAGE_ROUTE_SOURCE_KIND),
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
        "modelRoutingMode": "vllm-sr",
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

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn optional_string_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
) -> Option<String> {
    lookup(key)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
