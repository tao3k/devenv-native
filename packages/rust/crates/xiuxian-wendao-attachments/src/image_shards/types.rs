//! Data types for standalone image shard planning.

use std::path::PathBuf;

/// Default maximum tile width before a standalone image is split.
pub const DEFAULT_IMAGE_TILE_WIDTH_PX: u32 = 2048;
/// Default maximum tile height before a standalone image is split.
pub const DEFAULT_IMAGE_TILE_HEIGHT_PX: u32 = 2048;

/// Supported image MIME tokens for standalone shard planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageMimeType {
    /// Portable Network Graphics.
    Png,
    /// JPEG image.
    Jpeg,
    /// Tagged Image File Format.
    Tiff,
    /// Bitmap image.
    Bmp,
    /// WebP image.
    Webp,
    /// GIF image.
    Gif,
}

impl ImageMimeType {
    /// Return the stable MIME token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Tiff => "image/tiff",
            Self::Bmp => "image/bmp",
            Self::Webp => "image/webp",
            Self::Gif => "image/gif",
        }
    }
}

impl TryFrom<&str> for ImageMimeType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "image/png" => Ok(Self::Png),
            "image/jpeg" => Ok(Self::Jpeg),
            "image/tiff" => Ok(Self::Tiff),
            "image/bmp" => Ok(Self::Bmp),
            "image/webp" => Ok(Self::Webp),
            "image/gif" => Ok(Self::Gif),
            value => Err(format!("unsupported image shard MIME type `{value}`")),
        }
    }
}

/// Pixel rectangle for one standalone image tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageTileBox {
    /// Left pixel coordinate, zero-based from the source image origin.
    pub left: u32,
    /// Top pixel coordinate, zero-based from the source image origin.
    pub top: u32,
    /// Tile width in pixels.
    pub width: u32,
    /// Tile height in pixels.
    pub height: u32,
}

impl ImageTileBox {
    /// Build a validated tile box.
    ///
    /// # Errors
    ///
    /// Returns an error when width or height is zero.
    pub fn new(left: u32, top: u32, width: u32, height: u32) -> Result<Self, String> {
        if width == 0 || height == 0 {
            return Err("image tile dimensions must be positive".to_string());
        }
        Ok(Self {
            left,
            top,
            width,
            height,
        })
    }

    /// Return the exclusive right pixel coordinate.
    #[must_use]
    pub fn right_px(self) -> u32 {
        self.left.saturating_add(self.width)
    }

    /// Return the exclusive bottom pixel coordinate.
    #[must_use]
    pub fn bottom_px(self) -> u32 {
        self.top.saturating_add(self.height)
    }
}

/// Planner controls for standalone image tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageShardOptions {
    /// Maximum emitted tile width in pixels.
    pub max_tile_width: u32,
    /// Maximum emitted tile height in pixels.
    pub max_tile_height: u32,
    /// Optional overlap between adjacent tiles in pixels.
    pub overlap: u32,
}

impl Default for ImageShardOptions {
    fn default() -> Self {
        Self {
            max_tile_width: DEFAULT_IMAGE_TILE_WIDTH_PX,
            max_tile_height: DEFAULT_IMAGE_TILE_HEIGHT_PX,
            overlap: 0,
        }
    }
}

impl ImageShardOptions {
    /// Validate planner options.
    ///
    /// # Errors
    ///
    /// Returns an error when max tile dimensions are zero or overlap would
    /// prevent forward progress.
    pub fn validate(self) -> Result<Self, String> {
        if self.max_tile_width == 0 || self.max_tile_height == 0 {
            return Err("image shard max tile dimensions must be positive".to_string());
        }
        if self.overlap >= self.max_tile_width || self.overlap >= self.max_tile_height {
            return Err(
                "image shard overlap must be smaller than both tile dimensions".to_string(),
            );
        }
        Ok(self)
    }
}

/// Planned tile identity for a standalone image attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageShardSpec {
    /// Stable zero-based shard index in row-major order.
    pub shard_index: u32,
    /// Pixel tile rectangle.
    pub tile_box: ImageTileBox,
    /// Stable row-major reading-order key.
    pub reading_order_key: String,
    /// Digest derived from source hash, tile coordinates, and output profile.
    pub shard_digest: String,
}

/// Full standalone image shard plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageShardPlan {
    /// Source image path.
    pub source_path: PathBuf,
    /// SHA-256 digest of the source image bytes.
    pub source_content_hash: String,
    /// Source image MIME type.
    pub source_mime_type: ImageMimeType,
    /// Source image width in pixels.
    pub source_width: u32,
    /// Source image height in pixels.
    pub source_height: u32,
    /// Output MIME type for materialized tiles.
    pub output_mime_type: ImageMimeType,
    /// Planner options used to produce this plan.
    pub options: ImageShardOptions,
    /// Planned image tiles.
    pub shards: Vec<ImageShardSpec>,
}

/// Materialized tile output for one image shard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedImageShard {
    /// Planned shard spec.
    pub spec: ImageShardSpec,
    /// Tile image output path.
    pub image_path: PathBuf,
    /// SHA-256 digest of the materialized tile bytes.
    pub raster_sha256: String,
    /// Materialized tile byte length.
    pub byte_len: u64,
    /// Materialized tile width in pixels.
    pub width: u32,
    /// Materialized tile height in pixels.
    pub height: u32,
    /// Materialized tile MIME type.
    pub image_mime_type: ImageMimeType,
}

/// Complete materialization result for a standalone image attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageShardManifest {
    /// Deterministic image shard plan.
    pub plan: ImageShardPlan,
    /// Materialized shard files.
    pub tiles: Vec<MaterializedImageShard>,
}
