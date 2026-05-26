//! Materialization for standalone image attachment shards.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use image::{DynamicImage, ImageFormat};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::plan::{output_profile, plan_image_shards, sha256_hex};
use super::types::{
    ImageMimeType, ImageShardManifest, ImageShardOptions, ImageShardSpec, MaterializedImageShard,
};

/// Materialize standalone image shards as lossless PNG tile files.
///
/// # Errors
///
/// Returns an error when planning fails, the source image cannot be decoded, an
/// output directory cannot be created, or a tile cannot be written/read back.
pub fn materialize_image_shards(
    source_path: &Path,
    output_root: &Path,
    options: ImageShardOptions,
) -> Result<ImageShardManifest, String> {
    let plan = plan_image_shards(source_path, options)?;
    let output_dir = output_root
        .join(plan.source_content_hash.as_str())
        .join(output_profile());
    fs::create_dir_all(output_dir.as_path())
        .map_err(|error| format!("create image shard dir `{}`: {error}", output_dir.display()))?;

    let image =
        Arc::new(image::open(source_path).map_err(|error| {
            format!("decode image source `{}`: {error}", source_path.display())
        })?);
    let tiles = plan
        .shards
        .par_iter()
        .map(|spec| materialize_one(image.as_ref(), output_dir.as_path(), spec))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ImageShardManifest { plan, tiles })
}

fn materialize_one(
    image: &DynamicImage,
    output_dir: &Path,
    spec: &ImageShardSpec,
) -> Result<MaterializedImageShard, String> {
    let tile = image.crop_imm(
        spec.tile_box.left,
        spec.tile_box.top,
        spec.tile_box.width,
        spec.tile_box.height,
    );
    let image_path = output_dir.join(format!("image-shard-{:06}.png", spec.shard_index));
    tile.save_with_format(image_path.as_path(), ImageFormat::Png)
        .map_err(|error| format!("write image shard `{}`: {error}", image_path.display()))?;
    let bytes = fs::read(image_path.as_path())
        .map_err(|error| format!("read image shard `{}`: {error}", image_path.display()))?;
    Ok(MaterializedImageShard {
        spec: spec.clone(),
        image_path,
        raster_sha256: sha256_hex(bytes.as_slice()),
        byte_len: u64::try_from(bytes.len()).map_err(|_| "image shard byte length exceeds u64")?,
        width: spec.tile_box.width,
        height: spec.tile_box.height,
        image_mime_type: ImageMimeType::Png,
    })
}
