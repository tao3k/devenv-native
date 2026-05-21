//! `enhancer::markdown_config::index` owns Wendao enhancer markdown config index behavior.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::blocks::extract_markdown_config_blocks;
use super::types::MarkdownConfigBlock;

/// O(1) in-memory index for markdown configuration blocks keyed by `id`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkdownConfigMemoryIndex {
    blocks_by_id: HashMap<String, MarkdownConfigBlock>,
}

impl MarkdownConfigMemoryIndex {
    /// Creates an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds an index from markdown by parsing tagged AST blocks.
    #[must_use]
    pub fn from_markdown(markdown: &str) -> Self {
        Self::from_blocks(extract_markdown_config_blocks(markdown))
    }

    /// Builds an index from pre-extracted blocks.
    #[must_use]
    pub fn from_blocks<I>(blocks: I) -> Self
    where
        I: IntoIterator<Item = MarkdownConfigBlock>,
    {
        let mut index = Self::new();
        index.extend(blocks);
        index
    }

    /// Inserts or replaces one block by its exact `id`.
    pub fn insert(&mut self, block: MarkdownConfigBlock) -> Option<MarkdownConfigBlock> {
        self.blocks_by_id.insert(block.id.clone(), block)
    }

    /// Extends the index with multiple blocks.
    pub fn extend<I>(&mut self, blocks: I)
    where
        I: IntoIterator<Item = MarkdownConfigBlock>,
    {
        for block in blocks {
            self.insert(block);
        }
    }

    /// Returns a block by exact `id` lookup in O(1).
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn get(&self, id: &str) -> Option<&MarkdownConfigBlock> {
        self.blocks_by_id.get(id)
    }

    /// Returns the number of indexed blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks_by_id.len()
    }

    /// Returns `true` when the index has no blocks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks_by_id.is_empty()
    }

    /// Returns an iterator over all indexed config blocks.
    pub fn values(&self) -> impl Iterator<Item = &MarkdownConfigBlock> {
        self.blocks_by_id.values()
    }
}
