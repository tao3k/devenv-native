use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use xiuxian_wendao_server::transport::DocumentExtractFlightRouteResponse;

use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, mirror_document_extract_cache, read_arrow_file,
    rewrite_document_extract_resource_paths,
};

const FULL_ARTIFACT_CACHE_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_PDF_FULL_ARTIFACT_CACHE";
const FULL_ARTIFACT_CACHE_ROOT_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_PDF_FULL_ARTIFACT_CACHE_ROOT";
const FULL_ARTIFACT_CACHE_ENABLED: &str = "enabled";
const FULL_ARTIFACT_CACHE_SCHEMA: &str = "xiuxian_wendao.hybrid_page_ocr_artifact_cache.v1";

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
    let artifact_dir = hybrid_page_ocr_artifact_cache_dir(source, lookup)?;
    if !artifact_dir.join("_complete.marker").exists()
        || !artifact_dir
            .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
            .exists()
    {
        return Ok(None);
    }
    mirror_document_extract_cache(artifact_dir.as_path(), output)?;
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
    let artifact_dir = hybrid_page_ocr_artifact_cache_dir(source, lookup)?;
    let parent = artifact_dir.parent().ok_or_else(|| {
        format!(
            "invalid OCR artifact cache path `{}`",
            artifact_dir.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "create OCR artifact cache parent `{}`: {error}",
            parent.display()
        )
    })?;
    let temp_dir = parent.join(format!(
        ".{}.tmp",
        artifact_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("artifact")
    ));
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir.as_path()).map_err(|error| {
            format!(
                "remove stale OCR artifact cache temp `{}`: {error}",
                temp_dir.display()
            )
        })?;
    }
    mirror_document_extract_cache(output, temp_dir.as_path())?;
    if artifact_dir.exists() {
        fs::remove_dir_all(artifact_dir.as_path()).map_err(|error| {
            format!(
                "remove stale OCR artifact cache `{}`: {error}",
                artifact_dir.display()
            )
        })?;
    }
    fs::rename(temp_dir.as_path(), artifact_dir.as_path()).map_err(|error| {
        format!(
            "promote OCR artifact cache `{}` to `{}`: {error}",
            temp_dir.display(),
            artifact_dir.display()
        )
    })?;
    rewrite_document_extract_resource_paths(
        artifact_dir.as_path(),
        temp_dir.as_path(),
        artifact_dir.as_path(),
    )?;
    Ok(true)
}

fn hybrid_page_ocr_artifact_cache_dir(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<PathBuf, String> {
    let root = hybrid_page_ocr_artifact_cache_root(lookup)?;
    let key = hybrid_page_ocr_artifact_cache_key(source, lookup)?;
    Ok(root.join(&key[..2]).join(key))
}

fn hybrid_page_ocr_artifact_cache_key(
    source: &Path,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let source_hash = sha256_file(source)?;
    let metadata = source
        .metadata()
        .map_err(|error| format!("read OCR artifact cache source metadata: {error}"))?;
    let mut hasher = Sha256::new();
    hasher.update(FULL_ARTIFACT_CACHE_SCHEMA.as_bytes());
    hasher.update(b"\nsource_sha256=");
    hasher.update(source_hash.as_bytes());
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

fn hybrid_page_ocr_artifact_cache_root(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<PathBuf, String> {
    if let Some(root) =
        lookup(FULL_ARTIFACT_CACHE_ROOT_ENV).filter(|value| !value.trim().is_empty())
    {
        return Ok(PathBuf::from(root));
    }
    let cache_home = lookup("PRJ_CACHE_HOME")
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "{FULL_ARTIFACT_CACHE_ROOT_ENV} or PRJ_CACHE_HOME must be set when full OCR artifact cache is enabled"
            )
        })?;
    Ok(PathBuf::from(cache_home)
        .join("wendao-document-extract")
        .join("hybrid-page-ocr-artifacts"))
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
