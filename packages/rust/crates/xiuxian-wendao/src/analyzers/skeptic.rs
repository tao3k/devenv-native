//! Skeptic audit logic for verifying documentation and symbol consistency.

use super::records::{DocRecord, RelationKind, RelationRecord, SymbolRecord};
use std::collections::HashMap;

/// Result of a skepticism audit for a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditResult {
    /// The entity is verified against its documentation or implementation.
    Verified,
    /// The entity is suspicious or has mismatched documentation.
    Unverified,
    /// No sufficient evidence to audit.
    Unknown,
}

impl AuditResult {
    /// Returns the string representation of the audit result.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Unverified => "unverified",
            Self::Unknown => "unknown",
        }
    }
}

/// Perform a basic skepticism audit on all symbols in the analysis output.
/// Returns a map from symbol ID to its verification state.
pub fn audit_symbols(
    symbols: &[SymbolRecord],
    docs: &[DocRecord],
    relations: &[RelationRecord],
) -> HashMap<String, String> {
    let mut audit_map = HashMap::new();

    // 1. Build a lookup for which symbols are documented by which docs
    let mut symbol_to_docs = HashMap::new();
    for rel in relations {
        if rel.kind == RelationKind::Documents {
            symbol_to_docs
                .entry(rel.target_id.clone())
                .or_insert_with(Vec::new)
                .push(rel.source_id.clone());
        }
    }

    let doc_map: HashMap<String, &DocRecord> = docs
        .iter()
        .map(|doc| (doc.doc_id.to_string(), doc))
        .collect();

    // 2. Audit each symbol
    for symbol in symbols {
        let result = if let Some(doc_ids) = symbol_to_docs.get(symbol.symbol_id.as_str()) {
            // Basic check: does any associated doc title contain the symbol name?
            // (In a future version, we would read the actual file content via VFS)
            let has_valid_doc = doc_ids.iter().any(|doc_id| {
                if let Some(doc) = doc_map.get(doc_id) {
                    let title = doc.title.to_lowercase();
                    let name = symbol.name.to_lowercase();

                    title.contains(&name) || name.contains(&title)
                } else {
                    false
                }
            });

            if has_valid_doc {
                AuditResult::Verified
            } else {
                AuditResult::Unverified
            }
        } else {
            // No documentation linked
            AuditResult::Unknown
        };

        audit_map.insert(symbol.symbol_id.to_string(), result.as_str().to_string());
    }

    audit_map
}

#[cfg(test)]
#[path = "../../tests/unit/analyzers/skeptic.rs"]
mod tests;
