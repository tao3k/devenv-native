//! Triple-A Addressing Protocol for semantic node resolution.
//!
//! Implements the three-layer addressing protocol from the Project Anchor blueprint:
//! 1. **Anchor** (explicit ID): Resolve by `:ID:` property drawer attribute
//! 2. **AST Path** (structural path): Resolve by heading hierarchy path
//! 3. **Alias** (content hash): Resolve by Blake3 content fingerprint
//!
//! ## Dual-Index Architecture
//!
//! This module provides two complementary index types:
//! - **`RegistryIndex`**: O(1) lookup for stable ID anchoring (used by agents/systems)
//! - **`TopologyIndex`**: Fuzzy path discovery for human-friendly navigation
//!
//! ## Usage
//!
//! ```ignore
//! use crate::link_graph::addressing::{Address, resolve_node, ResolveMode};
//!
//! // Resolve by explicit ID
//! let addr = Address::Id("arch-v1".to_string());
//! let node = resolve_node(&index, &addr, "doc.md");
//!
//! // Resolve by structural path
//! let addr = Address::Path(vec!["Architecture".to_string(), "Storage".to_string()]);
//! let node = resolve_node(&index, &addr, "doc.md");
//!
//! // Resolve by content hash (self-healing)
//! let addr = Address::Hash("a1b2c3d4e5f6".to_string());
//! let node = resolve_node(&index, &addr, "doc.md");
//!
//! // Resolve block within section
//! let addr = Address::Block {
//!     section_path: vec!["Architecture".to_string()],
//!     block_addr: BlockAddress::new(BlockKindSpecifier::Paragraph, 2),
//! };
//!
//! // Use dual indices for enhanced resolution
//! let registry = RegistryIndex::build_from_trees(trees);
//! let topology = TopologyIndex::build_from_trees(trees);
//! let result = resolve_with_indices(&registry, &topology, &addr, doc_id, ResolveMode::Anchor);
//! ```

mod address;
mod errors;
mod indices;
mod mutation;
mod node_lookup;
mod registry;
mod resolve;
mod skeleton_rerank;
mod structural_transaction;
#[path = "topology/mod.rs"]
mod topology;

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/addressing/mod.rs"]
mod tests;

pub use address::{Address, EnhancedResolvedNode, ResolveMode, ResolvedNode};
pub use errors::{ModificationError, ResolveError};
pub use indices::{build_hash_index, build_id_index};
pub use mutation::{
    ModificationResult, adjust_line_range, replace_byte_range, update_section_content,
};
pub use node_lookup::resolve_node;
pub use registry::{IdCollision, IndexedNode, RegistryBuildResult, RegistryIndex};
pub use resolve::resolve_with_indices;
pub use skeleton_rerank::{SkeletonRerankOptions, SkeletonValidatedHit, skeleton_rerank};
pub use structural_transaction::{
    StructuralTransaction, StructuralTransactionCoordinator, StructureUpdateSignal,
};
pub use topology::{MatchType, PathEntry, PathMatch, TopologyIndex};
