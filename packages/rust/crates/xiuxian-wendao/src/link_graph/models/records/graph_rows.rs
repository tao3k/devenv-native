//! `link_graph::models::records::graph_rows` owns Wendao models records graph rows behavior.

use crate::link_graph::models::query::LinkGraphDirection;
use serde::{Deserialize, Serialize};

/// Neighbor row for link traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphNeighbor {
    /// Stem identifier.
    pub stem: String,
    /// Relative direction to queried note.
    pub direction: LinkGraphDirection,
    /// Hop distance from queried note.
    pub distance: usize,
    /// Optional title.
    pub title: String,
    /// Relative path.
    pub path: String,
}

/// Metadata row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphMetadata {
    /// Stem identifier.
    pub stem: String,
    /// Optional title.
    pub title: String,
    /// Relative path.
    pub path: String,
    /// Tag list.
    pub tags: Vec<String>,
}

/// Summary stats.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LinkGraphStats {
    /// Total indexed notes.
    pub total_notes: usize,
    /// Notes with no incoming/outgoing links.
    pub orphans: usize,
    /// Total directed links.
    pub links_in_graph: usize,
    /// Total graph nodes.
    pub nodes_in_graph: usize,
}
