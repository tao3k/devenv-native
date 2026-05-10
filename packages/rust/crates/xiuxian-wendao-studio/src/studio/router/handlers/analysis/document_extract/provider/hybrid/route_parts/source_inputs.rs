use super::{
    BTreeMap, BTreeSet, DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE,
    DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD, PDF_OCR_BACKEND_TEXT_PROFILE,
    PDF_OCR_DEFAULT_PROFILE, PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, Path,
    PdfOcrShardInput, PdfSourcePageProfile, pdf_source_page_is_backend_text_topup_profile,
    pdf_source_page_is_fast_profile_risk, pdf_source_page_requires_structure_authority,
    source_pdf_page_profiles_cached,
};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;

pub(super) const DOCUMENT_EXTRACT_PDF_DOCLING_TEXT_SHORTCUT_PROMOTION_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_PDF_DOCLING_TEXT_SHORTCUT_PROMOTION";

const DOCLING_TEXT_SHORTCUT_PROMOTION_DISABLED: &str = "disabled";
const SOURCE_PDF_PAGE_RANGE_RENDER_PROFILE: &str = "source-pdf-page-range-shards-v1";
const SOURCE_PDF_PAGE_RANGE_MIME_TYPE: &str = "application/x-wendao-source-pdf-page";
const PDF_OCR_DEFAULT_ENGINE: &str = "docling-compatible-ocr";
const PDF_OCR_FAST_TEXT_ENGINE: &str = "docling-fast-text-ocr";
const PDF_OCR_BACKEND_TEXT_ENGINE: &str = "docling-backend-text-ocr";
const SYNTHETIC_SOURCE_PAGE_WIDTH_POINTS: f64 = 612.0;
const SYNTHETIC_SOURCE_PAGE_HEIGHT_POINTS: f64 = 792.0;
const SYNTHETIC_SOURCE_PAGE_DPI: u32 = 300;

pub(super) fn direct_docling_structure_recovery_source_inputs(
    source: &Path,
    page_count: u32,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let profiles = source_pdf_page_profiles_cached(source).unwrap_or_default();
    direct_docling_structure_recovery_source_inputs_for_profiles(
        source,
        page_count,
        profiles.as_slice(),
    )
}

#[cfg(test)]
pub(super) fn direct_docling_structure_recovery_source_inputs_for_profiles(
    source: &Path,
    page_count: u32,
    profiles: &[PdfSourcePageProfile],
) -> Result<Vec<PdfOcrShardInput>, String> {
    direct_docling_structure_recovery_source_inputs_for_profiles_with_lookup(
        source,
        page_count,
        profiles,
        &|key| std::env::var(key).ok(),
    )
}

#[cfg(not(test))]
fn direct_docling_structure_recovery_source_inputs_for_profiles(
    source: &Path,
    page_count: u32,
    profiles: &[PdfSourcePageProfile],
) -> Result<Vec<PdfOcrShardInput>, String> {
    direct_docling_structure_recovery_source_inputs_for_profiles_impl(
        source,
        page_count,
        profiles,
        &|key| std::env::var(key).ok(),
    )
}

#[cfg(test)]
pub(super) fn direct_docling_structure_recovery_source_inputs_for_profiles_with_lookup(
    source: &Path,
    page_count: u32,
    profiles: &[PdfSourcePageProfile],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    direct_docling_structure_recovery_source_inputs_for_profiles_impl(
        source, page_count, profiles, lookup,
    )
}

fn direct_docling_structure_recovery_source_inputs_for_profiles_impl(
    source: &Path,
    page_count: u32,
    profiles: &[PdfSourcePageProfile],
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let source_content_hash = sha256_file_hex(source)?;
    let profiles_by_page = profiles
        .iter()
        .map(|profile| (profile.page_index, profile))
        .collect::<BTreeMap<_, _>>();
    let source_path = source.to_string_lossy().to_string();
    let raster_width_px = points_to_pixels(SYNTHETIC_SOURCE_PAGE_WIDTH_POINTS);
    let raster_height_px = points_to_pixels(SYNTHETIC_SOURCE_PAGE_HEIGHT_POINTS);
    let scale_x = f64::from(raster_width_px) / SYNTHETIC_SOURCE_PAGE_WIDTH_POINTS;
    let scale_y = f64::from(raster_height_px) / SYNTHETIC_SOURCE_PAGE_HEIGHT_POINTS;

    let mut inputs = (0..page_count)
        .map(|page_index| {
            let (ocr_profile, ocr_engine, preserve_layout) =
                direct_docling_structure_recovery_page_profile(
                    profiles_by_page.get(&page_index).copied(),
                );
            let shard_element_id = sha256_hex(
                format!(
                    "{source_content_hash}:{page_index}:{SOURCE_PDF_PAGE_RANGE_RENDER_PROFILE}"
                )
                .as_bytes(),
            );
            let raster_sha256 = sha256_hex(
                format!("source-page-range:{source_content_hash}:{page_index}").as_bytes(),
            );
            PdfOcrShardInput {
                contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
                source_path: source_path.clone(),
                source_content_hash: source_content_hash.clone(),
                page_index,
                image_path: format!(
                    "{source_path}#source-page-range-{page_index:05}.source-page-range"
                ),
                image_mime_type: SOURCE_PDF_PAGE_RANGE_MIME_TYPE.to_string(),
                raster_sha256,
                render_profile: SOURCE_PDF_PAGE_RANGE_RENDER_PROFILE.to_string(),
                ocr_profile: ocr_profile.to_string(),
                ocr_engine: ocr_engine.to_string(),
                preferred_languages: vec!["auto".to_string()],
                min_confidence: 0.0,
                preserve_layout,
                raster_width_px,
                raster_height_px,
                render_dpi: SYNTHETIC_SOURCE_PAGE_DPI,
                rotation_degrees: 0,
                crop_left: 0.0,
                crop_bottom: 0.0,
                crop_right: SYNTHETIC_SOURCE_PAGE_WIDTH_POINTS,
                crop_top: SYNTHETIC_SOURCE_PAGE_HEIGHT_POINTS,
                point_to_pixel_scale_x: scale_x,
                point_to_pixel_scale_y: scale_y,
                shard_element_id,
                shard_type: "page".to_string(),
                region_index: 0,
                parent_shard_element_id: String::new(),
                reading_order_key: format!("{page_index:06}.000000"),
                source_page_pixel_left: 0,
                source_page_pixel_top: 0,
                source_page_pixel_right: raster_width_px,
                source_page_pixel_bottom: raster_height_px,
            }
        })
        .collect::<Vec<_>>();
    guard_direct_docling_structure_recovery_text_shortcuts(&mut inputs, lookup);
    Ok(inputs)
}

fn direct_docling_structure_recovery_page_profile(
    profile: Option<&PdfSourcePageProfile>,
) -> (&'static str, &'static str, bool) {
    let Some(profile) = profile else {
        return (PDF_OCR_DEFAULT_PROFILE, PDF_OCR_DEFAULT_ENGINE, true);
    };
    if pdf_source_page_requires_structure_authority(profile)
        || pdf_source_page_is_fast_profile_risk(profile)
    {
        return (PDF_OCR_DEFAULT_PROFILE, PDF_OCR_DEFAULT_ENGINE, true);
    }
    if profile.text_show_ops == 0 {
        return (PDF_OCR_DEFAULT_PROFILE, PDF_OCR_DEFAULT_ENGINE, true);
    }
    if pdf_source_page_is_backend_text_topup_profile(profile) {
        return (PDF_OCR_FAST_TEXT_PROFILE, PDF_OCR_FAST_TEXT_ENGINE, true);
    }
    (
        PDF_OCR_BACKEND_TEXT_PROFILE,
        PDF_OCR_BACKEND_TEXT_ENGINE,
        false,
    )
}

fn guard_direct_docling_structure_recovery_text_shortcuts(
    inputs: &mut [PdfOcrShardInput],
    lookup: &dyn Fn(&str) -> Option<String>,
) {
    if !docling_text_shortcut_promotion_enabled_with_lookup(lookup) {
        return;
    }
    let page_count = inputs
        .iter()
        .filter(|input| input.shard_type == "page")
        .count();
    if page_count == 0 || page_count <= DOCLING_STRUCTURE_RECOVERY_SMALL_PAGE_RANGE_THRESHOLD {
        return;
    }
    loop {
        let fallback_pages = direct_docling_structure_recovery_fallback_pages(inputs);
        let current_range_count = chunked_page_range_count(
            &fallback_pages,
            DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE,
        );
        let Some(index_to_promote) =
            best_text_shortcut_to_promote(inputs, &fallback_pages, current_range_count)
        else {
            break;
        };
        inputs[index_to_promote].ocr_profile = PDF_OCR_DEFAULT_PROFILE.to_string();
        inputs[index_to_promote].ocr_engine = PDF_OCR_DEFAULT_ENGINE.to_string();
        inputs[index_to_promote].preserve_layout = true;
    }
}

fn docling_text_shortcut_promotion_enabled_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    !matches!(
        lookup(DOCUMENT_EXTRACT_PDF_DOCLING_TEXT_SHORTCUT_PROMOTION_ENV)
            .unwrap_or_default()
            .trim()
            .replace('_', "-")
            .to_ascii_lowercase()
            .as_str(),
        DOCLING_TEXT_SHORTCUT_PROMOTION_DISABLED | "off" | "none"
    )
}

fn best_text_shortcut_to_promote(
    inputs: &[PdfOcrShardInput],
    fallback_pages: &BTreeSet<u32>,
    current_range_count: usize,
) -> Option<usize> {
    inputs
        .iter()
        .enumerate()
        .filter(|(_, input)| {
            input.shard_type == "page"
                && matches!(
                    input.ocr_profile.as_str(),
                    PDF_OCR_BACKEND_TEXT_PROFILE | PDF_OCR_FAST_TEXT_PROFILE
                )
        })
        .filter_map(|(index, input)| {
            let mut candidate_pages = fallback_pages.clone();
            candidate_pages.insert(input.page_index);
            let candidate_range_count = chunked_page_range_count(
                &candidate_pages,
                DOCLING_STRUCTURE_RECOVERY_DEFAULT_PAGE_RANGE_CHUNK_SIZE,
            );
            (candidate_range_count < current_range_count
                || (candidate_range_count == current_range_count
                    && text_shortcut_bridges_fallback_gap(input.page_index, fallback_pages)))
            .then_some((index, candidate_range_count))
        })
        .min_by_key(|(_, candidate_range_count)| *candidate_range_count)
        .map(|(index, _)| index)
}

fn text_shortcut_bridges_fallback_gap(page_index: u32, fallback_pages: &BTreeSet<u32>) -> bool {
    page_index
        .checked_sub(1)
        .is_some_and(|previous| fallback_pages.contains(&previous))
        && fallback_pages.contains(&page_index.saturating_add(1))
}

fn direct_docling_structure_recovery_fallback_pages(inputs: &[PdfOcrShardInput]) -> BTreeSet<u32> {
    inputs
        .iter()
        .filter(|input| input.shard_type == "page" && input.ocr_profile == PDF_OCR_DEFAULT_PROFILE)
        .map(|input| input.page_index)
        .collect()
}

fn chunked_page_range_count(pages: &BTreeSet<u32>, max_chunk_pages: u32) -> usize {
    if pages.is_empty() {
        return 0;
    }
    let max_chunk_pages = usize::try_from(max_chunk_pages).unwrap_or(1).max(1);
    let mut range_count = 0usize;
    let mut current_start: Option<u32> = None;
    let mut previous: Option<u32> = None;
    for page in pages {
        match (current_start, previous) {
            (Some(_), Some(previous)) if *page == previous.saturating_add(1) => {}
            (Some(start), Some(end)) => {
                range_count =
                    range_count.saturating_add(chunk_count_for_range(start, end, max_chunk_pages));
                current_start = Some(*page);
            }
            _ => current_start = Some(*page),
        }
        previous = Some(*page);
    }
    if let (Some(start), Some(end)) = (current_start, previous) {
        range_count =
            range_count.saturating_add(chunk_count_for_range(start, end, max_chunk_pages));
    }
    range_count
}

fn chunk_count_for_range(start: u32, end: u32, max_chunk_pages: usize) -> usize {
    let page_count =
        usize::try_from(u64::from(end.saturating_sub(start)) + 1).unwrap_or(usize::MAX);
    page_count.div_ceil(max_chunk_pages)
}

fn points_to_pixels(points: f64) -> u32 {
    ((points / 72.0) * f64::from(SYNTHETIC_SOURCE_PAGE_DPI))
        .round()
        .max(1.0) as u32
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("open source PDF `{}`: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("hash source PDF `{}`: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
