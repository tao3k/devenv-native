use crate::workflow_kernel::tests::support::{WorkflowEdgeKind, WorkflowStageFacts};

#[test]
fn workflow_kernel_facts_describe_arrow_edges_without_owning_arrow_buffers() {
    let facts = WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_input", "v1")
        .with_item_count(12)
        .with_cache_hit_count(3);

    assert_eq!(facts.item_count, Some(12));
    assert_eq!(facts.cache_hit_count, Some(3));
    assert_eq!(
        facts.edge_kind,
        Some(WorkflowEdgeKind::ArrowRecordBatch {
            schema_name: "xiuxian_wendao.audio_shard_input".to_owned(),
            schema_version: "v1".to_owned(),
        })
    );
}
