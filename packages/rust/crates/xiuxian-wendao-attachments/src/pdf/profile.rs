//! Lightweight PDF page complexity profile for source-range OCR planning.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use lopdf::{Document as LopdfDocument, ObjectId, content::Operation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SOURCE_PROFILE_CACHE_SCHEMA: &str = "xiuxian_wendao.pdf_source_page_profiles.v1";
const SOURCE_PROFILE_CACHE_DIR_NAME: &str = "pdf-source-page-profiles";

/// Lightweight facts derived from one source PDF page content stream.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PdfSourcePageProfile {
    /// Zero-based page index.
    pub page_index: u32,
    /// Decoded content stream byte count.
    pub content_bytes: u32,
    /// PDF content operation count.
    pub operation_count: u32,
    /// Text-showing operation count.
    pub text_show_ops: u32,
    /// Path drawing operation count.
    pub path_ops: u32,
    /// Rectangle path operation count.
    pub rectangle_ops: u32,
    /// External object draw operation count.
    pub draw_object_ops: u32,
    /// Conservative planner weight derived from the operation counts.
    pub estimated_weight: u32,
}

/// Conservative planner facts derived from one source PDF page profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfSourcePageClassification {
    /// Zero-based page index.
    pub page_index: u32,
    /// Conservative Docling-structure scheduling cost.
    pub estimated_structure_cost: u32,
    /// Whether Docling must remain the structure authority for this page.
    pub structure_authority_required: bool,
    /// Whether the page is eligible for a high-precision OCR/VLM patch.
    pub ocr_patch_candidate: bool,
    /// Whether backend-text may be used as a text shortcut for this page.
    pub text_shortcut_eligible: bool,
}

/// Return conservative planner facts for one source PDF page.
#[must_use]
pub fn classify_pdf_source_page(profile: &PdfSourcePageProfile) -> PdfSourcePageClassification {
    let structure_authority_required = pdf_source_page_requires_structure_authority(profile);
    PdfSourcePageClassification {
        page_index: profile.page_index,
        estimated_structure_cost: pdf_source_page_structure_cost(profile),
        structure_authority_required,
        ocr_patch_candidate: pdf_source_page_is_fast_profile_risk(profile),
        text_shortcut_eligible: !structure_authority_required && profile.text_show_ops > 0,
    }
}

/// Return conservative planner facts for all source PDF pages.
#[must_use]
pub fn classify_pdf_source_pages(
    profiles: &[PdfSourcePageProfile],
) -> Vec<PdfSourcePageClassification> {
    profiles.iter().map(classify_pdf_source_page).collect()
}

/// Return whether a page must keep Docling as its structure authority.
#[must_use]
pub fn pdf_source_page_requires_structure_authority(profile: &PdfSourcePageProfile) -> bool {
    profile.draw_object_ops > 0
        || profile.rectangle_ops > 0
        || profile.path_ops >= 64
        || pdf_source_page_is_text_table_candidate(profile)
}

/// Return the conservative Docling-structure scheduling cost for one source page.
#[must_use]
pub fn pdf_source_page_structure_cost(profile: &PdfSourcePageProfile) -> u32 {
    let authority_bonus = if pdf_source_page_requires_structure_authority(profile) {
        256
    } else {
        0
    };
    let patch_bonus = if pdf_source_page_is_fast_profile_risk(profile) {
        128
    } else {
        0
    };

    1_u32
        .saturating_add(profile.estimated_weight.max(1))
        .saturating_add(profile.operation_count.div_ceil(16))
        .saturating_add(profile.content_bytes.div_ceil(2048))
        .saturating_add(profile.path_ops.saturating_mul(3))
        .saturating_add(profile.rectangle_ops.saturating_mul(12))
        .saturating_add(profile.draw_object_ops.saturating_mul(96))
        .saturating_add(authority_bonus)
        .saturating_add(patch_bonus)
}

/// Return whether a page matches the existing fast-profile structural risk.
#[must_use]
pub fn pdf_source_page_is_fast_profile_risk(profile: &PdfSourcePageProfile) -> bool {
    let compact_table_grid = (1..=8).contains(&profile.rectangle_ops)
        && profile.operation_count >= 640
        && profile.text_show_ops >= 120;
    let dense_table_path_band = (64..=120).contains(&profile.path_ops)
        && profile.operation_count >= 640
        && profile.text_show_ops >= 150;
    compact_table_grid || dense_table_path_band
}

/// Return whether a page has a dense text grid signal that may hide a table.
#[must_use]
pub fn pdf_source_page_is_text_table_candidate(profile: &PdfSourcePageProfile) -> bool {
    profile.draw_object_ops == 0
        && profile.rectangle_ops == 0
        && profile.path_ops <= 8
        && profile.text_show_ops >= 96
        && profile.operation_count >= 280
        && profile.content_bytes >= 10_000
}

/// Return whether a page matches the existing dense backend-text top-up signal.
#[must_use]
pub fn pdf_source_page_is_backend_text_topup_profile(profile: &PdfSourcePageProfile) -> bool {
    let dense_text_page = profile.text_show_ops >= 320 && profile.operation_count >= 640;
    let large_content_stream = profile.content_bytes >= 65_536 && profile.text_show_ops >= 180;
    dense_text_page || large_content_stream
}

/// Return page-level source PDF complexity profiles.
///
/// # Errors
///
/// Returns an error if the PDF cannot be loaded or page content streams cannot
/// be decoded.
pub fn source_pdf_page_profiles(path: &Path) -> Result<Vec<PdfSourcePageProfile>, String> {
    let document =
        LopdfDocument::load(path).map_err(|error| format!("load PDF with lopdf: {error}"))?;
    document
        .get_pages()
        .into_values()
        .enumerate()
        .map(|(page_index, page_id)| {
            source_pdf_page_profile(
                &document,
                u32::try_from(page_index).unwrap_or(u32::MAX),
                page_id,
            )
        })
        .collect()
}

/// Return cached page-level source PDF complexity profiles for one file state.
///
/// # Errors
///
/// Returns an error if the PDF cannot be loaded or page content streams cannot
/// be decoded.
pub fn source_pdf_page_profiles_cached(path: &Path) -> Result<Vec<PdfSourcePageProfile>, String> {
    let Some(key) = source_pdf_page_profile_cache_key(path) else {
        return source_pdf_page_profiles(path);
    };
    let cache = source_pdf_page_profile_cache();
    if let Some(profiles) = lock_profile_cache(cache).get(&key).cloned() {
        return Ok(profiles);
    }
    if let Some(profiles) = read_source_pdf_page_profile_disk_cache(&key) {
        lock_profile_cache(cache).insert(key, profiles.clone());
        return Ok(profiles);
    }

    let profiles = source_pdf_page_profiles(path)?;
    lock_profile_cache(cache).insert(key.clone(), profiles.clone());
    write_source_pdf_page_profile_disk_cache(&key, profiles.as_slice());
    Ok(profiles)
}

fn source_pdf_page_profile(
    document: &LopdfDocument,
    page_index: u32,
    page_id: ObjectId,
) -> Result<PdfSourcePageProfile, String> {
    let content_bytes = document
        .get_page_content(page_id)
        .map_err(|error| format!("read PDF page {page_index} content: {error}"))?;
    let content = lopdf::content::Content::decode(content_bytes.as_slice())
        .map_err(|error| format!("decode PDF page {page_index} content: {error}"))?;
    let mut profile = PdfSourcePageProfile {
        page_index,
        content_bytes: u32::try_from(content_bytes.len()).unwrap_or(u32::MAX),
        operation_count: u32::try_from(content.operations.len()).unwrap_or(u32::MAX),
        text_show_ops: 0,
        path_ops: 0,
        rectangle_ops: 0,
        draw_object_ops: 0,
        estimated_weight: 1,
    };
    for operation in &content.operations {
        update_profile_counts(&mut profile, operation);
    }
    profile.estimated_weight = estimated_weight(&profile);
    Ok(profile)
}

fn update_profile_counts(profile: &mut PdfSourcePageProfile, operation: &Operation) {
    match operation.operator.as_str() {
        "Tj" | "TJ" | "'" | "\"" => {
            profile.text_show_ops = profile.text_show_ops.saturating_add(1);
        }
        "Do" => {
            profile.draw_object_ops = profile.draw_object_ops.saturating_add(1);
        }
        "re" => {
            profile.rectangle_ops = profile.rectangle_ops.saturating_add(1);
            profile.path_ops = profile.path_ops.saturating_add(1);
        }
        "m" | "l" | "c" | "v" | "y" | "h" | "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b"
        | "b*" | "n" | "W" | "W*" => {
            profile.path_ops = profile.path_ops.saturating_add(1);
        }
        _ => {}
    }
}

fn estimated_weight(profile: &PdfSourcePageProfile) -> u32 {
    1_u32
        .saturating_add(profile.text_show_ops)
        .saturating_add(profile.operation_count.div_ceil(24))
        .saturating_add(profile.content_bytes.div_ceil(4096))
        .saturating_add(profile.path_ops.div_ceil(4))
        .saturating_add(profile.rectangle_ops.saturating_mul(4))
        .saturating_add(profile.draw_object_ops.saturating_mul(6))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourcePdfPageProfileCacheKey {
    path: PathBuf,
    len: u64,
    modified_secs: u64,
    modified_nanos: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourcePdfPageProfileDiskCache {
    schema: String,
    path: String,
    len: u64,
    modified_secs: u64,
    modified_nanos: u32,
    profiles: Vec<PdfSourcePageProfile>,
}

impl SourcePdfPageProfileDiskCache {
    fn new(key: &SourcePdfPageProfileCacheKey, profiles: &[PdfSourcePageProfile]) -> Self {
        Self {
            schema: SOURCE_PROFILE_CACHE_SCHEMA.to_string(),
            path: key.path.to_string_lossy().to_string(),
            len: key.len,
            modified_secs: key.modified_secs,
            modified_nanos: key.modified_nanos,
            profiles: profiles.to_vec(),
        }
    }

    fn matches(&self, key: &SourcePdfPageProfileCacheKey) -> bool {
        self.schema == SOURCE_PROFILE_CACHE_SCHEMA
            && self.path == key.path.to_string_lossy()
            && self.len == key.len
            && self.modified_secs == key.modified_secs
            && self.modified_nanos == key.modified_nanos
    }
}

type SourcePdfPageProfileCache =
    Mutex<BTreeMap<SourcePdfPageProfileCacheKey, Vec<PdfSourcePageProfile>>>;

fn source_pdf_page_profile_cache_key(path: &Path) -> Option<SourcePdfPageProfileCacheKey> {
    let path = path.canonicalize().ok()?;
    let metadata = std::fs::metadata(path.as_path()).ok()?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok());
    Some(SourcePdfPageProfileCacheKey {
        path,
        len: metadata.len(),
        modified_secs: modified.map_or(0, |duration| duration.as_secs()),
        modified_nanos: modified.map_or(0, |duration| duration.subsec_nanos()),
    })
}

fn source_pdf_page_profile_cache() -> &'static SourcePdfPageProfileCache {
    static CACHE: OnceLock<SourcePdfPageProfileCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn read_source_pdf_page_profile_disk_cache(
    key: &SourcePdfPageProfileCacheKey,
) -> Option<Vec<PdfSourcePageProfile>> {
    let path = source_pdf_page_profile_disk_cache_path(key)?;
    let bytes = std::fs::read(path).ok()?;
    let cache = serde_json::from_slice::<SourcePdfPageProfileDiskCache>(bytes.as_slice()).ok()?;
    cache.matches(key).then_some(cache.profiles)
}

fn write_source_pdf_page_profile_disk_cache(
    key: &SourcePdfPageProfileCacheKey,
    profiles: &[PdfSourcePageProfile],
) {
    let Some(path) = source_pdf_page_profile_disk_cache_path(key) else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let cache = SourcePdfPageProfileDiskCache::new(key, profiles);
    let Ok(bytes) = serde_json::to_vec(&cache) else {
        return;
    };
    let tmp_path = path.with_extension(format!("{}.tmp", std::process::id()));
    if std::fs::write(tmp_path.as_path(), bytes).is_err() {
        return;
    }
    if std::fs::rename(tmp_path.as_path(), path.as_path()).is_err() {
        let _ = std::fs::remove_file(tmp_path);
    }
}

fn source_pdf_page_profile_disk_cache_path(key: &SourcePdfPageProfileCacheKey) -> Option<PathBuf> {
    Some(source_pdf_page_profile_disk_cache_root()?.join(format!(
        "{}.json",
        source_pdf_page_profile_disk_cache_key(key)
    )))
}

fn source_pdf_page_profile_disk_cache_root() -> Option<PathBuf> {
    std::env::var_os("PRJ_CACHE_HOME")
        .map(PathBuf::from)
        .map(|root| {
            root.join("wendao-document-extract")
                .join(SOURCE_PROFILE_CACHE_DIR_NAME)
        })
}

fn source_pdf_page_profile_disk_cache_key(key: &SourcePdfPageProfileCacheKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.path.to_string_lossy().as_bytes());
    hasher.update([0]);
    hasher.update(key.len.to_le_bytes());
    hasher.update(key.modified_secs.to_le_bytes());
    hasher.update(key.modified_nanos.to_le_bytes());
    format!("{:x}", hasher.finalize())
}

fn lock_profile_cache(
    cache: &SourcePdfPageProfileCache,
) -> std::sync::MutexGuard<'_, BTreeMap<SourcePdfPageProfileCacheKey, Vec<PdfSourcePageProfile>>> {
    match cache.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/profile.rs"]
mod tests;
