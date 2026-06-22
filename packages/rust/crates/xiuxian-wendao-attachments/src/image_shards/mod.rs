//! Standalone image shard planning and materialization.

mod materialize;
mod plan;
mod types;

pub use materialize::materialize_image_shards;
pub use plan::plan_image_shards;
pub use types::{
    ImageShardManifest, ImageShardOptions, ImageShardPlan, ImageShardSpec, ImageTileBox,
    MaterializedImageShard,
};

#[cfg(test)]
#[path = "../../tests/unit/image_shards.rs"]
mod tests;
