use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, PdfOcrShardResultStatus, build_ocr_shard_result_batch,
    decode_ocr_shard_result_batches,
};

use super::arrow_cache::{read_arrow_file, write_arrow_file};
use super::pdf_ocr_order::{order_ocr_results_by_inputs, validate_ocr_result_matches_input};

pub(super) const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT";

#[derive(Debug, Clone)]
pub(super) struct PdfOcrShardCache {
    root: PathBuf,
}

#[derive(Debug)]
pub(super) struct PdfOcrShardCacheResolution {
    slots: Vec<Option<PdfOcrShardResult>>,
    misses: Vec<PdfOcrShardInput>,
    miss_positions: Vec<usize>,
    hit_count: usize,
}

impl PdfOcrShardCache {
    pub(super) fn from_environment() -> Self {
        if let Some(root) = std::env::var_os(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV) {
            return Self::new(PathBuf::from(root));
        }
        let cache_root = std::env::var_os("PRJ_CACHE_HOME")
            .map_or_else(|| PathBuf::from(".cache"), PathBuf::from);
        Self::new(cache_root.join("wendao-document-extract/ocr-shards"))
    }

    pub(super) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(super) fn resolve(&self, inputs: &[PdfOcrShardInput]) -> PdfOcrShardCacheResolution {
        let mut slots = vec![None; inputs.len()];
        let mut misses = Vec::new();
        let mut miss_positions = Vec::new();
        let mut hit_count = 0;

        for (position, input) in inputs.iter().enumerate() {
            if let Some(result) = self.read(input) {
                slots[position] = Some(result);
                hit_count += 1;
            } else {
                misses.push(input.clone());
                miss_positions.push(position);
            }
        }

        PdfOcrShardCacheResolution {
            slots,
            misses,
            miss_positions,
            hit_count,
        }
    }

    pub(super) fn store_successful(
        &self,
        input: &PdfOcrShardInput,
        result: &PdfOcrShardResult,
    ) -> Result<bool, String> {
        if result.status != PdfOcrShardResultStatus::Succeeded {
            return Ok(false);
        }
        validate_ocr_result_matches_input(input, result)?;
        let path = self.cache_path(input);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "create OCR shard cache directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        let batch = build_ocr_shard_result_batch(std::slice::from_ref(result))?;
        let temporary = temporary_cache_path(path.as_path());
        write_arrow_file(temporary.as_path(), std::slice::from_ref(&batch))?;
        fs::rename(temporary.as_path(), path.as_path()).map_err(|error| {
            let _ = fs::remove_file(temporary.as_path());
            format!(
                "publish OCR shard cache `{}` to `{}`: {error}",
                temporary.display(),
                path.display()
            )
        })?;
        Ok(true)
    }

    fn read(&self, input: &PdfOcrShardInput) -> Option<PdfOcrShardResult> {
        let path = self.cache_path(input);
        if !path.exists() {
            return None;
        }
        let result = read_existing_result(input, path.as_path());
        if result.is_none() {
            let _ = fs::remove_file(path.as_path());
        }
        result
    }

    fn cache_path(&self, input: &PdfOcrShardInput) -> PathBuf {
        let key = ocr_shard_cache_key(input);
        self.root.join(&key[0..2]).join(format!("{key}.arrow"))
    }
}

fn read_existing_result(input: &PdfOcrShardInput, path: &Path) -> Option<PdfOcrShardResult> {
    let batches = read_arrow_file(path).ok()?;
    let mut results = decode_ocr_shard_result_batches(batches.as_slice()).ok()?;
    if results.len() != 1 {
        return None;
    }
    let result = results.pop()?;
    validate_ocr_result_matches_input(input, &result).ok()?;
    (result.status == PdfOcrShardResultStatus::Succeeded).then_some(result)
}

impl PdfOcrShardCacheResolution {
    pub(super) fn misses(&self) -> &[PdfOcrShardInput] {
        self.misses.as_slice()
    }

    pub(super) fn hit_count(&self) -> usize {
        self.hit_count
    }

    pub(super) fn merge(
        mut self,
        live_results: Vec<PdfOcrShardResult>,
    ) -> Result<Vec<PdfOcrShardResult>, String> {
        let ordered_live = order_ocr_results_by_inputs(self.misses.as_slice(), live_results)?;
        for (position, result) in self.miss_positions.into_iter().zip(ordered_live) {
            self.slots[position] = Some(result);
        }
        self.slots
            .into_iter()
            .enumerate()
            .map(|(position, result)| {
                result.ok_or_else(|| {
                    format!("OCR shard cache merge left input position {position} unresolved")
                })
            })
            .collect()
    }
}

pub(super) fn ocr_shard_cache_key(input: &PdfOcrShardInput) -> String {
    let mut hasher = Sha256::new();
    for fragment in ocr_shard_cache_fragments(input) {
        hasher.update(fragment.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn ocr_shard_cache_fragments(input: &PdfOcrShardInput) -> Vec<String> {
    vec![
        input.contract_version.clone(),
        input.source_path.clone(),
        input.source_content_hash.clone(),
        input.page_index.to_string(),
        input.shard_type.clone(),
        input.region_index.to_string(),
        input.parent_shard_element_id.clone(),
        input.reading_order_key.clone(),
        input.image_mime_type.clone(),
        input.raster_sha256.clone(),
        input.render_profile.clone(),
        input.ocr_profile.clone(),
        input.ocr_engine.clone(),
        input.preferred_languages.join("\u{1f}"),
        f64_bits(input.min_confidence),
        input.preserve_layout.to_string(),
        input.raster_width_px.to_string(),
        input.raster_height_px.to_string(),
        input.render_dpi.to_string(),
        input.rotation_degrees.to_string(),
        f64_bits(input.crop_left),
        f64_bits(input.crop_bottom),
        f64_bits(input.crop_right),
        f64_bits(input.crop_top),
        f64_bits(input.point_to_pixel_scale_x),
        f64_bits(input.point_to_pixel_scale_y),
        input.source_page_pixel_left.to_string(),
        input.source_page_pixel_top.to_string(),
        input.source_page_pixel_right.to_string(),
        input.source_page_pixel_bottom.to_string(),
        input.shard_element_id.clone(),
    ]
}

fn f64_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn temporary_cache_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ocr-shard.arrow");
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nanos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiuxian_wendao_attachments::pdf::ocr::PDF_OCR_SHARD_INPUT_SCHEMA_VERSION;

    #[test]
    fn cache_key_changes_for_page_region_profile_and_raster() {
        let page = sample_ocr_input(0, "page");
        let mut other_page = sample_ocr_input(1, "page");
        let mut region = sample_ocr_input(0, "region");
        let mut profile = sample_ocr_input(0, "page");
        let mut raster = sample_ocr_input(0, "page");
        other_page.shard_element_id = "page-shard-1".to_string();
        region.region_index = 3;
        region.shard_element_id = "region-shard-0-3".to_string();
        profile.ocr_profile = "docling-fast-text-ocr".to_string();
        raster.raster_sha256 = "different-raster".to_string();

        let base = ocr_shard_cache_key(&page);

        assert_ne!(base, ocr_shard_cache_key(&other_page));
        assert_ne!(base, ocr_shard_cache_key(&region));
        assert_ne!(base, ocr_shard_cache_key(&profile));
        assert_ne!(base, ocr_shard_cache_key(&raster));
    }

    #[test]
    fn cache_roundtrips_successful_result() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
        let input = sample_ocr_input(0, "page");
        let result = PdfOcrShardResult::succeeded(&input, "cached text", 0.97);

        assert!(cache.store_successful(&input, &result)?);
        let resolution = cache.resolve(std::slice::from_ref(&input));
        let merged = resolution.merge(Vec::new())?;

        assert_eq!(merged, vec![result]);
        Ok(())
    }

    #[test]
    fn cache_merges_hits_and_live_misses_in_input_order() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
        let inputs = vec![
            sample_ocr_input(0, "page"),
            sample_ocr_input(1, "page"),
            sample_ocr_input(2, "page"),
        ];
        let hit_zero = PdfOcrShardResult::succeeded(&inputs[0], "cached 0", 1.0);
        let hit_two = PdfOcrShardResult::succeeded(&inputs[2], "cached 2", 1.0);
        cache.store_successful(&inputs[0], &hit_zero)?;
        cache.store_successful(&inputs[2], &hit_two)?;

        let resolution = cache.resolve(inputs.as_slice());

        assert_eq!(resolution.hit_count(), 2);
        assert_eq!(resolution.misses().len(), 1);
        assert_eq!(resolution.misses()[0].shard_element_id, "page-shard-1");

        let live = vec![PdfOcrShardResult::succeeded(&inputs[1], "live 1", 1.0)];
        let merged = resolution.merge(live)?;

        assert_eq!(merged[0].text.as_deref(), Some("cached 0"));
        assert_eq!(merged[1].text.as_deref(), Some("live 1"));
        assert_eq!(merged[2].text.as_deref(), Some("cached 2"));
        Ok(())
    }

    #[test]
    fn cache_does_not_persist_failed_results() -> Result<(), String> {
        let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
        let cache = PdfOcrShardCache::new(temp.path().to_path_buf());
        let input = sample_ocr_input(0, "page");
        let failed = PdfOcrShardResult::failed(&input, "transient failure");

        assert!(!cache.store_successful(&input, &failed)?);
        let resolution = cache.resolve(std::slice::from_ref(&input));

        assert_eq!(resolution.hit_count(), 0);
        assert_eq!(resolution.misses().len(), 1);
        Ok(())
    }

    fn sample_ocr_input(page_index: u32, shard_type: &str) -> PdfOcrShardInput {
        PdfOcrShardInput {
            contract_version: PDF_OCR_SHARD_INPUT_SCHEMA_VERSION.to_string(),
            source_path: "/tmp/source.pdf".to_string(),
            source_content_hash: "sourcehash".to_string(),
            page_index,
            image_path: format!("/tmp/page-{page_index:05}.png"),
            image_mime_type: "image/png".to_string(),
            raster_sha256: format!("rasterhash-{page_index}"),
            render_profile: "pdfium-render-page-shards-v1".to_string(),
            ocr_profile: "docling-compatible-page-ocr-v1".to_string(),
            ocr_engine: "docling-compatible-ocr".to_string(),
            preferred_languages: vec!["auto".to_string()],
            min_confidence: 0.0,
            preserve_layout: true,
            raster_width_px: 2400,
            raster_height_px: 3100,
            render_dpi: 300,
            rotation_degrees: 0,
            crop_left: 0.0,
            crop_bottom: 0.0,
            crop_right: 612.0,
            crop_top: 792.0,
            point_to_pixel_scale_x: 3.921_568_627,
            point_to_pixel_scale_y: 3.914_141_414,
            shard_element_id: format!("{shard_type}-shard-{page_index}"),
            shard_type: shard_type.to_string(),
            region_index: 0,
            parent_shard_element_id: String::new(),
            reading_order_key: format!("{page_index:06}.000000"),
            source_page_pixel_left: 0,
            source_page_pixel_top: 0,
            source_page_pixel_right: 2400,
            source_page_pixel_bottom: 3100,
        }
    }
}
