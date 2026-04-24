use xiuxian_db_store::{LanceArray, LanceStringArray};

use crate::analyzers::RefineEntityDocResponse;
use crate::gateway::studio::router::handlers::repo::analysis::refine_doc_flight::{
    refine_doc_batch, refine_doc_metadata,
};

fn demo_response() -> RefineEntityDocResponse {
    RefineEntityDocResponse {
        repo_id: "gateway-sync".to_string(),
        entity_id: "repo:gateway-sync:symbol:GatewaySyncPkg.solve".to_string(),
        refined_content: "## Refined Explanation\n\nUse `solve()`.".to_string(),
        verification_state: "verified".to_string(),
    }
}

#[test]
fn refine_doc_batch_preserves_response_payload() {
    let batch = refine_doc_batch(&demo_response())
        .unwrap_or_else(|error| panic!("batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(entity_id_column) = batch.column_by_name("entityId") else {
        panic!("entityId column");
    };
    let Some(entity_ids) = entity_id_column.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("entityId column type");
    };
    assert_eq!(
        entity_ids.value(0),
        "repo:gateway-sync:symbol:GatewaySyncPkg.solve"
    );

    let Some(refined_content_column) = batch.column_by_name("refinedContent") else {
        panic!("refinedContent column");
    };
    let Some(refined_content) = refined_content_column
        .as_any()
        .downcast_ref::<LanceStringArray>()
    else {
        panic!("refinedContent column type");
    };
    assert_eq!(
        refined_content.value(0),
        "## Refined Explanation\n\nUse `solve()`."
    );
}

#[test]
fn refine_doc_metadata_preserves_summary_fields() {
    let metadata = refine_doc_metadata(&demo_response())
        .unwrap_or_else(|error| panic!("metadata should encode: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["repoId"], "gateway-sync");
    assert_eq!(
        payload["entityId"],
        "repo:gateway-sync:symbol:GatewaySyncPkg.solve"
    );
    assert_eq!(payload["verificationState"], "verified");
}
