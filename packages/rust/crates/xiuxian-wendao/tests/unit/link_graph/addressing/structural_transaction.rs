//! Unit tests for `structural_transaction` module.

use super::{StructuralTransaction, StructuralTransactionCoordinator};

fn make_test_transaction(
    doc_id: &str,
    byte_range: (usize, usize),
    byte_delta: i64,
    line_delta: i64,
) -> StructuralTransaction {
    StructuralTransaction::new(
        doc_id.to_string(),
        format!("{doc_id}.md"),
        Some(format!("{doc_id}#section")),
        byte_range,
        byte_delta,
        line_delta,
        "old_hash".to_string(),
        "new_hash".to_string(),
        Some("test_agent".to_string()),
    )
}

#[test]
fn test_adjust_byte_range_before_edit() {
    let tx = make_test_transaction("doc", (100, 200), 50, 2);
    // Range before edit
    let adjusted = tx.adjust_byte_range((0, 50));
    assert_eq!(adjusted, Some((0, 50)));
}

#[test]
fn test_adjust_byte_range_after_edit() {
    let tx = make_test_transaction("doc", (100, 200), 50, 2);
    // Range after edit - should shift by +50
    let adjusted = tx.adjust_byte_range((250, 300));
    assert_eq!(adjusted, Some((300, 350)));
}

#[test]
fn test_adjust_byte_range_after_negative_delta() {
    let tx = make_test_transaction("doc", (100, 200), -50, -2);
    let adjusted = tx.adjust_byte_range((250, 300));
    assert_eq!(adjusted, Some((200, 250)));
}

#[test]
fn test_adjust_byte_range_overlapping_conflict() {
    let tx = make_test_transaction("doc", (100, 200), 50, 2);
    // Range overlapping edit
    let adjusted = tx.adjust_byte_range((150, 250));
    assert_eq!(adjusted, None);
}

#[test]
fn test_compatible_transactions_different_docs() {
    let tx1 = make_test_transaction("doc1", (100, 200), 50, 2);
    let tx2 = make_test_transaction("doc2", (100, 200), 50, 2);
    assert!(tx1.is_compatible_with(&tx2));
}

#[test]
fn test_compatible_transactions_non_overlapping() {
    let tx1 = make_test_transaction("doc", (100, 200), 50, 2);
    let tx2 = make_test_transaction("doc", (300, 400), 10, 1);
    assert!(tx1.is_compatible_with(&tx2));
}

#[test]
fn test_incompatible_transactions_overlapping() {
    let tx1 = make_test_transaction("doc", (100, 200), 50, 2);
    let tx2 = make_test_transaction("doc", (150, 250), 10, 1);
    assert!(!tx1.is_compatible_with(&tx2));
}

#[test]
fn test_coordinator_record_and_flush() {
    let mut coordinator = StructuralTransactionCoordinator::new();
    let tx = make_test_transaction("doc", (100, 200), 50, 2);

    assert!(coordinator.record(tx).is_ok());
    let signals = coordinator.flush_pending();
    assert_eq!(signals.len(), 1);
}

#[test]
fn test_coordinator_rejects_conflicting() {
    let mut coordinator = StructuralTransactionCoordinator::new();
    let tx1 = make_test_transaction("doc", (100, 200), 50, 2);
    let tx2 = make_test_transaction("doc", (150, 250), 10, 1);

    assert!(coordinator.record(tx1).is_ok());
    assert!(coordinator.record(tx2).is_err());
}

#[test]
fn test_coordinator_adjust_byte_range() {
    let mut coordinator = StructuralTransactionCoordinator::new();
    let tx = make_test_transaction("doc", (100, 200), 50, 2);
    assert!(coordinator.record(tx).is_ok());

    let adjusted = coordinator.adjust_byte_range("doc", (250, 300));
    assert_eq!(adjusted, Some((300, 350)));
}
