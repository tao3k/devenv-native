//! Artifact-cache helpers for rendered PDF raster bytes.

#[cfg(feature = "foyer-artifact-cache")]
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
#[cfg(feature = "foyer-artifact-cache")]
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use arrow::record_batch::RecordBatch;
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ARTIFACT_CACHE_BACKEND_ENV, ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV, ARTIFACT_CACHE_FLUSHERS_ENV,
    ARTIFACT_CACHE_MEMORY_BYTES_ENV, ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
    ARTIFACT_CACHE_RECLAIMERS_ENV, ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV, ARTIFACT_CACHE_ROOT_ENV,
    ARTIFACT_CACHE_RUNTIME_WORKERS_ENV, ARTIFACT_CACHE_STORAGE_BYTES_ENV, ArtifactBlobCache,
    ArtifactBlobCacheBackend, ArtifactBlobCacheBackendConfig, ArtifactBlobReadStatus,
    ArtifactCacheError, ArtifactKind, ArtifactReadThrough, AttachmentArtifactKeyParts,
    attachment_artifact_key, read_through_artifact_bytes,
};
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::{decode_record_batches_ipc, write_record_batches_ipc_artifact};

use super::xiuxian_db_store_key;
use crate::pdf::render::identity::sha256_hex;
use crate::pdf::render::types::{
    PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile, RenderedRasterIdentity,
};

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) struct PdfRenderArtifactCache {
    backend: Arc<ArtifactBlobCacheBackend>,
    stats: PdfRenderArtifactCacheCounters,
}

#[cfg(not(feature = "foyer-artifact-cache"))]
pub(crate) struct PdfRenderArtifactCache;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PdfRenderArtifactCacheStats {
    pub(crate) backend_name: Option<String>,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) throttled_count: u64,
    pub(crate) byte_count: u64,
    pub(crate) page_raster_hit_count: u64,
    pub(crate) page_raster_miss_count: u64,
    pub(crate) page_raster_throttled_count: u64,
    pub(crate) page_raster_byte_count: u64,
    pub(crate) region_crop_hit_count: u64,
    pub(crate) region_crop_miss_count: u64,
    pub(crate) region_crop_throttled_count: u64,
    pub(crate) region_crop_byte_count: u64,
    pub(crate) region_manifest_projection_hit_count: u64,
    pub(crate) region_manifest_projection_miss_count: u64,
    pub(crate) region_manifest_projection_throttled_count: u64,
    pub(crate) region_manifest_projection_byte_count: u64,
    pub(crate) region_manifest_projection_row_hit_count: u64,
    pub(crate) region_manifest_projection_row_miss_count: u64,
    pub(crate) region_manifest_projection_row_throttled_count: u64,
    pub(crate) region_manifest_projection_row_byte_count: u64,
}

#[cfg(feature = "foyer-artifact-cache")]
#[derive(Default)]
struct PdfRenderArtifactCacheCounters {
    total: PdfRenderArtifactCounter,
    page_raster: PdfRenderArtifactCounter,
    region_crop: PdfRenderArtifactCounter,
    region_manifest_projection: PdfRenderArtifactCounter,
    region_manifest_projection_row: PdfRenderArtifactCounter,
}

#[cfg(feature = "foyer-artifact-cache")]
#[derive(Default)]
struct PdfRenderArtifactCounter {
    hits: AtomicU64,
    misses: AtomicU64,
    throttled: AtomicU64,
    bytes: AtomicU64,
}

#[cfg(feature = "foyer-artifact-cache")]
#[derive(Debug, Clone, Copy, Default)]
struct PdfRenderArtifactCounterStats {
    hits: u64,
    misses: u64,
    throttled: u64,
    bytes: u64,
}

#[derive(Debug, Clone, Copy)]
enum PdfRenderArtifactRecordKind {
    PageRaster,
    RegionCrop,
    RegionManifestProjection,
    RegionManifestProjectionRow,
}

#[cfg(feature = "foyer-artifact-cache")]
static PDF_RENDER_ARTIFACT_CACHE_BACKENDS: OnceLock<
    Mutex<BTreeMap<String, Arc<ArtifactBlobCacheBackend>>>,
> = OnceLock::new();

#[cfg(feature = "foyer-artifact-cache")]
impl PdfRenderArtifactCache {
    fn new(backend: Arc<ArtifactBlobCacheBackend>) -> Self {
        Self {
            backend,
            stats: PdfRenderArtifactCacheCounters::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_backend(backend: ArtifactBlobCacheBackend) -> Self {
        Self::new(Arc::new(backend))
    }

    pub(crate) fn snapshot(&self) -> PdfRenderArtifactCacheStats {
        let total = self.stats.total.snapshot();
        let page_raster = self.stats.page_raster.snapshot();
        let region_crop = self.stats.region_crop.snapshot();
        let region_manifest_projection = self.stats.region_manifest_projection.snapshot();
        let region_manifest_projection_row = self.stats.region_manifest_projection_row.snapshot();
        PdfRenderArtifactCacheStats {
            backend_name: Some(self.backend.backend_name().to_string()),
            hit_count: total.hits,
            miss_count: total.misses,
            throttled_count: total.throttled,
            byte_count: total.bytes,
            page_raster_hit_count: page_raster.hits,
            page_raster_miss_count: page_raster.misses,
            page_raster_throttled_count: page_raster.throttled,
            page_raster_byte_count: page_raster.bytes,
            region_crop_hit_count: region_crop.hits,
            region_crop_miss_count: region_crop.misses,
            region_crop_throttled_count: region_crop.throttled,
            region_crop_byte_count: region_crop.bytes,
            region_manifest_projection_hit_count: region_manifest_projection.hits,
            region_manifest_projection_miss_count: region_manifest_projection.misses,
            region_manifest_projection_throttled_count: region_manifest_projection.throttled,
            region_manifest_projection_byte_count: region_manifest_projection.bytes,
            region_manifest_projection_row_hit_count: region_manifest_projection_row.hits,
            region_manifest_projection_row_miss_count: region_manifest_projection_row.misses,
            region_manifest_projection_row_throttled_count: region_manifest_projection_row
                .throttled,
            region_manifest_projection_row_byte_count: region_manifest_projection_row.bytes,
        }
    }

    fn record(&self, kind: PdfRenderArtifactRecordKind, artifact: &ArtifactReadThrough) {
        if artifact.cache_hit() {
            self.record_hit(kind, artifact.byte_len());
        } else if artifact.cache_throttled() {
            self.record_throttled(kind);
        } else {
            self.record_miss(kind);
        }
    }

    fn record_hit(&self, kind: PdfRenderArtifactRecordKind, byte_len: usize) {
        self.stats.total.record_hit(byte_len);
        self.stats.for_kind(kind).record_hit(byte_len);
    }

    fn record_miss(&self, kind: PdfRenderArtifactRecordKind) {
        self.stats.total.record_miss();
        self.stats.for_kind(kind).record_miss();
    }

    fn record_throttled(&self, kind: PdfRenderArtifactRecordKind) {
        self.stats.total.record_throttled();
        self.stats.for_kind(kind).record_throttled();
    }
}

#[cfg(feature = "foyer-artifact-cache")]
impl PdfRenderArtifactCacheCounters {
    fn for_kind(&self, kind: PdfRenderArtifactRecordKind) -> &PdfRenderArtifactCounter {
        match kind {
            PdfRenderArtifactRecordKind::PageRaster => &self.page_raster,
            PdfRenderArtifactRecordKind::RegionCrop => &self.region_crop,
            PdfRenderArtifactRecordKind::RegionManifestProjection => {
                &self.region_manifest_projection
            }
            PdfRenderArtifactRecordKind::RegionManifestProjectionRow => {
                &self.region_manifest_projection_row
            }
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
impl PdfRenderArtifactCounter {
    fn snapshot(&self) -> PdfRenderArtifactCounterStats {
        PdfRenderArtifactCounterStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            throttled: self.throttled.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
        }
    }

    fn record_hit(&self, byte_len: usize) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.bytes
            .fetch_add(usize_to_u64(byte_len), Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_throttled(&self) {
        self.throttled.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "foyer-artifact-cache")]
impl Drop for PdfRenderArtifactCache {
    fn drop(&mut self) {
        let _ = self.backend.close();
    }
}

pub(crate) fn pdf_render_artifact_cache_from_environment()
-> Result<Option<PdfRenderArtifactCache>, String> {
    #[cfg(feature = "foyer-artifact-cache")]
    {
        if !artifact_cache_env_present() {
            return Ok(None);
        }
        let config = ArtifactBlobCacheBackendConfig::from_env()
            .map_err(|error| format!("resolve PDF render ArtifactBlobCache backend: {error}"))?;
        let backend = shared_pdf_render_artifact_cache_backend(&config)?;
        Ok(Some(PdfRenderArtifactCache::new(backend)))
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        if env::var("WENDAO_ARTIFACT_CACHE_BACKEND").is_ok()
            || env::var("WENDAO_ARTIFACT_CACHE_ROOT").is_ok()
        {
            return Err(
                "PDF render artifact cache is configured but foyer-artifact-cache is not enabled"
                    .to_string(),
            );
        }
        Ok(None)
    }
}

#[cfg(feature = "foyer-artifact-cache")]
fn shared_pdf_render_artifact_cache_backend(
    config: &ArtifactBlobCacheBackendConfig,
) -> Result<Arc<ArtifactBlobCacheBackend>, String> {
    let key = artifact_cache_backend_config_key(config);
    let backends = PDF_RENDER_ARTIFACT_CACHE_BACKENDS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut backends = backends
        .lock()
        .map_err(|_| "PDF render ArtifactBlobCache backend registry lock poisoned".to_string())?;
    if let Some(backend) = backends.get(&key) {
        return Ok(Arc::clone(backend));
    }
    let backend = Arc::new(
        config
            .build()
            .map_err(|error| format!("build PDF render ArtifactBlobCache backend: {error}"))?,
    );
    backends.insert(key, Arc::clone(&backend));
    Ok(backend)
}

#[cfg(feature = "foyer-artifact-cache")]
fn artifact_cache_backend_config_key(config: &ArtifactBlobCacheBackendConfig) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        config.kind().as_str(),
        config.root().display(),
        config.memory_capacity_bytes(),
        config.storage_capacity_bytes(),
        config.runtime_worker_threads(),
        config.memory_shards(),
        config.block_size_bytes(),
        config.recover_concurrency(),
        config.flushers(),
        config.reclaimers()
    )
}

pub(crate) fn materialize_page_raster_identity(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    materialize_raster_identity(
        cache,
        PdfRenderArtifactRecordKind::PageRaster,
        || page_raster_artifact_key(source_hash, profile, page_index),
        image_path,
        build_bytes,
    )
}

pub(crate) fn materialize_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    identity: RegionCropArtifactIdentity<'_>,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        let _ = (
            identity.source_hash,
            identity.profile,
            identity.page_index,
            identity.region_index,
            identity.region_box,
        );
    }
    materialize_raster_identity(
        cache,
        PdfRenderArtifactRecordKind::RegionCrop,
        || region_crop_artifact_key(identity),
        image_path,
        build_bytes,
    )
}

pub(crate) fn materialize_requested_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
    region_box: PdfPageBox,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    materialize_region_crop_identity(
        cache,
        RegionCropArtifactIdentity {
            source_hash,
            profile,
            page_index: request.page_index,
            region_index: request.region_index,
            region_box,
        },
        image_path,
        build_bytes,
    )
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn restore_requested_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
    region_box: PdfPageBox,
    image_path: &Path,
) -> Result<Option<RenderedRasterIdentity>, String> {
    restore_region_crop_identity(
        cache,
        RegionCropArtifactIdentity {
            source_hash,
            profile,
            page_index: request.page_index,
            region_index: request.region_index,
            region_box,
        },
        image_path,
    )
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn restore_region_manifest_projection(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Option<Vec<RecordBatch>>, String> {
    let Some(cache) = cache else {
        return Ok(None);
    };
    let key = region_manifest_projection_artifact_key(source_hash, profile, page_index, regions)?;
    match cache
        .backend
        .read_with_status(&key)
        .map_err(|error| format!("read PDF region manifest projection artifact: {error}"))?
    {
        ArtifactBlobReadStatus::Hit(read) => {
            cache.record_hit(
                PdfRenderArtifactRecordKind::RegionManifestProjection,
                read.byte_len(),
            );
            decode_record_batches_ipc(read.bytes())
                .map(Some)
                .map_err(|error| format!("decode PDF region manifest projection: {error}"))
        }
        ArtifactBlobReadStatus::Miss => {
            cache.record_miss(PdfRenderArtifactRecordKind::RegionManifestProjection);
            Ok(None)
        }
        ArtifactBlobReadStatus::Throttled => {
            cache.record_throttled(PdfRenderArtifactRecordKind::RegionManifestProjection);
            Ok(None)
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn restore_region_manifest_projection_row(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
) -> Result<Option<Vec<RecordBatch>>, String> {
    let Some(cache) = cache else {
        return Ok(None);
    };
    let key = region_manifest_projection_row_artifact_key(source_hash, profile, request)?;
    match cache
        .backend
        .read_with_status(&key)
        .map_err(|error| format!("read PDF region manifest row projection artifact: {error}"))?
    {
        ArtifactBlobReadStatus::Hit(read) => {
            cache.record_hit(
                PdfRenderArtifactRecordKind::RegionManifestProjectionRow,
                read.byte_len(),
            );
            decode_record_batches_ipc(read.bytes())
                .map(Some)
                .map_err(|error| format!("decode PDF region manifest row projection: {error}"))
        }
        ArtifactBlobReadStatus::Miss => {
            cache.record_miss(PdfRenderArtifactRecordKind::RegionManifestProjectionRow);
            Ok(None)
        }
        ArtifactBlobReadStatus::Throttled => {
            cache.record_throttled(PdfRenderArtifactRecordKind::RegionManifestProjectionRow);
            Ok(None)
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn write_region_manifest_projection(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    regions: &[PdfPageRegionRenderRequest],
    batches: &[RecordBatch],
) -> Result<(), String> {
    if let Some(cache) = cache {
        let key =
            region_manifest_projection_artifact_key(source_hash, profile, page_index, regions)?;
        write_record_batches_ipc_artifact(cache.backend.as_ref(), &key, batches)
            .map_err(|error| format!("write PDF region manifest projection artifact: {error}"))?;
    }
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn write_region_manifest_projection_row(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
    batches: &[RecordBatch],
) -> Result<(), String> {
    if let Some(cache) = cache {
        let key = region_manifest_projection_row_artifact_key(source_hash, profile, request)?;
        write_record_batches_ipc_artifact(cache.backend.as_ref(), &key, batches).map_err(
            |error| format!("write PDF region manifest row projection artifact: {error}"),
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegionCropArtifactIdentity<'a> {
    pub(crate) source_hash: &'a str,
    pub(crate) profile: &'a PdfPageRenderProfile,
    pub(crate) page_index: u32,
    pub(crate) region_index: u32,
    pub(crate) region_box: PdfPageBox,
}

#[cfg(feature = "foyer-artifact-cache")]
fn restore_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    identity: RegionCropArtifactIdentity<'_>,
    image_path: &Path,
) -> Result<Option<RenderedRasterIdentity>, String> {
    let Some(cache) = cache else {
        return Ok(None);
    };
    let key = region_crop_artifact_key(identity)?;
    match cache
        .backend
        .read_with_status(&key)
        .map_err(|error| format!("read PDF region crop artifact cache entry: {error}"))?
    {
        ArtifactBlobReadStatus::Hit(read) => {
            cache.record_hit(PdfRenderArtifactRecordKind::RegionCrop, read.byte_len());
            write_restored_bytes(image_path, read.bytes())?;
            raster_identity_from_bytes(image_path, read.bytes()).map(Some)
        }
        ArtifactBlobReadStatus::Miss => {
            cache.record_miss(PdfRenderArtifactRecordKind::RegionCrop);
            Ok(None)
        }
        ArtifactBlobReadStatus::Throttled => {
            cache.record_throttled(PdfRenderArtifactRecordKind::RegionCrop);
            Ok(None)
        }
    }
}

pub(crate) fn save_image_bytes(
    image: &image::DynamicImage,
    image_path: &Path,
) -> Result<Vec<u8>, String> {
    image
        .save(image_path)
        .map_err(|error| format!("write shard image `{}`: {error}", image_path.display()))?;
    fs::read(image_path)
        .map_err(|error| format!("read shard image `{}`: {error}", image_path.display()))
}

fn materialize_raster_identity(
    cache: Option<&PdfRenderArtifactCache>,
    #[cfg_attr(not(feature = "foyer-artifact-cache"), allow(unused_variables))]
    kind: PdfRenderArtifactRecordKind,
    key: impl FnOnce() -> Result<xiuxian_db_store_key::ArtifactKey, String>,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    #[cfg(feature = "foyer-artifact-cache")]
    if let Some(cache) = cache {
        let key = key()?;
        let artifact = read_through_artifact_bytes(cache.backend.as_ref(), &key, || {
            build_bytes().map_err(|error| {
                ArtifactCacheError::backend("pdf-render", "materializing raster bytes", error)
            })
        })
        .map_err(|error| format!("materialize PDF render artifact cache entry: {error}"))?;
        cache.record(kind, &artifact);
        write_restored_bytes(image_path, artifact.bytes())?;
        return raster_identity_from_bytes(image_path, artifact.bytes());
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        let _ = cache;
        let _ = key;
    }

    let bytes = build_bytes()?;
    raster_identity_from_bytes(image_path, bytes.as_slice())
}

#[cfg(feature = "foyer-artifact-cache")]
fn write_restored_bytes(image_path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = image_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create PDF render artifact output directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(image_path, bytes).map_err(|error| {
        format!(
            "restore PDF render artifact `{}`: {error}",
            image_path.display()
        )
    })
}

fn raster_identity_from_bytes(
    image_path: &Path,
    bytes: &[u8],
) -> Result<RenderedRasterIdentity, String> {
    let image = image::load_from_memory(bytes).map_err(|error| {
        format!(
            "decode PDF render artifact `{}`: {error}",
            image_path.display()
        )
    })?;
    Ok(RenderedRasterIdentity {
        path: image_path.to_path_buf(),
        sha256: sha256_hex(bytes),
        width_px: image.width(),
        height_px: image.height(),
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn page_raster_artifact_key(
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
) -> Result<xiuxian_db_store_key::ArtifactKey, String> {
    attachment_artifact_key(AttachmentArtifactKeyParts {
        kind: ArtifactKind::PdfPageRaster,
        source_digest: source_hash.to_string(),
        profile_digest: profile_digest(profile, "page-raster"),
        shard_digest: digest_component([page_index.to_string()]),
    })
    .map_err(|error| error.to_string())
}

#[cfg(not(feature = "foyer-artifact-cache"))]
fn page_raster_artifact_key(
    _source_hash: &str,
    _profile: &PdfPageRenderProfile,
    _page_index: u32,
) -> Result<xiuxian_db_store_key::ArtifactKey, String> {
    unreachable!("artifact keys are unused without foyer-artifact-cache")
}

#[cfg(feature = "foyer-artifact-cache")]
fn region_crop_artifact_key(
    identity: RegionCropArtifactIdentity<'_>,
) -> Result<xiuxian_db_store_key::ArtifactKey, String> {
    attachment_artifact_key(AttachmentArtifactKeyParts {
        kind: ArtifactKind::OcrRegionCrop,
        source_digest: identity.source_hash.to_string(),
        profile_digest: profile_digest(identity.profile, "region-crop"),
        shard_digest: digest_component([
            identity.page_index.to_string(),
            identity.region_index.to_string(),
            f64_bits(identity.region_box.left),
            f64_bits(identity.region_box.bottom),
            f64_bits(identity.region_box.right),
            f64_bits(identity.region_box.top),
        ]),
    })
    .map_err(|error| error.to_string())
}

#[cfg(feature = "foyer-artifact-cache")]
fn region_manifest_projection_artifact_key(
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<xiuxian_db_store_key::ArtifactKey, String> {
    let mut shard_fragments = vec![page_index.to_string()];
    for region in regions {
        shard_fragments.extend([
            region.page_index.to_string(),
            region.region_index.to_string(),
            f64_bits(region.region_box.left),
            f64_bits(region.region_box.bottom),
            f64_bits(region.region_box.right),
            f64_bits(region.region_box.top),
            region.effective_reading_order_key(),
        ]);
    }
    attachment_artifact_key(AttachmentArtifactKeyParts {
        kind: ArtifactKind::ArrowIpcBatch,
        source_digest: source_hash.to_string(),
        profile_digest: profile_digest(profile, "region-manifest-projection"),
        shard_digest: digest_component(shard_fragments),
    })
    .map_err(|error| error.to_string())
}

#[cfg(feature = "foyer-artifact-cache")]
fn region_manifest_projection_row_artifact_key(
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
) -> Result<xiuxian_db_store_key::ArtifactKey, String> {
    attachment_artifact_key(AttachmentArtifactKeyParts {
        kind: ArtifactKind::ArrowIpcBatch,
        source_digest: source_hash.to_string(),
        profile_digest: profile_digest(profile, "region-manifest-projection-row"),
        shard_digest: digest_component([
            request.page_index.to_string(),
            request.region_index.to_string(),
            f64_bits(request.region_box.left),
            f64_bits(request.region_box.bottom),
            f64_bits(request.region_box.right),
            f64_bits(request.region_box.top),
            request.effective_reading_order_key(),
        ]),
    })
    .map_err(|error| error.to_string())
}

#[cfg(not(feature = "foyer-artifact-cache"))]
fn region_crop_artifact_key(
    _identity: RegionCropArtifactIdentity<'_>,
) -> Result<xiuxian_db_store_key::ArtifactKey, String> {
    unreachable!("artifact keys are unused without foyer-artifact-cache")
}

#[cfg(feature = "foyer-artifact-cache")]
fn profile_digest(profile: &PdfPageRenderProfile, materialization: &str) -> String {
    digest_component([
        profile.profile_id.clone(),
        profile.dpi.to_string(),
        profile.image_extension.clone(),
        profile.image_mime_type.clone(),
        profile.render_annotations.to_string(),
        profile.render_form_data.to_string(),
        materialization.to_string(),
    ])
}

#[cfg(feature = "foyer-artifact-cache")]
fn digest_component(fragments: impl IntoIterator<Item = String>) -> String {
    let mut bytes = Vec::new();
    for fragment in fragments {
        bytes.extend_from_slice(fragment.as_bytes());
        bytes.push(0);
    }
    sha256_hex(bytes.as_slice())
}

#[cfg(feature = "foyer-artifact-cache")]
fn f64_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

#[cfg(feature = "foyer-artifact-cache")]
fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[cfg(feature = "foyer-artifact-cache")]
fn artifact_cache_env_present() -> bool {
    [
        ARTIFACT_CACHE_BACKEND_ENV,
        ARTIFACT_CACHE_ROOT_ENV,
        ARTIFACT_CACHE_MEMORY_BYTES_ENV,
        ARTIFACT_CACHE_STORAGE_BYTES_ENV,
        ARTIFACT_CACHE_RUNTIME_WORKERS_ENV,
        ARTIFACT_CACHE_MEMORY_SHARDS_ENV,
        ARTIFACT_CACHE_BLOCK_SIZE_BYTES_ENV,
        ARTIFACT_CACHE_RECOVER_CONCURRENCY_ENV,
        ARTIFACT_CACHE_FLUSHERS_ENV,
        ARTIFACT_CACHE_RECLAIMERS_ENV,
        "PRJ_CACHE_HOME",
    ]
    .iter()
    .any(|key| env::var(key).is_ok_and(|value| !value.trim().is_empty()))
}
