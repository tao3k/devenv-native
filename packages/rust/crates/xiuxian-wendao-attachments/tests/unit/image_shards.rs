use image::{ImageBuffer, Rgba};

use super::{ImageShardOptions, materialize_image_shards, plan_image_shards};

fn write_sample_png(path: &std::path::Path, width: u32, height: u32) -> Result<(), String> {
    let image = ImageBuffer::from_fn(width, height, |x, y| {
        Rgba([
            u8::try_from(x % 251).unwrap_or_default(),
            u8::try_from(y % 251).unwrap_or_default(),
            u8::try_from((x + y) % 251).unwrap_or_default(),
            255,
        ])
    });
    image
        .save(path)
        .map_err(|error| format!("write sample image: {error}"))
}

#[test]
fn image_shards_plan_single_tile_when_source_fits() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp_dir.path().join("source.png");
    write_sample_png(source.as_path(), 32, 24)?;

    let plan = plan_image_shards(
        source.as_path(),
        ImageShardOptions {
            max_tile_width: 64,
            max_tile_height: 64,
            overlap: 0,
        },
    )?;

    assert_eq!(plan.source_width, 32);
    assert_eq!(plan.source_height, 24);
    assert_eq!(plan.output_mime_type.as_str(), "image/png");
    assert_eq!(plan.shards.len(), 1);
    assert_eq!(plan.shards[0].tile_box.width, 32);
    assert_eq!(plan.shards[0].tile_box.height, 24);
    assert_eq!(plan.shards[0].reading_order_key, "000000");
    Ok(())
}

#[test]
fn image_shards_plan_row_major_tiles_without_downsampling() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp_dir.path().join("source.png");
    write_sample_png(source.as_path(), 5, 4)?;

    let plan = plan_image_shards(
        source.as_path(),
        ImageShardOptions {
            max_tile_width: 3,
            max_tile_height: 3,
            overlap: 0,
        },
    )?;

    assert_eq!(
        plan.shards
            .iter()
            .map(|shard| {
                (
                    shard.tile_box.left,
                    shard.tile_box.top,
                    shard.tile_box.width,
                    shard.tile_box.height,
                    shard.reading_order_key.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (0, 0, 3, 3, "000000"),
            (3, 0, 2, 3, "000001"),
            (0, 3, 3, 1, "000002"),
            (3, 3, 2, 1, "000003"),
        ]
    );
    Ok(())
}

#[test]
fn image_shards_materialize_lossless_png_tiles() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp_dir.path().join("source.png");
    let output_root = temp_dir.path().join("shards");
    write_sample_png(source.as_path(), 5, 4)?;

    let manifest = materialize_image_shards(
        source.as_path(),
        output_root.as_path(),
        ImageShardOptions {
            max_tile_width: 3,
            max_tile_height: 3,
            overlap: 0,
        },
    )?;

    assert_eq!(manifest.tiles.len(), 4);
    for tile in &manifest.tiles {
        assert!(tile.image_path.exists());
        assert_eq!(tile.image_mime_type.as_str(), "image/png");
        assert_eq!(tile.width, tile.spec.tile_box.width);
        assert_eq!(tile.height, tile.spec.tile_box.height);
        assert!(!tile.raster_sha256.is_empty());
        let dimensions = image::image_dimensions(tile.image_path.as_path())
            .map_err(|error| format!("read tile dimensions: {error}"))?;
        assert_eq!(dimensions, (tile.width, tile.height));
    }
    Ok(())
}

#[test]
fn image_shards_reject_overlap_that_prevents_progress() -> Result<(), String> {
    let error = match (ImageShardOptions {
        max_tile_width: 64,
        max_tile_height: 64,
        overlap: 64,
    }
    .validate())
    {
        Ok(_) => return Err("overlap equal to tile width should fail".to_string()),
        Err(error) => error,
    };

    assert!(error.contains("overlap"));
    Ok(())
}
