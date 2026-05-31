//! PDF render artifact cache runtime models and counters.

#[cfg(feature = "foyer-artifact-cache")]
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{ArtifactBlobCacheBackend, ArtifactReadThrough};

use crate::pdf::render::types::{PdfPageBox, PdfPageRenderProfile};

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) struct PdfRenderArtifactCache {
    pub(super) backend: Arc<ArtifactBlobCacheBackend>,
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
pub(super) enum PdfRenderArtifactRecordKind {
    PageRaster,
    RegionCrop,
    RegionManifestProjection,
    RegionManifestProjectionRow,
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
impl PdfRenderArtifactCache {
    pub(super) fn new(backend: Arc<ArtifactBlobCacheBackend>) -> Self {
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

    pub(super) fn record(&self, kind: PdfRenderArtifactRecordKind, artifact: &ArtifactReadThrough) {
        if artifact.cache_hit() {
            self.record_hit(kind, artifact.byte_len());
        } else if artifact.cache_throttled() {
            self.record_throttled(kind);
        } else {
            self.record_miss(kind);
        }
    }

    pub(super) fn record_hit(&self, kind: PdfRenderArtifactRecordKind, byte_len: usize) {
        self.stats.total.record_hit(byte_len);
        self.stats.for_kind(kind).record_hit(byte_len);
    }

    pub(super) fn record_miss(&self, kind: PdfRenderArtifactRecordKind) {
        self.stats.total.record_miss();
        self.stats.for_kind(kind).record_miss();
    }

    pub(super) fn record_throttled(&self, kind: PdfRenderArtifactRecordKind) {
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

    pub(super) fn record_hit(&self, byte_len: usize) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.bytes
            .fetch_add(usize_to_u64(byte_len), Ordering::Relaxed);
    }

    pub(super) fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_throttled(&self) {
        self.throttled.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "foyer-artifact-cache")]
impl Drop for PdfRenderArtifactCache {
    fn drop(&mut self) {
        let _ = self.backend.close();
    }
}

#[cfg(feature = "foyer-artifact-cache")]
fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
