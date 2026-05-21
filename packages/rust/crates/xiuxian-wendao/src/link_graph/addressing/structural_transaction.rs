//! Structural Transactions for Concurrent Editing (Blueprint Section 3.2).
//!
//! This module implements the "Structural Transactions" concept:
//! - After each edit, the system should trigger instantaneous AST re-parsing
//! - Use `adjust_line_range` to calculate drifted viewports for concurrent agents
//! - Broadcast "structure update" signals to coordinate multi-agent collaboration

use serde::{Deserialize, Serialize};

fn timestamp_ms_now() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        Err(_) => 0,
    }
}

fn apply_signed_delta(base: usize, delta: i64) -> Option<usize> {
    if delta >= 0 {
        let magnitude = usize::try_from(delta).ok()?;
        base.checked_add(magnitude)
    } else {
        let magnitude = match usize::try_from(delta.unsigned_abs()) {
            Ok(magnitude) => magnitude,
            Err(_) => usize::MAX,
        };
        Some(base.saturating_sub(magnitude))
    }
}

/// Result of a structural edit operation.
///
/// Contains all information needed for:
/// 1. Concurrent agents to adjust their viewports
/// 2. The system to trigger AST re-parsing
/// 3. Conflict detection when multiple agents edit overlapping regions
///
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralTransaction {
    /// Document that was modified.
    pub doc_id: String,
    /// Document path relative to root.
    pub doc_path: String,
    /// Node ID that was edited (if targeting a specific node).
    pub node_id: Option<String>,
    /// Byte range that was modified (original, before edit).
    pub original_byte_range: (usize, usize),
    /// Change in byte count (positive = grew, negative = shrank).
    pub byte_delta: i64,
    /// Change in line count (positive = added lines, negative = removed).
    pub line_delta: i64,
    /// Content hash before edit.
    pub old_hash: String,
    /// Content hash after edit.
    pub new_hash: String,
    /// Timestamp of the edit (Unix epoch milliseconds).
    pub timestamp_ms: u64,
    /// Agent identifier that performed the edit.
    pub agent_id: Option<String>,
}

impl StructuralTransaction {
    /// Create a new structural transaction record.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    /// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn new(
        doc_id: String,
        doc_path: String,
        node_id: Option<String>,
        original_byte_range: (usize, usize),
        byte_delta: i64,
        line_delta: i64,
        old_hash: String,
        new_hash: String,
        agent_id: Option<String>,
    ) -> Self {
        Self {
            doc_id,
            doc_path,
            node_id,
            original_byte_range,
            byte_delta,
            line_delta,
            old_hash,
            new_hash,
            timestamp_ms: timestamp_ms_now(),
            agent_id,
        }
    }

    /// Adjust a byte range for concurrent access after this transaction.
    ///
    /// If the range is before the edit, it's unchanged.
    /// If the range is after the edit, it's shifted by `byte_delta`.
    /// If the range overlaps the edit, returns None (conflict).
    #[must_use]
    /// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
    pub fn adjust_byte_range(&self, range: (usize, usize)) -> Option<(usize, usize)> {
        let (start, end) = range;
        let (edit_start, edit_end) = self.original_byte_range;

        // Range is before edit - no change
        if end <= edit_start {
            return Some((start, end));
        }

        // Range is after edit - shift by delta
        if start >= edit_end {
            let delta = self.byte_delta;
            let new_start = apply_signed_delta(start, delta)?;
            let new_end = apply_signed_delta(end, delta)?;
            return Some((new_start, new_end));
        }

        // Range overlaps edit - conflict
        None
    }

    /// Check if a given byte range conflicts with this transaction.
    #[must_use]
    /// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
    pub fn conflicts_with(&self, range: (usize, usize)) -> bool {
        self.adjust_byte_range(range).is_none()
    }

    /// Check if this transaction can be applied concurrently with another.
    ///
    /// Two transactions are compatible if they don't overlap in byte ranges.
    #[must_use]
    pub fn is_compatible_with(&self, other: &StructuralTransaction) -> bool {
        // Different documents are always compatible
        if self.doc_id != other.doc_id {
            return true;
        }

        // Check if ranges overlap
        let self_range = self.original_byte_range;
        let other_range = other.original_byte_range;

        // No overlap if one ends before the other starts
        self_range.1 <= other_range.0 || other_range.1 <= self_range.0
    }
}

/// Broadcast signal for structure updates.
///
/// When an edit completes, this signal is broadcast to inform
/// concurrent agents that they may need to adjust their viewports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureUpdateSignal {
    /// The transaction that triggered this update.
    pub transaction: StructuralTransaction,
    /// Nodes that were affected (directly or indirectly).
    pub affected_node_ids: Vec<String>,
    /// Whether the document structure changed (headings added/removed/renamed).
    pub structure_changed: bool,
}

impl StructureUpdateSignal {
    /// Create a new structure update signal.
    #[must_use]
    pub fn new(
        transaction: StructuralTransaction,
        affected_node_ids: Vec<String>,
        structure_changed: bool,
    ) -> Self {
        Self {
            transaction,
            affected_node_ids,
            structure_changed,
        }
    }

    /// Create a minimal signal for simple content edits (no structure change).
    #[must_use]
    pub fn content_only(transaction: StructuralTransaction) -> Self {
        let node_id = transaction.node_id.clone();
        Self {
            transaction,
            affected_node_ids: node_id.into_iter().collect(),
            structure_changed: false,
        }
    }
}

/// Coordinator for managing concurrent structural transactions.
///
/// This is a simple in-memory coordinator. For production use with
/// multiple processes, this would need to be backed by a distributed
/// coordination service (e.g., Valkey pub/sub).
#[derive(Debug, Default)]
pub struct StructuralTransactionCoordinator {
    /// Pending transactions that haven't been broadcast yet.
    pending: Vec<StructuralTransaction>,
    /// Last known transaction per document.
    last_per_doc: std::collections::HashMap<String, StructuralTransaction>,
}

impl StructuralTransactionCoordinator {
    /// Create a new coordinator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            last_per_doc: std::collections::HashMap::new(),
        }
    }

    /// Record a new transaction.
    ///
    /// Returns `Ok(())` if the transaction is compatible with pending transactions,
    /// or `Err(conflicting_transactions)` if there are conflicts.
    ///
    /// # Errors
    ///
    /// Returns the conflicting pending transactions when the new transaction overlaps an
    /// in-flight edit for the same document.
    pub fn record(
        &mut self,
        transaction: StructuralTransaction,
    ) -> Result<(), Vec<StructuralTransaction>> {
        // Check for conflicts with pending transactions
        let conflicts: Vec<&StructuralTransaction> = self
            .pending
            .iter()
            .filter(|pending| !pending.is_compatible_with(&transaction))
            .collect();

        if !conflicts.is_empty() {
            return Err(conflicts.into_iter().cloned().collect());
        }

        // Update last transaction for this document
        self.last_per_doc
            .insert(transaction.doc_id.clone(), transaction.clone());

        // Add to pending
        self.pending.push(transaction);
        Ok(())
    }

    /// Flush pending transactions and return them as update signals.
    pub fn flush_pending(&mut self) -> Vec<StructureUpdateSignal> {
        let signals: Vec<StructureUpdateSignal> = self
            .pending
            .drain(..)
            .map(StructureUpdateSignal::content_only)
            .collect();
        signals
    }

    /// Get the last transaction for a document.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn last_transaction_for(&self, doc_id: &str) -> Option<&StructuralTransaction> {
        self.last_per_doc.get(doc_id)
    }

    /// Adjust a byte range for a document based on all pending transactions.
    #[must_use]
    /// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn adjust_byte_range(&self, doc_id: &str, range: (usize, usize)) -> Option<(usize, usize)> {
        let mut current = range;
        for tx in &self.pending {
            if tx.doc_id == doc_id {
                current = tx.adjust_byte_range(current)?;
            }
        }
        Some(current)
    }

    /// Check if a byte range conflicts with any pending transaction for a document.
    #[must_use]
    /// Tuple API boundary: this public API preserves byte or count pairs used by existing addressing contracts.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn has_conflict(&self, doc_id: &str, range: (usize, usize)) -> bool {
        self.pending
            .iter()
            .any(|tx| tx.doc_id == doc_id && tx.conflicts_with(range))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/addressing/structural_transaction.rs"]
mod tests;
