//! `parsers::markdown::relations::types` owns Wendao markdown relations types behavior.

use crate::entity::RelationType;
use crate::link_graph::addressing::Address;

/// Source scope for an explicit property-drawer relation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitRelationSource {
    /// Slash-delimited heading path for the owning section.
    pub heading_path: String,
    /// Explicit `:ID:` anchor declared by the owning section, when present.
    pub explicit_id: Option<String>,
}

impl ExplicitRelationSource {
    /// Resolve the owning section into the canonical Triple-A address shape.
    #[must_use]
    pub fn scope_address(&self) -> Option<Address> {
        if let Some(id) = &self.explicit_id {
            return Some(Address::id(id.clone()));
        }

        if self.heading_path.trim().is_empty() {
            return None;
        }

        Some(Address::path(
            self.heading_path
                .split(" / ")
                .map(str::trim)
                .filter(|component| !component.is_empty()),
        ))
    }

    /// Format the owning scope as a display string.
    #[must_use]
    pub fn scope_display(&self) -> Option<String> {
        self.scope_address()
            .map(|address| address.to_display_string())
    }
}

/// Target reference for an explicit property-drawer relation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitRelationTarget {
    /// Optional note or document alias from a wiki-link target.
    pub note_target: Option<String>,
    /// Optional scoped address inside the target note or the current note.
    pub address: Option<Address>,
    /// Original property token before parsing.
    pub original: String,
}

impl ExplicitRelationTarget {
    /// Format the target as a note plus optional scoped suffix.
    #[must_use]
    pub fn display(&self) -> String {
        match (&self.note_target, &self.address) {
            (Some(note_target), Some(Address::Id(id))) => format!("{note_target}#{id}"),
            (Some(note_target), Some(Address::Hash(hash))) => format!("{note_target}@{hash}"),
            (Some(note_target), Some(address)) => {
                format!("{note_target}#{}", address.to_display_string())
            }
            (Some(note_target), None) => note_target.clone(),
            (None, Some(address)) => address.to_display_string(),
            (None, None) => self.original.clone(),
        }
    }
}

/// Canonical explicit relation row parsed from one section property drawer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitSectionRelation {
    /// Property drawer key that declared the relation.
    pub property_key: String,
    /// Semantic relation type owned by that property key.
    pub relation_type: RelationType,
    /// Owning source section scope.
    pub source: ExplicitRelationSource,
    /// Parsed target note and optional scoped address.
    pub target: ExplicitRelationTarget,
}
