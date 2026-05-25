use super::{
    WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE, candidate_discovery_attempt_receipt,
    candidate_discovery_receipt,
};

#[test]
fn candidate_discovery_attempt_receipt_records_elapsed_time() {
    let receipt = candidate_discovery_attempt_receipt("search strategy", "docs", 3, 42);

    assert_eq!(receipt.get("rowCount"), Some(&serde_json::json!(3)));
    assert_eq!(receipt.get("elapsedMs"), Some(&serde_json::json!(42)));
}

#[test]
fn candidate_discovery_receipt_uses_gateway_retrieval_source() {
    let receipt = candidate_discovery_receipt(
        "main",
        32,
        123,
        &[candidate_discovery_attempt_receipt(
            "hybrid search",
            "docs",
            24,
            10,
        )],
    );

    assert_eq!(
        receipt.get("candidateInputSource"),
        Some(&serde_json::json!(
            WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE
        ))
    );
    assert_eq!(
        receipt.get("receiptSource"),
        Some(&serde_json::json!(
            WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE
        ))
    );
    assert_eq!(
        receipt.get("retrievalOwner"),
        Some(&serde_json::json!("wendao-gateway"))
    );
    assert_eq!(
        receipt.get("candidateInputCount"),
        Some(&serde_json::json!(32))
    );
}
