//! PDF render artifact cache key builders.

#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactKind, AttachmentArtifactKeyParts, attachment_artifact_key,
};

use crate::pdf::render::artifact_cache::xiuxian_db_store_key;
#[cfg(feature = "foyer-artifact-cache")]
use crate::pdf::render::identity::sha256_hex;
use crate::pdf::render::types::{PdfPageRegionRenderRequest, PdfPageRenderProfile};

use super::model::RegionCropArtifactIdentity;

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
