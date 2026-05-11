//! Markdown block extraction and block identity contracts.

mod api;
mod counter;
mod sourcepos;
mod types;

pub use api::extract_blocks;
pub use sourcepos::line_col_to_byte_range;
pub use types::{
    BlockCore, BlockCoreRequest, BlockExplicitId, BlockKindIdentity, MarkdownBlock,
    MarkdownBlockKind, compute_block_hash,
};
