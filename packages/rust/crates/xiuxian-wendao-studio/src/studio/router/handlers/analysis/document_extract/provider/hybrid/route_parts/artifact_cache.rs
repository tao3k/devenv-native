use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobCacheBackend, ArtifactBlobCacheBackendConfig, ArtifactBlobWrite,
    ArtifactKey, ArtifactKeyParts, ArtifactKind, pack_artifact_directory,
    unpack_artifact_directory,
};
use xiuxian_wendao_server::transport::DocumentExtractFlightRouteResponse;

use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, mirror_document_extract_cache, read_arrow_file,
    rewrite_document_extract_resource_paths,
};

const FULL_ARTIFACT_CACHE_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_PDF_FULL_ARTIFACT_CACHE";
const FULL_ARTIFACT_CACHE_ENABLED: &str = "enabled";
const FULL_ARTIFACT_CACHE_SCHEMA: &str = "xiuxian_wendao.hybrid_page_ocr_artifact_cache.v1";
const FULL_ARTIFACT_CACHE_NAMESPACE: &str = "wendao-pdf-full-artifact";
const FULL_ARTIFACT_CACHE_KIND: &str = "document-extract-bundle";
const FULL_ARTIFACT_CACHE_SHARD: &str = "full-document";

const CACHE_SIGNATURE_ENV_KEYS: &[&str] = &[
    "WENDAO_DOCUMENT_EXTRACT_PDF_OCR_PROFILE_PLANNER",
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_SELECTION",
    "WENDAO_DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_JSON",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PLANNER",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_PIPELINE",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_RENDER_DPI",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_CHUNK",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_RENDER_AHEAD",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_SCAFFOLD_MODE",
    "WENDAO_DOCUMENT_EXTRACT_PDF_REGION_CONTEXT_RATIO",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_TARGET_PIXELS",
    "WENDAO_DOCUMENT_EXTRACT_PDF_HOSTED_VLM_REGION_MAX_SLICES",
    "WENDAO_DOCUMENT_EXTRACT_PDF_FAILED_PAGE_RECOVERY",
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_PROFILE",
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_SIZE",
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_PLAN",
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_CHUNK_CONCURRENCY",
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_STRUCTURE_COST_BUDGET",
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_PAGE_RANGE_HEDGE_DELAY_MS",
    "WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_TOPUP",
    "WENDAO_DOCUMENT_EXTRACT_PDF_BACKEND_TEXT_EMPTY_PAGE",
    "WENDAO_DOCUMENT_EXTRACT_PDF_LOCAL_BACKEND_TEXT",
];

pub(super) fn hybrid_page_ocr_artifact_cache_response(
    source: &Path,
    output: &Path,
) -> Result<Option<DocumentExtractFlightRouteResponse>, String> {
    hybrid_page_ocr_artifact_cache_response_with_lookup(source, output, &env_lookup)
}

pub(super) fn store_hybrid_page_ocr_artifact_cache(
    source: &Path,
    output: &Path,
) -> Result<bool, String> {
    store_hybrid_page_ocr_artifact_cache_with_lookup(source, output, &env_lookup)
}

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(super) fn hybrid_page_ocr_artifact_cache_key_for_test(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    hybrid_page_ocr_artifact_cache_key(source, lookup)
}

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(super) fn store_hybrid_page_ocr_artifact_cache_for_test(
    source: &Path,
    output: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<bool, String> {
    store_hybrid_page_ocr_artifact_cache_with_lookup(source, output, lookup)
}

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(super) fn hybrid_page_ocr_artifact_cache_response_for_test(
    source: &Path,
    output: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<DocumentExtractFlightRouteResponse>, String> {
    hybrid_page_ocr_artifact_cache_response_with_lookup(source, output, lookup)
}

fn hybrid_page_ocr_artifact_cache_response_with_lookup(
    source: &Path,
    output: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<DocumentExtractFlightRouteResponse>, String> {
    if !hybrid_page_ocr_artifact_cache_enabled(lookup) {
        return Ok(None);
    }
    let backend = hybrid_page_ocr_artifact_cache_backend(lookup)?;
    let artifact_key = hybrid_page_ocr_artifact_key(source, lookup)?;
    let Some(read) = backend
        .read(&artifact_key)
        .map_err(|error| format!("read PDF full artifact bundle cache: {error}"))?
    else {
        return Ok(None);
    };
    unpack_artifact_directory(read.bytes(), output)
        .map_err(|error| format!("unpack PDF full artifact bundle: {error}"))?;
    rewrite_document_extract_resource_paths(
        output,
        hybrid_page_ocr_artifact_virtual_root(&artifact_key).as_path(),
        output,
    )?;
    if !output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).exists() {
        return Ok(None);
    }
    let batches = read_arrow_file(output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).as_path())?;
    Ok(Some(DocumentExtractFlightRouteResponse::from_batches(
        batches,
    )))
}

fn store_hybrid_page_ocr_artifact_cache_with_lookup(
    source: &Path,
    output: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<bool, String> {
    if !hybrid_page_ocr_artifact_cache_enabled(lookup) {
        return Ok(false);
    }
    if !output.join("_complete.marker").exists()
        || !output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).exists()
    {
        return Ok(false);
    }
    let backend = hybrid_page_ocr_artifact_cache_backend(lookup)?;
    let artifact_key = hybrid_page_ocr_artifact_key(source, lookup)?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let temp_dir = tempfile::Builder::new()
        .prefix(".wendao-pdf-full-artifact.")
        .tempdir_in(parent)
        .map_err(|error| format!("create PDF full artifact bundle temp dir: {error}"))?;
    mirror_document_extract_cache(output, temp_dir.path())?;
    rewrite_document_extract_resource_paths(
        temp_dir.path(),
        temp_dir.path(),
        hybrid_page_ocr_artifact_virtual_root(&artifact_key).as_path(),
    )?;
    let bytes = pack_artifact_directory(temp_dir.path())
        .map_err(|error| format!("pack PDF full artifact bundle: {error}"))?;
    backend
        .write(&artifact_key, ArtifactBlobWrite::new(bytes.as_slice()))
        .map_err(|error| format!("write PDF full artifact bundle cache: {error}"))?;
    Ok(true)
}

#[cfg(all(test, feature = "document-extract-pdf-render"))]
fn hybrid_page_ocr_artifact_cache_key(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let key = hybrid_page_ocr_artifact_key(source, lookup)?;
    Ok(format!(
        "{}/{}/{}/{}/{}",
        key.namespace().as_str(),
        key.kind().as_storage_component(),
        key.source_digest().as_str(),
        key.profile_digest().as_str(),
        key.shard_digest().as_str()
    ))
}

fn hybrid_page_ocr_artifact_key(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ArtifactKey, String> {
    let source_hash = sha256_file(source)?;
    let profile_digest = hybrid_page_ocr_artifact_profile_digest(source, lookup)?;
    ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: FULL_ARTIFACT_CACHE_NAMESPACE.to_owned(),
        kind: ArtifactKind::custom(FULL_ARTIFACT_CACHE_KIND)
            .map_err(|error| format!("build PDF full artifact kind: {error}"))?,
        source_digest: source_hash,
        profile_digest,
        shard_digest: FULL_ARTIFACT_CACHE_SHARD.to_owned(),
    })
    .map_err(|error| format!("build PDF full artifact key: {error}"))
}

fn hybrid_page_ocr_artifact_profile_digest(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let metadata = source
        .metadata()
        .map_err(|error| format!("read OCR artifact cache source metadata: {error}"))?;
    let mut hasher = Sha256::new();
    hasher.update(FULL_ARTIFACT_CACHE_SCHEMA.as_bytes());
    hasher.update(b"\nsource_len=");
    hasher.update(metadata.len().to_string().as_bytes());
    hasher.update(b"\nsource_ext=");
    hasher.update(
        source
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .as_bytes(),
    );
    for key in CACHE_SIGNATURE_ENV_KEYS {
        hasher.update(b"\n");
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        if let Some(value) = lookup(key) {
            hasher.update(value.as_bytes());
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hybrid_page_ocr_artifact_cache_enabled(lookup: &dyn Fn(&str) -> Option<String>) -> bool {
    lookup(FULL_ARTIFACT_CACHE_ENV).is_some_and(|value| {
        value
            .trim()
            .eq_ignore_ascii_case(FULL_ARTIFACT_CACHE_ENABLED)
    })
}

fn hybrid_page_ocr_artifact_cache_backend(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<ArtifactBlobCacheBackend, String> {
    let config = ArtifactBlobCacheBackendConfig::from_lookup(lookup)
        .map_err(|error| format!("resolve PDF full artifact cache backend: {error}"))?;
    config
        .build()
        .map_err(|error| format!("build PDF full artifact cache backend: {error}"))
}

fn hybrid_page_ocr_artifact_virtual_root(key: &ArtifactKey) -> PathBuf {
    PathBuf::from(format!(
        "/__wendao_artifact_bundle/{}/{}/{}/{}",
        key.namespace().as_str(),
        key.source_digest().as_str(),
        key.profile_digest().as_str(),
        key.shard_digest().as_str()
    ))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| {
        format!(
            "open OCR artifact cache source `{}`: {error}",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024].into_boxed_slice();
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "read OCR artifact cache source `{}`: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn env_lookup(key: &str) -> Option<String> {
    std::env::var(key).ok()
}
